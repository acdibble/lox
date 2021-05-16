#include "chunk.h"
#include "common.h"
#include "debug.h"
#include "vm.h"

int main(int argc, const char* argv[]) {
  initVM();

  Chunk chunk;
  initChunk(&chunk);

  writeConstant(&chunk, 4, 1);
  for (size_t i = 0; i < 100'000'000; i++) {
    writeChunk(&chunk, OP_NEGATE, 1);
  }

  writeChunk(&chunk, OP_RETURN, 1);

  // disassembleChunk(&chunk, "test chunk");
  interpret(&chunk);
  freeVM();
  freeChunk(&chunk);
  return 0;
}
