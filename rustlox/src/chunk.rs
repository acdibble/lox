use crate::value::*;
use std::convert::TryFrom;
use std::convert::TryInto;
use std::result::Result;

#[repr(u8)]
pub enum Op {
    Constant,
    Nil,
    True,
    False,
    Pop,
    GetLocal,
    SetLocal,
    GetGlobal,
    DefineGlobal,
    SetGlobal,
    Equal,
    Greater,
    Less,
    Add,
    Subtract,
    Multiply,
    Divide,
    Not,
    Negate,
    Print,
    Jump,
    JumpIfFalse,
    Return,
}

impl TryFrom<u8> for Op {
    type Error = u8;

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            x if x == Op::Constant as u8 => Ok(Op::Constant),
            x if x == Op::Nil as u8 => Ok(Op::Nil),
            x if x == Op::True as u8 => Ok(Op::True),
            x if x == Op::False as u8 => Ok(Op::False),
            x if x == Op::Pop as u8 => Ok(Op::Pop),
            x if x == Op::GetLocal as u8 => Ok(Op::GetLocal),
            x if x == Op::SetLocal as u8 => Ok(Op::SetLocal),
            x if x == Op::GetGlobal as u8 => Ok(Op::GetGlobal),
            x if x == Op::DefineGlobal as u8 => Ok(Op::DefineGlobal),
            x if x == Op::SetGlobal as u8 => Ok(Op::SetGlobal),
            x if x == Op::Equal as u8 => Ok(Op::Equal),
            x if x == Op::Greater as u8 => Ok(Op::Greater),
            x if x == Op::Less as u8 => Ok(Op::Less),
            x if x == Op::Add as u8 => Ok(Op::Add),
            x if x == Op::Subtract as u8 => Ok(Op::Subtract),
            x if x == Op::Multiply as u8 => Ok(Op::Multiply),
            x if x == Op::Divide as u8 => Ok(Op::Divide),
            x if x == Op::Not as u8 => Ok(Op::Not),
            x if x == Op::Negate as u8 => Ok(Op::Negate),
            x if x == Op::Print as u8 => Ok(Op::Print),
            x if x == Op::Jump as u8 => Ok(Op::Jump),
            x if x == Op::JumpIfFalse as u8 => Ok(Op::JumpIfFalse),
            x if x == Op::Return as u8 => Ok(Op::Return),
            _ => {
                eprintln!("New case needed in TryFrom<u8>?");
                Err(v)
            }
        }
    }
}

impl TryFrom<&u8> for Op {
    type Error = u8;

    fn try_from(v: &u8) -> Result<Self, Self::Error> {
        Self::try_from(*v)
    }
}

pub struct Chunk {
    pub code: Vec<u8>,
    pub constants: Vec<Value>,
    pub lines: Vec<i32>,
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

    pub fn add_constant(&mut self, value: Value) -> Result<u8, ()> {
        match self.constants.len().try_into() {
            Err(_) => Err(()),
            Ok(index) => {
                self.constants.push(value);
                Ok(index)
            }
        }
    }
}

impl Chunk {
    #![cfg(feature = "trace-execution")]

    pub fn disassemble(&self, name: &'static str) {
        println!("== {} ==", name);

        let mut offset: usize = 0;

        while offset < self.code.len() {
            offset = self.disassemble_instruction(offset)
        }
    }

    pub fn disassemble_instruction(&self, offset: usize) -> usize {
        print!("{:04} ", offset);

        if offset > 0 && self.lines[offset] == self.lines[offset - 1] {
            print!("   | ")
        } else {
            print!("{:4} ", self.lines[offset]);
        }

        let instruction = *self.code.get(offset).expect("Expect instruction");
        match instruction.try_into() {
            Ok(Op::Constant) => self.constant_instruction("OP_CONSTANT", offset),
            Ok(Op::Nil) => self.simple_instruction("OP_NIL", offset),
            Ok(Op::True) => self.simple_instruction("OP_TRUE", offset),
            Ok(Op::False) => self.simple_instruction("OP_FALSE", offset),
            Ok(Op::Pop) => self.simple_instruction("OP_POP", offset),
            Ok(Op::GetLocal) => self.byte_instruction("OP_GET_LOCAL", offset),
            Ok(Op::SetLocal) => self.byte_instruction("OP_SET_LOCAL", offset),
            Ok(Op::GetGlobal) => self.constant_instruction("OP_GET_GLOBAL", offset),
            Ok(Op::DefineGlobal) => self.constant_instruction("OP_DEFINE_GLOBAL", offset),
            Ok(Op::SetGlobal) => self.constant_instruction("OP_SET_GLOBAL", offset),
            Ok(Op::Equal) => self.simple_instruction("OP_EQUAL", offset),
            Ok(Op::Greater) => self.simple_instruction("OP_GREATER", offset),
            Ok(Op::Less) => self.simple_instruction("OP_LESS", offset),
            Ok(Op::Add) => self.simple_instruction("OP_ADD", offset),
            Ok(Op::Subtract) => self.simple_instruction("OP_SUBTRACT", offset),
            Ok(Op::Multiply) => self.simple_instruction("OP_MULTIPLY", offset),
            Ok(Op::Divide) => self.simple_instruction("OP_DIVIDE", offset),
            Ok(Op::Not) => self.simple_instruction("OP_NOT", offset),
            Ok(Op::Negate) => self.simple_instruction("OP_NEGATE", offset),
            Ok(Op::Print) => self.simple_instruction("OP_PRINT", offset),
            Ok(Op::Jump) => self.jump_instruction("OP_JUMP", 1, offset),
            Ok(Op::JumpIfFalse) => self.jump_instruction("OP_JUMP_IF_FALSE", 1, offset),
            Ok(Op::Return) => self.simple_instruction("OP_RETURN", offset),
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
        &self.constants[constant as usize].print();
        println!("'");
        return offset + 2;
    }

    fn byte_instruction(&self, name: &'static str, offset: usize) -> usize {
        let slot = self.code[offset + 1];
        println!("{:16} {:4}", name, slot);
        return offset + 2;
    }

    fn jump_instruction(&self, name: &'static str, sign: i32, offset: usize) -> usize {
        let mut jump: u16 = (self.code[offset + 1] as u16) << 8;
        jump |= self.code[offset + 2] as u16;
        println!(
            "{:16} {:4} -> {}",
            name,
            offset,
            offset as i32 + 3 + sign as i32 * jump as i32
        );
        return offset + 3;
    }
}
