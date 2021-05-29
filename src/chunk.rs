use crate::value::*;
use std::convert::TryFrom;
use std::convert::TryInto;

#[repr(u8)]
pub enum Op {
  Constant,
  Return,
}

impl TryFrom<u8> for Op {
  type Error = u8;

  fn try_from(v: u8) -> Result<Self, Self::Error> {
    match v {
      x if x == Op::Return as u8 => Ok(Op::Return),
      x if x == Op::Constant as u8 => Ok(Op::Constant),
      _ => Err(v),
    }
  }
}

pub struct Chunk {
  code: Vec<u8>,
  constants: Vec<Value>,
  lines: Vec<i32>,
}

impl Chunk {
  pub fn new() -> Chunk {
    Chunk {
      code: Vec::new(),
      constants: Vec::new(),
      lines: Vec::new(),
    }
  }

  pub fn write(&mut self, byte: u8, line: i32) {
    self.code.push(byte);
    self.lines.push(line);
  }

  pub fn add_constant(&mut self, value: Value) -> u8 {
    self.constants.push(value);
    return (self.constants.len() - 1)
      .try_into()
      .expect("Too many constants");
  }

  pub fn disassemble(&self, name: &'static str) {
    println!("== {} ==", name);

    let mut offset: usize = 0;

    while offset < self.code.len() {
      offset = self.disassemble_instruction(offset)
    }
  }

  fn disassemble_instruction(&self, offset: usize) -> usize {
    print!("{:04} ", offset);

    if offset > 0 && self.lines[offset] == self.lines[offset - 1] {
      print!("   | ")
    } else {
      print!("{:4} ", self.lines[offset]);
    }

    let instruction = *self.code.get(offset).expect("Expect instruction");
    match instruction.try_into() {
      Ok(Op::Return) => self.simple_instruction("OP_RETURN", offset),
      Ok(Op::Constant) => self.constant_instruction("OP_CONSTANT", offset),
      Err(v) => {
        println!("Unknown opcode {}", v);
        offset + 1
      }
    }
  }

  fn simple_instruction(&self, name: &'static str, offset: usize) -> usize {
    println!("{}", name);
    offset + 1
  }

  fn constant_instruction(&self, name: &'static str, offset: usize) -> usize {
    let constant = *self
      .code
      .get(offset + 1)
      .expect("Could not get constant index");
    print!("{:16} {:04} '", name, constant);
    print_value(self.constants[constant as usize]);
    println!("'");
    return offset + 2;
  }
}
