use crate::chunk::*;
use crate::compiler::*;
use crate::string;
use crate::value::*;
use std::collections::HashMap;
use std::convert::TryInto;

#[derive(PartialEq)]
pub enum InterpretError {
  CompileError,
  RuntimeError,
  InternalError(&'static str),
}

#[derive(Default)]
pub struct VM {
  chunk: Option<Chunk>,
  ip: usize,
  stack: Vec<Value>,
  globals: HashMap<&'static str, Value>,
}

type Result<T> = std::result::Result<T, InterpretError>;

impl VM {
  pub fn new() -> VM {
    VM::default()
  }

  fn reset_stack(&mut self) {
    self.stack.clear()
  }

  fn chunk(&mut self) -> Result<&Chunk> {
    match self.chunk.as_ref() {
      Some(value) => Ok(value),
      _ => Err(InterpretError::InternalError("No chunk!")),
    }
  }

  fn runtime_error<'a>(&mut self, string: &'a str) -> Result<()> {
    eprintln!("{}", string);

    let instruction = self.ip - 1;
    let line = self.chunk()?.lines[instruction as usize];
    eprintln!("[line {}] in script", line);
    self.reset_stack();
    Err(InterpretError::RuntimeError)
  }

  pub fn interpret(&mut self, source: &String) -> Result<()> {
    let mut chunk = Chunk::new();

    if !compile(source, &mut chunk) {
      return Err(InterpretError::CompileError);
    }

    self.chunk = Some(chunk);
    self.ip = 0;

    self.run()
  }

  fn push(&mut self, value: Value) {
    self.stack.push(value)
  }

  fn pop(&mut self) -> Result<Value> {
    match self.stack.pop() {
      Some(value) => Ok(value),
      _ => Err(InterpretError::InternalError("Can't pop on empty stack.")),
    }
  }

  fn peek(&self, index: usize) -> Result<&Value> {
    let len = self.stack.len();
    match self.stack.get(len - 1 - index) {
      Some(value) => Ok(value),
      None => Err(InterpretError::InternalError("Can't peek on empty stack.")),
    }
  }

  fn read_byte(&mut self) -> Result<u8> {
    self.ip += 1;
    let ip = self.ip - 1;
    let chunk = self.chunk()?;
    match chunk.code.get(ip) {
      Some(byte) => Ok(*byte),
      _ => return Err(InterpretError::InternalError("Failed to read byte.")),
    }
  }

  fn read_constant(&mut self) -> Result<&Value> {
    let constant = self.read_byte()? as usize;
    match self.chunk()?.constants.get(constant) {
      Some(value) => Ok(value),
      _ => return Err(InterpretError::InternalError("Failed to read constant.")),
    }
  }

  fn read_string(&mut self) -> Result<&string::Handle> {
    match self.read_constant()? {
      Value::String(handle) => Ok(handle),
      _ => return Err(InterpretError::InternalError("Value was not a string.")),
    }
  }

  fn run(&mut self) -> Result<()> {
    macro_rules! binary_op {
      ($op: tt, $variant: ident) => {{
        let value = match (self.peek(1)?, self.peek(0)?) {
          (Value::Number(b), Value::Number(a)) => (a $op b),
          _ => {
            return self.runtime_error("Operands must be numbers.");
          }
        };

        self.pop()?;
        self.pop()?;
        self.push(Value::$variant(value))
      }};
    }

    loop {
      {
        #![cfg(feature = "trace-execution")]
        print!("          ");
        for value in self.stack.iter() {
          print!("[ ");
          value.print();
          print!(" ]");
        }
        println!("");
        let ip = self.ip;
        self.chunk()?.disassemble_instruction(ip);
      }

      let instruction = match self.read_byte()?.try_into() {
        Ok(op) => op,
        Err(value) => {
          let message = format!("Got unexpected instruction: '{}'", value);
          return self.runtime_error(message.as_str());
        }
      };

      match instruction {
        Op::Constant => {
          let constant = self.read_constant()?;
          {
            #![cfg(feature = "trace-execution")]
            constant.println();
          }
          let copy = *constant;
          self.push(copy);
        }
        Op::Nil => self.push(Value::Nil),
        Op::True => self.push(Value::Bool(true)),
        Op::False => self.push(Value::Bool(false)),
        Op::Pop => {
          self.pop()?;
        }
        Op::GetLocal => {
          let slot = self.read_byte()?;
          self.push(self.stack[slot as usize]);
        }
        Op::SetLocal => {
          let slot = self.read_byte()?;
          self.stack[slot as usize] = *self.peek(0)?;
        }
        Op::GetGlobal => {
          let name = self.read_string()?.as_str().string;
          match self.globals.get(name) {
            Some(value) => {
              let copy = *value;
              self.push(copy);
            }
            _ => {
              let error = format!("Undefined variable '{}'.", name);
              return self.runtime_error(error.as_str());
            }
          }
        }
        Op::DefineGlobal => {
          let name = self.read_string()?.as_str().string;
          self.globals.insert(name, *self.peek(0)?);
          self.pop()?;
        }
        Op::SetGlobal => {
          let name = self.read_string()?;
          let string = name.as_str().string;
          if self.globals.insert(string, *self.peek(0)?).is_none() {
            self.globals.remove(string);
            let error = format!("Undefined variable '{}'.", string);
            return self.runtime_error(error.as_str());
          }
        }
        Op::Equal => {
          let b = self.pop();
          let a = self.pop();
          self.push(Value::Bool(a == b));
        }
        Op::Greater => binary_op!(>, Bool),
        Op::Less => binary_op!(<, Bool),
        Op::Add => {
          let value = match (self.peek(0)?, self.peek(1)?) {
            (Value::Number(b), Value::Number(a)) => Value::Number(a + b),
            (Value::String(b), Value::String(a)) => Value::String(a + b),
            _ => {
              return self.runtime_error("Operands must be numbers.");
            }
          };

          self.pop()?;
          self.pop()?;
          self.push(value);
        }
        Op::Subtract => binary_op!(-, Number),
        Op::Multiply => binary_op!(*, Number),
        Op::Divide => binary_op!(/, Number),
        Op::Not => {
          let value = self.pop()?.is_falsy();
          self.push(Value::Bool(value))
        }
        Op::Negate => {
          let num = match self.peek(0)? {
            Value::Number(num) => *num,
            _ => {
              return self.runtime_error("Operand must be a number.");
            }
          };
          self.pop()?;
          self.push(Value::Number(-num))
        }
        Op::Print => {
          self.pop()?.println();
        }
        Op::Return => {
          return Ok(());
        }
      }
    }
  }
}
