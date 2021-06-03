use crate::chunk::*;
use crate::compiler::*;
use crate::value::*;
use std::collections::HashMap;
use std::convert::TryInto;

#[derive(PartialEq)]
pub enum InterpretResult {
  Ok,
  CompileError,
  RuntimeError,
}

#[derive(Default)]
pub struct VM {
  chunk: Option<Chunk>,
  ip: usize,
  stack: Vec<Value>,
  globals: HashMap<&'static str, Value>,
}

impl VM {
  pub fn new() -> VM {
    VM::default()
  }

  fn reset_stack(&mut self) {
    self.stack.clear()
  }

  fn runtime_error(&mut self, string: &str) {
    eprintln!("{}", string);

    let instruction = self.ip - 1;
    let line = self.chunk.as_ref().unwrap().lines[instruction as usize];
    eprintln!("[line {}] in script", line);
    self.reset_stack();
  }

  pub fn interpret(&mut self, source: &String) -> InterpretResult {
    let mut chunk = Chunk::new();

    if !compile(source, &mut chunk) {
      return InterpretResult::CompileError;
    }

    self.chunk = Some(chunk);
    self.ip = 0;

    let result = self.run();
    self.chunk = None;
    result
  }

  fn push(&mut self, value: Value) {
    self.stack.push(value)
  }

  fn pop(&mut self) -> Value {
    self.stack.pop().unwrap()
  }

  fn peek(&self, index: usize) -> &Value {
    match self.stack.len() {
      0 => panic!(),
      n => &self.stack.get(n - 1 - index).unwrap(),
    }
  }

  fn run(&mut self) -> InterpretResult {
    macro_rules! read_byte {
      () => {{
        self.ip += 1;
        self.chunk.as_ref().unwrap().code[self.ip - 1]
      }};
    }

    macro_rules! read_constant {
      () => {
        &self.chunk.as_ref().unwrap().constants[read_byte!() as usize]
      };
    }

    macro_rules! read_string {
      () => {{
        match read_constant!() {
          Value::String(handle) => handle,
          _ => panic!(),
        }
      }};
    }

    macro_rules! binary_op {
      ($op: tt, $variant: ident) => {{
        let value = match (self.peek(1), self.peek(0)) {
          (Value::Number(b), Value::Number(a)) => (a $op b),
          _ => {
            self.runtime_error("Operands must be numbers.");
            return InterpretResult::RuntimeError;
          }
        };

        self.pop();
        self.pop();
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
        self
          .chunk
          .as_ref()
          .unwrap()
          .disassemble_instruction(self.ip);
      }

      let instruction = match read_byte!().try_into() {
        Ok(op) => op,
        Err(value) => {
          println!("Got unexpected instruction: '{}'", value);
          panic!();
        }
      };

      match instruction {
        Op::Constant => {
          let constant = read_constant!();
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
          self.pop();
        }
        Op::GetGlobal => {
          let name = read_string!();
          match self.globals.get(name.as_str().string) {
            Some(value) => {
              let copy = *value;
              self.push(copy);
            }
            _ => {
              let error = format!("Undefined variable '{}'.", name);
              self.runtime_error(error.as_str());
              return InterpretResult::RuntimeError;
            }
          }
        }
        Op::DefineGlobal => {
          let name = read_string!();
          self.globals.insert(name.as_str().string, *self.peek(0));
          self.pop();
        }
        Op::SetGlobal => {
          let name = read_string!();
          let string = name.as_str().string;
          if self.globals.insert(string, *self.peek(0)).is_none() {
            self.globals.remove(string);
            let error = format!("Undefined variable '{}'.", string);
            self.runtime_error(error.as_str());
            return InterpretResult::RuntimeError;
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
          let value = match (self.peek(0), self.peek(1)) {
            (Value::Number(b), Value::Number(a)) => Value::Number(a + b),
            (Value::String(b), Value::String(a)) => Value::String(a + b),
            _ => {
              self.runtime_error("Operands must be numbers.");
              return InterpretResult::RuntimeError;
            }
          };

          self.pop();
          self.pop();
          self.push(value);
        }
        Op::Subtract => binary_op!(-, Number),
        Op::Multiply => binary_op!(*, Number),
        Op::Divide => binary_op!(/, Number),
        Op::Not => {
          let value = self.pop().is_falsy();
          self.push(Value::Bool(value))
        }
        Op::Negate => match self.peek(0) {
          &Value::Number(num) => {
            self.pop();
            self.push(Value::Number(-num))
          }
          _ => {
            self.runtime_error("Operand must be a number.");
            return InterpretResult::RuntimeError;
          }
        },
        Op::Print => {
          self.pop().println();
        }
        Op::Return => {
          return InterpretResult::Ok;
        }
      }
    }
  }
}
