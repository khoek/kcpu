#include <cstdlib>
#include <iostream>
#include <cstring>
#include <fstream>
#include <execinfo.h>
#include <signal.h>
#include <unistd.h>

#include "lib/compiler.h"
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

int main() {
    printf("There are %u control bits.\n", UCODE_END);

    try {
      init_arch(); // alternatively, load the microcode from somewhere.

      kcpu cpu;
      load_binary("BIOS", "bin/bios.bin", BIOS_SIZE, cpu.mem.bios.raw);
      load_binary("PROG", "bin/prog.bin", PROG_SIZE, cpu.mem.prog.raw);

      printf("CPU Start\n");

      // while(true) {
      //    cpu.ustep();
      //    fgetc(stdin);
      // }
      
      do {
        kcpu::STATE s = cpu.run();
        if(s == kcpu::STATE_ABORTED) {
          printf("\nCPU Aborted, continue(y)? ");

          char c;
          std::cin >> std::noskipws >> c;
          if(c == 'n' || c == 'N') {
            printf("\nStopping...");
            break;
          }

          printf("\nContinuing...");
          cpu.resume();
        }
      } while(cpu.get_state() == kcpu::STATE_RUNNING);

      printf("\nCPU %s, %d uinstructions executed\n", cpu.ctl.cbits[CBIT_ABORTED] ? "Aborted" : "Halted", cpu.get_total_clocks());
    } catch(std::string msg) {
        std::cerr << msg << "\n";
    } catch(const char * msg) {
        std::cerr << msg << "\n";
    }

    return 0;
}
