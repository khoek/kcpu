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

int main(int argc, char **argv) {
    std::cout << "There are " << UCODE_END << " control bits." << std::endl;

    bool verbose = false;
    bool disasm_mode = false;
    bool step_mode = false;

    for(int i = 1; i < argc; i++) {
      std::string arg(argv[i]);
      if(arg == "-v") {
        verbose = true;
      } else if(arg == "-d") {
        disasm_mode = true;
      } else if(arg == "-s") {
        step_mode = true;
        disasm_mode = true;
      } else {
        std::cerr << "Unknown cmdline option: " << arg << std::endl;
        return 1;
      }
    }

    try {
      init_arch(); // alternatively, load the microcode from somewhere.

      kcpu cpu(vm_logger{disasm_mode, verbose, verbose});
      load_binary("BIOS", "bin/bios.bin", BIOS_SIZE, cpu.mem.bios.raw);
      load_binary("PROG", "bin/prog.bin", PROG_SIZE, cpu.mem.prog.raw);

      printf("CPU Start\n");
      
      do {
        kcpu::STATE s = step_mode ? cpu.step() : cpu.run();
        if(step_mode && cpu.ctl.cbits[CBIT_INSTMASK]) {
          std::string prompt_msg("[ENTER to step]");
          std::cout << prompt_msg << std::flush;
          char c;
          std::cin >> std::noskipws >> c;
          std::cout << "\r" << std::string(prompt_msg.length(), ' ') << "\r" << std::flush;
        }

        if(s == kcpu::STATE_ABORTED) {
          printf("CPU Aborted, continue(y)? ");

          char c;
          std::cin >> std::noskipws >> c;
          if(c == 'n' || c == 'N') {
            printf("Stopping...\n");
            break;
          }

          printf("Continuing...\n");
          cpu.resume();
        }
      } while(cpu.get_state() == kcpu::STATE_RUNNING);

      printf("CPU %s, %d uinstructions executed\n", cpu.ctl.cbits[CBIT_ABORTED] ? "Aborted" : "Halted", cpu.get_total_clocks());
    } catch(std::string msg) {
        std::cerr << msg << "\n";
    } catch(const char * msg) {
        std::cerr << msg << "\n";
    }

    return 0;
}
