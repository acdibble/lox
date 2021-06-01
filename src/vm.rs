use crate::chunk::*;
use crate::compiler::*;
use crate::value::*;
use std::convert::TryInto;

#[derive(PartialEq)]
pub enum InterpretResult {
  Ok,
  CompileError,
  RuntimeError,
}

#[derive(Default)]
pub struct VM<'a> {
  chunk: Option<&'a Chunk>,
  ip: usize,
  stack: Vec<Value>,
}

impl<'a> VM<'a> {
  pub fn new() -> VM<'a> {
    VM::default()
  }

  fn reset_stack(&mut self) {
    self.stack.clear()
  }

  fn runtime_error(&mut self, string: &str) {
    eprintln!("{}", string);

    let instruction = self.ip - 1;
    let line = self.chunk.unwrap().lines[instruction as usize];
    eprintln!("[line {}] in script", line);
    self.reset_stack();
  }

  pub fn interpret(&mut self, source: &String) -> InterpretResult {
    let mut vm = VM::new();
    let mut chunk = Chunk::new();

    if !compile(source, &mut chunk) {
      return InterpretResult::CompileError;
    }

    vm.chunk = Some(&chunk);
    vm.ip = 0;

    return vm.run();
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
        self.chunk.unwrap().code[self.ip - 1]
      }};
    }

    macro_rules! read_constant {
      () => {
        &self.chunk.unwrap().constants[read_byte!() as usize]
      };
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
      print!("          ");
      for value in self.stack.iter() {
        print!("[ ");
        value.print();
        print!(" ]");
      }
      println!("");
      self.chunk.unwrap().disassemble_instruction(self.ip);

      let instruction = read_byte!().try_into();
      match instruction {
        Ok(Op::Constant) => {
          let constant = read_constant!();
          constant.print();
          self.push(constant.clone());
          println!("");
        }
        Ok(Op::Nil) => self.push(Value::Nil),
        Ok(Op::True) => self.push(Value::Bool(true)),
        Ok(Op::False) => self.push(Value::Bool(false)),
        Ok(Op::Equal) => {
          let b = self.pop();
          let a = self.pop();
          self.push(Value::Bool(a == b));
        }
        Ok(Op::Greater) => binary_op!(>, Bool),
        Ok(Op::Less) => binary_op!(<, Bool),
        Ok(Op::Add) => {
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
        Ok(Op::Subtract) => binary_op!(-, Number),
        Ok(Op::Multiply) => binary_op!(*, Number),
        Ok(Op::Divide) => binary_op!(/, Number),
        Ok(Op::Not) => {
          let value = self.pop().is_falsy();
          self.push(Value::Bool(value))
        }
        Ok(Op::Negate) => match self.peek(0) {
          &Value::Number(num) => {
            self.pop();
            self.push(Value::Number(-num))
          }
          _ => {
            self.runtime_error("Operand must be a number.");
            return InterpretResult::RuntimeError;
          }
        },
        Ok(Op::Return) => {
          self.pop().print();
          println!("");
          return InterpretResult::Ok;
        }
        Err(value) => {
          println!("Got unexpected instruction: '{}'", value);
          panic!();
        }
      }
    }
  }
}
