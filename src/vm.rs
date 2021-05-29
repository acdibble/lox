use crate::chunk::*;
use crate::value::*;
use std::convert::TryInto;

pub enum InterpretResult {
  Ok,
  CompileError,
  RuntimeError,
}

pub struct VM<'a> {
  chunk: Option<&'a Chunk>,
  ip: usize,
  stack: Vec<Value>,
}

impl<'a> VM<'a> {
  pub fn new() -> VM<'a> {
    VM {
      chunk: None,
      ip: 0,
      stack: Vec::new(),
    }
  }

  // fn reset_stack(&mut self) {
  //   self.stack_top = 0;
  // }

  pub fn interpret(&mut self, chunk: &'a Chunk) -> InterpretResult {
    self.chunk = Some(chunk);
    return self.run();
  }

  fn push(&mut self, value: Value) {
    self.stack.push(value)
  }

  fn pop(&mut self) -> Value {
    self.stack.pop().unwrap()
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
        self.chunk.unwrap().constants[read_byte!() as usize]
      };
    }

    loop {
      print!("          ");
      for value in self.stack.iter() {
        print!("[ ");
        print_value(*value);
        print!(" ]");
      }
      println!("");
      self.chunk.unwrap().disassemble_instruction(self.ip);

      let instruction = read_byte!().try_into();
      match instruction {
        Ok(Op::Constant) => {
          let constant = read_constant!();
          self.push(constant);
          print_value(constant);
          println!("");
        }
        Ok(Op::Add) => {
          let b = self.pop();
          let a = self.pop();
          self.push(a + b);
        }
        Ok(Op::Subtract) => {
          let b = self.pop();
          let a = self.pop();
          self.push(a - b);
        }
        Ok(Op::Multiply) => {
          let b = self.pop();
          let a = self.pop();
          self.push(a * b);
        }
        Ok(Op::Divide) => {
          let b = self.pop();
          let a = self.pop();
          self.push(a / b);
        }
        Ok(Op::Negate) => {
          let value = self.pop();
          self.push(-value)
        }
        Ok(Op::Return) => {
          print_value(self.pop());
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
