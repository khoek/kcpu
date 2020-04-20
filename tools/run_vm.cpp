#include <iostream>

#include "lib/binutils.hpp"
#include "src/lang/arch.hpp"
#include "src/spec/ucode.hpp"
#include "src/vm/kcpu.hpp"

int main(int argc, char **argv) {
    bool verbose = false;
    bool disasm_mode = false;
    bool step_mode = false;
    bool headless = false;

    std::vector<std::string> args;
    for(int i = 1; i < argc; i++) {
        std::string arg(argv[i]);
        if(arg == "-v") {
            verbose = true;
        } else if(arg == "-d") {
            disasm_mode = true;
        } else if(arg == "-s") {
            step_mode = true;
            disasm_mode = true;
        } else if(arg == "-h") {
            headless = true;
        } else if(arg == "-nh") {
            headless = false;
        } else {
            args.push_back(argv[i]);
        }
    }

    if(args.size() != 2) {
        std::cerr << "Need two non-switch arguments, the bios and prog bin paths." << std::endl;
        return 1;
    }

    graphics::get_graphics().configure(headless);

    kcpu::vm_logger logger(disasm_mode, verbose, verbose);
    kcpu::vm cpu(logger);
    load_binary("BIOS", args[0], BIOS_SIZE, cpu.mem.bios.data());
    load_binary("PROG", args[1], PROG_SIZE, cpu.mem.prog.data());

    std::cout << "CPU Start" << std::endl;

    do {
        kcpu::vm::state s = step_mode ? cpu.step() : cpu.run();
        if(step_mode && cpu.ctl.cbits[CBIT_INSTMASK]) {
            std::string prompt_msg("[ENTER to step]");
            std::cout << prompt_msg << std::flush;
            char c;
            std::cin >> std::noskipws >> c;
            std::cout << "\r" << std::string(prompt_msg.length(), ' ') << "\r" << std::flush;
        }

        if(s == kcpu::vm::state::ABORTED) {
            std::cout << "CPU Aborted, continue(y)? ";

            char c;
            std::cin >> std::noskipws >> c;
            if(c == 'n' || c == 'N') {
                std::cout << "Stopping..." << std::endl;

                cpu.dump_registers();
                break;
            }

            std::cout << "Continuing..." << std::endl;
            cpu.resume();
        }
    } while(cpu.get_state() == kcpu::vm::state::RUNNING);

    std::cout << std::endl << "CPU " << (cpu.get_state() == kcpu::vm::state::HALTED ? "Halted" : "Aborted")
              << ", " << cpu.get_total_clocks() << " uinstructions executed taking "
              << (cpu.get_real_ns_elapsed() / 1000 / 1000) << "ms" << std::endl;

    return 0;
}
