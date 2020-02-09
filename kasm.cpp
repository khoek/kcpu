#include <iostream>
#include <sstream>
#include <fstream>

#include "src/gen/arch.h"
#include "src/gen/assembler.h"

static void assemble_file(std::istream &in, std::ostream &out) {
    auto ops = assemble(&in);
    out.write((const char *) ops.data(), ops.size() * (sizeof(regval_t) / sizeof(uint8_t)));
}

int main() {
    try {
        init_arch();

        {
            std::ifstream in("asm/bios.kasm");
            std::ofstream out("bios.bin", std::ios::binary);
            assemble_file(in, out);
        }

        {
            std::ifstream in("asm/prog.kasm");
            std::ofstream out("prog.bin", std::ios::binary);
            assemble_file(in, out);
        }
    } catch(std::string msg) {
        std::cerr << msg << '\n';
        return 1;
    } catch(const char * msg) {
        std::cerr << msg << "\n";
        return 1;
    }
}