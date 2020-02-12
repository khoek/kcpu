#ifndef CODEGEN_DISASSEMBLER_H
#define CODEGEN_DISASSEMBLER_H

#include "../lang/arch.hpp"
#include "../vm/mod/mem.hpp"

namespace kcpu {

namespace codegen {

class bindings {
    public:
    bool load_data;
    regval_t opcode;
    preg_t ius[NUM_IUS];
    std::optional<regval_t> constval;

    bindings(regval_t raw, std::optional<regval_t> constval);
};

class bound_instruction {
    public:
    instruction inst;
    bindings bds;

    bound_instruction(instruction inst, bindings bds);
};

bound_instruction disassemble_opcode(regval_t opcode, std::optional<regval_t> constval);
bound_instruction disassemble_peek(regval_t rip, mem_bank &bank);

std::string pretty_print(bound_instruction bi);

}

}

#endif