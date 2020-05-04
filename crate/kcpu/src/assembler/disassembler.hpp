#ifndef CODEGEN_DISASSEMBLER_H
#define CODEGEN_DISASSEMBLER_H

#include "../lang/arch.hpp"
#include "../vm/mod/mem.hpp"

namespace kcpu {

namespace codegen {

class bindings {
    public:
    bool load_data;
    RegVal opcode;
    preg_t ius[NUM_IUS];
    std::optional<RegVal> constval;

    bindings(RegVal raw, std::optional<RegVal> constval);
};

class bound_instruction {
    public:
    instruction inst;
    bindings bds;

    bound_instruction(instruction inst, bindings bds);
};

bound_instruction disassemble_opcode(RegVal opcode, std::optional<RegVal> constval);
bound_instruction disassemble_peek(RegVal rip, mem_bank &bank);

std::string pretty_print(bound_instruction bi);

}

}

#endif