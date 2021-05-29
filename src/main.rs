mod chunk;
mod value;
mod vm;

use chunk::*;
use vm::*;

fn main() {
    let mut chunk = Chunk::new();

    let constant = chunk.add_constant(1.2);
    chunk.write(Op::Constant as u8, 123);
    chunk.write(constant, 123);

    let constant = chunk.add_constant(3.4);
    chunk.write(Op::Constant as u8, 123);
    chunk.write(constant, 123);

    chunk.write(Op::Add as u8, 123);

    let constant = chunk.add_constant(5.6);
    chunk.write(Op::Constant as u8, 123);
    chunk.write(constant, 123);

    chunk.write(Op::Divide as u8, 123);
    chunk.write(Op::Return as u8, 123);
    chunk.disassemble("test chunk");

    let mut vm = VM::new();

    vm.interpret(&chunk);
}
