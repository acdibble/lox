mod chunk;
mod value;

use chunk::{Chunk, Op};

fn main() {
    let mut chunk = Chunk::new();

    let constant = chunk.add_constant(1.2);
    chunk.write(Op::Constant as u8, 123);
    chunk.write(constant, 123);

    chunk.write(Op::Return as u8, 123);
    chunk.disassemble("test chunk")
}
