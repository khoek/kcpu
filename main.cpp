#include <cstdlib>
#include <iostream>
#include <cstring>
#include <fstream>
#include <execinfo.h>
#include <signal.h>
#include <unistd.h>

#include "src/gen/arch.h"
#include "src/spec/ucode.h"
#include "src/vm/kcpu.h"

/*void print_bt() {
  void *array[10];
  size_t size;

  // get void*'s for all entries on the stack
  size = backtrace(array, 10);

  // print out all the frames to stderr
  fprintf(stderr, "Error: signal %d:\n", sig);
  backtrace_symbols_fd(array, size, STDERR_FILENO);
  exit(1);
}*/

// regval_t bios_code[] = {
//   // FAR STPFX $(1 << 7)
//   (0b1000010001 << INST_SHIFT) | INST_MK_IU1(REG_ID),
//   (1 << 7),
//   // LDW $0x136 rc
//   (0b1000000011 << INST_SHIFT) | INST_MK_IU1(REG_ID) | INST_MK_IU2(REG_C),
//   0x136,
//   // FAR LDW $0x138 rd
//   (0b1000010011 << INST_SHIFT) | INST_MK_IU1(REG_ID) | INST_MK_IU2(REG_D),
//   0x138,
//   // MOV rc rb
//   (0b0000110000 << INST_SHIFT) | INST_MK_IU1(REG_C ) | INST_MK_IU2(REG_B),
//   // MOV rd ra
//   (0b0000110000 << INST_SHIFT) | INST_MK_IU1(REG_D ) | INST_MK_IU2(REG_A),
//   // MOV $1 rc
//   (0b1000110000 << INST_SHIFT) | INST_MK_IU1(REG_ID) | INST_MK_IU2(REG_C),
//   1,
//   // MOV $2 rd
//   (0b1000110000 << INST_SHIFT) | INST_MK_IU1(REG_ID) | INST_MK_IU2(REG_D),
//   2,
//   // ADD rc rd
//   (0b0001000000 << INST_SHIFT) | INST_MK_IU1(REG_C) | INST_MK_IU2(REG_D),
//   // MOV rd ra
//   (0b0000110000 << INST_SHIFT) | INST_MK_IU1(REG_D) | INST_MK_IU2(REG_A),
//   // XOR rc ra
//   (0b0001000100 << INST_SHIFT) | INST_MK_IU1(REG_C) | INST_MK_IU2(REG_A),
//   // HLT
//   (0b0001111111 << INST_SHIFT)
// };

void load_binary(const char *name, const char *filename, size_t max_len, void *buff) {
  std::ifstream f(filename);  

  f.seekg(0, std::ios::end);  
  size_t len = f.tellg();  
  f.seekg(0, std::ios::beg);

  if(len > max_len) {
    printf("%s binary too long!", name);
  }

  f.read((char *) buff, len);  
  f.close();
}

int main() {
    try {
      init_arch(); // alternatively, load it from somewhere.

      kcpu cpu;
      load_binary("BIOS", "bios.bin", BIOS_SIZE, cpu.mem.bios.raw);
      load_binary("PROG", "prog.bin", PROG_SIZE, cpu.mem.prog.raw);

      printf("CPU Start\n");

      //  while(true) {
      //    cpu.ustep();
      //    fgetc(stdin);
      //  }
      uint32_t tot = cpu.run();

      printf("\nCPU Halted, %d uinstructions executed\n", tot);
    } catch(std::string msg) {
        std::cerr << msg << "\n";
    } catch(const char * msg) {
        std::cerr << msg << "\n";
    }

    return 0;
}
