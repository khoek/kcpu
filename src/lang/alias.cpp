#include "../spec/inst.hpp"
#include "../spec/ucode.hpp"
#include "arch.hpp"

namespace kcpu {

#define reg_alias arch::self().reg_alias

static void gen_ctl() {
    reg_alias(alias("CALL"   , ARGS_1, virtual_instruction(I_X_CALL ,   { slot_reg(REG_SP), slot_arg(0) })));
    reg_alias(alias("RET"    , ARGS_0, virtual_instruction(I_X_RET  ,   { slot_reg(REG_SP) })));
    reg_alias(alias("ENTER"  , ARGS_0, virtual_instruction(I_X_ENTER,   { slot_reg(REG_SP), slot_reg(REG_BP) })));
    reg_alias(alias("LEAVE"  , ARGS_0, virtual_instruction(I_X_LEAVE,   { slot_reg(REG_SP), slot_reg(REG_BP) })));
    reg_alias(alias("ENTERFR", ARGS_1, virtual_instruction(I_X_ENTERFR, { slot_reg(REG_BP), slot_arg(0) })));
}

static void gen_mem() {
    reg_alias(alias("PUSH", ARGS_1        , virtual_instruction(I_X_PUSH, { slot_reg(REG_SP), slot_arg(0) })));
    reg_alias(alias("POP" , ARGS_1_NOCONST, virtual_instruction(I_X_POP , { slot_reg(REG_SP), slot_arg(0) })));

    reg_alias(alias("PUSHFG", ARGS_0, virtual_instruction(I_X_PUSHFG, { slot_reg(REG_SP) })));
    reg_alias(alias("POPFG" , ARGS_0, virtual_instruction(I_X_POPFG , { slot_reg(REG_SP) })));
}

static void gen_alu() {
    // XOR with all 1s
    reg_alias(alias("NOT", ARGS_1_NOCONST,
        virtual_instruction(I_XOR, { slot_reg(REG_ONE), slot_arg(0) })));

    // FIXME just subtract from zero! need a "sub the other way" instruction for this
    // Negate then add 1
    reg_alias(alias("NEG", ARGS_1_NOCONST, {
        virtual_instruction(I_XOR , { slot_reg(REG_ONE), slot_arg(0) }),
        virtual_instruction(I_ADD2, { slot_constval(0x0001), slot_arg(0) }),
    }));

    reg_alias(alias("INC", ARGS_1_NOCONST, {
        virtual_instruction(I_ADD2, { slot_constval(0x0001), slot_arg(0) }),
    }));

    // FIXME use a "nodata" version of an arithmetic instruction instead
    // Subtract one operand from the other to perform the comparison.
    // e.g. FLAG_SIGN tells you which is greater.
    reg_alias(alias("CMP", ARGS_2_1CONST, {
        virtual_instruction(I_MOV, { slot_arg(0), slot_reg(REG_ID) }),
        virtual_instruction(I_SUB, { slot_arg(1), slot_reg(REG_ID) }),
    }));

    reg_alias(alias("JE" , ARGS_1, virtual_instruction(I_JZ,  { slot_arg(0) })));
    reg_alias(alias("JNE", ARGS_1, virtual_instruction(I_JNZ,  { slot_arg(0) })));
    reg_alias(alias("JL" , ARGS_1, virtual_instruction(I_JNC, { slot_arg(0) })));
    reg_alias(alias("JNL", ARGS_1, virtual_instruction(I_JC, { slot_arg(0) })));
    reg_alias(alias("JGE", ARGS_1, virtual_instruction(I_JC, { slot_arg(0) })));

    // TODO check these, I think they are just wrong
    // reg_alias(alias("JLE", ARGS_1, unbound_opcode(I_JS,  { slot_arg(0) })));
    // reg_alias(alias("JG" , ARGS_1, unbound_opcode(I_JNS, { slot_arg(0) })));
}

void internal::register_aliases() {
    gen_ctl();
    gen_mem();
    gen_alu();
}

}