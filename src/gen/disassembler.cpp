#include <optional>
#include <vector>
#include <sstream>

#include "disassembler.h"
#include "../spec/inst.h"

inst_pieces::inst_pieces(regval_t inst, std::optional<regval_t> constval)
    : load_data(INST_GET_LOADDATA(inst)), opcode(INST_GET_OPCODE(inst)), ius INST_GET_IUS(inst), constval(constval) { }

static instruction unknown_inst("????", 0b11111111, ARGS_3_NOCONST, { });

static std::string format_constval(regval_t constval) {
    std::stringstream ss;
    ss << "$";
    if(constval < 100) {
        ss << constval;
    } else {
        ss << "0x" << std::hex << constval;
    }
    return ss.str();
}

std::pair<inst_pieces, std::string> disassemble(regval_t inst, std::optional<regval_t> constval) {
    inst_pieces p(inst, constval);

    std::string pretty;
    std::vector<std::string> comments;

    instruction i = ({
        std::optional<instruction> i = inst_lookup(p.opcode);
        i ? *i : unknown_inst;
    });

    pretty += i.name;

    for(int j = 0; j < i.args.count; j++) {
        pretty += " ";

        if(j == i.args.maybeconst && constval) {
            pretty += format_constval(*constval);
        } else {
            pretty += PREG_NAMES[p.ius[j]];
        }
    }

    if(p.load_data && !constval) {
        comments.push_back("?? LOADDATA but no constval");
    }

    if(!p.load_data && constval) {
        comments.push_back("?? NO LOADATA but constval=" + format_constval(*constval));
    }

    for(int i = 0; i < comments.size(); i++) {
        pretty += "; " + comments[i];
    }

    return std::pair(p, pretty);
}

std::pair<inst_pieces, std::string> disassemble_peek(regval_t rip, mem_bank &bank) {
    regval_t inst = bank.load(rip);
    std::optional<regval_t> constval;
    if(INST_GET_LOADDATA(inst)) {
        constval = bank.load(rip + 2);
    }
    return disassemble(inst, constval);
}