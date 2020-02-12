#include "lib/compiler.hpp"
#include "src/lang/arch.hpp"
#include "src/spec/ucode.hpp"
#include "src/vm/kcpu.hpp"

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

    kcpu::vm cpu(kcpu::vm_logger{disasm_mode, verbose, verbose});
    load_binary("BIOS", "bin/bios.bin", BIOS_SIZE, cpu.mem.bios.data());
    load_binary("PROG", "bin/prog.bin", PROG_SIZE, cpu.mem.prog.data());

    printf("CPU Start\n");

    do {
      kcpu::vm::STATE s = step_mode ? cpu.step() : cpu.run();
      if(step_mode && cpu.ctl.cbits[CBIT_INSTMASK]) {
        std::string prompt_msg("[ENTER to step]");
        std::cout << prompt_msg << std::flush;
        char c;
        std::cin >> std::noskipws >> c;
        std::cout << "\r" << std::string(prompt_msg.length(), ' ') << "\r" << std::flush;
      }

      if(s == kcpu::vm::STATE_ABORTED) {
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
    } while(cpu.get_state() == kcpu::vm::STATE_RUNNING);

    printf("CPU %s, %d uinstructions executed\n", cpu.ctl.cbits[CBIT_ABORTED] ? "Aborted" : "Halted", cpu.get_total_clocks());

    return 0;
}
