#ifndef GEN_DISASSEMBLER_H
#define GEN_DISASSEMBLER_H

#include "../lang/arch.hpp"
#include "../vm/mod/mem.hpp"

namespace kcpu {

class inst_pieces {
    public:
    bool load_data;
    regval_t opcode;
    preg_t ius[NUM_IUS];
    std::optional<regval_t> constval;

    inst_pieces(regval_t inst, std::optional<regval_t> constval);
};

std::pair<inst_pieces, std::string> disassemble(regval_t inst, std::optional<regval_t> constval);
std::pair<inst_pieces, std::string> disassemble_peek(regval_t rip, mem_bank &bank);

}

#endif