#include <optional>
#include <vector>
#include <sstream>

#include "disassembler.hpp"
#include "../spec/inst.hpp"

namespace kcpu {

namespace codegen {

bindings::bindings(Word raw, std::optional<Word> constval)
    : load_data(INST_GET_LOADDATA(raw)), opcode(INST_GET_OPCODE(raw)), ius INST_GET_IUS(raw), constval(constval) { }

bound_instruction::bound_instruction(instruction inst, bindings bds)
    : inst(inst), bds(bds) { }

// Dummy instruction, with argument type so that we always report all 3 possible arguments
static instruction unknown_inst("????", opclass_iu3_all(0b111111000), ARGS_3_NOCONST, { });

static std::string format_constval(Word constval) {
    std::stringstream ss;
    ss << "$";
    if(constval < 100) {
        ss << constval;
    } else {
        ss << "0x" << std::hex << constval;
    }
    return ss.str();
}

bound_instruction disassemble_opcode(Word raw, std::optional<Word> constval) {
    bindings b(raw, constval);

    instruction i = ({
        std::optional<instruction> i = arch::self().lookup_inst(b.opcode);
        i ? *i : unknown_inst;
    });

    return bound_instruction(i, b);
}

bound_instruction disassemble_peek(Word rip, mem_bank &bank) {
    Word inst = bank.load(rip);
    std::optional<Word> constval;
    if(INST_GET_LOADDATA(inst)) {
        constval = bank.load(rip + 2);
    }
    return disassemble_opcode(inst, constval);
}

std::string pretty_print(bound_instruction bi) {
    std::stringstream ss;

    ss << bi.inst.name;

    for(int j = 0; j < bi.inst.args.count; j++) {
        ss << " ";

        if(j == bi.inst.args.maybeconst && bi.bds.constval) {
            ss << format_constval(*bi.bds.constval);
        } else {
            ss << PREG_NAMES[bi.bds.ius[j]];
        }
    }

    if(bi.bds.load_data && !bi.bds.constval) {
        ss << "; ?? LOADDATA but no constval";
    }

    if(!bi.bds.load_data && bi.bds.constval) {
        ss << "; ?? NO LOADATA but constval=" << format_constval(*bi.bds.constval);
    }

    return ss.str();
}

}

}