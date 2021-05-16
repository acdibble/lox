#include "chunk.h"
#include "common.h"
#include "debug.h"

int main(int argc, const char* argv[]) {
  Chunk chunk;
  initChunk(&chunk);

  // int constant = addConstant(&chunk, 1.2);
  // writeChunk(&chunk, OP_CONSTANT, 123);
  // writeChunk(&chunk, constant, 123);
  for (size_t i = 0, line = 0; i < 300; i++) {
    if (i % 3 == 0) line++;
    writeConstant(&chunk, i, line);
  }

  writeChunk(&chunk, OP_RETURN, 123);

  disassembleChunk(&chunk, "test chunk");
  freeChunk(&chunk);
  return 0;
}
