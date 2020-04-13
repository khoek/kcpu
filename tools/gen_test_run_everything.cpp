#include <cassert>
#include <fstream>
#include <algorithm>

#include "src/lang/arch.hpp"
#include "src/spec/hw.hpp"

int main() {
    std::vector<kcpu::family> families = kcpu::arch::self().list_families();
    std::vector<kcpu::alias> aliases = kcpu::arch::self().list_aliases();
    std::vector<kcpu::instruction> insts = kcpu::arch::self().list_insts();

    sort(families.begin(), families.end(), [](const auto &a, const auto &b) { return a.name < b.name; });
    sort(aliases.begin(), aliases.end(), [](const auto &a, const auto &b) { return a.name < b.name; });
    sort(insts.begin(), insts.end(), [](const auto &a, const auto &b) { return a.name < b.name; });

    std::ofstream of_families("bin/families.txt");
    for(auto f : families) {
        of_families << f.name << std::endl;
    }

    std::ofstream of_aliases("bin/aliases.txt");
    for(auto a : aliases) {
        of_aliases << a.name << std::endl;
    }

    std::ofstream of_insts("bin/insts.txt");
    for(auto i : insts) {
        of_insts << i.name << std::endl;
    }

    std::vector<std::string> blacklist = {
        "HLT", "ABRT",
        "STPFX", "FAR STPFX",
        "IOR", "IOW",
        "INT1", "INT2",
        "CALL", "RET", "X_CALL", "X_RET",
        "JMP", "LJMP",
        "LDJMP", "LDLJMP",
        "JC", "JO", "JS", "JZ", "JE", "JL", "JGE",
        "JNC", "JNO", "JNS", "JNZ", "JNE", "JNL",
        "LDJC", "LDJO", "LDJS", "LDJZ", "LDJE", "LDJL", "LDJGE",
        "LDJNC", "LDJNO", "LDJNS", "LDJNZ", "LDJNE", "LDJNL",
    };

    std::ofstream of_everything("test/run_everything/prog.kasm");
    of_everything << "# run_everything (mostly, except jumps)" << std::endl;
    of_everything << "STPFX $0x0080" << std::endl;
    of_everything << "FAR STPFX $0x0080" << std::endl;
    of_everything << std::endl;
    for(auto a : aliases) {
        if(std::find(blacklist.begin(), blacklist.end(), a.name) != blacklist.end()) {
            continue;
        }

        of_everything << a.name;
        for(int j = 0; j < a.args.count; j++) {
            assert(a.args.count < 8 - 4);
            of_everything << " %r" << kcpu::PREG_NAMES[j + 4];
        }
        of_everything << std::endl;
    }
    of_everything << "HLT" << std::endl;
    of_everything << "ABRT" << std::endl;
}
