#include <cstdlib>
#include <cstring>
#include <cstdio>

#include "../spec/inst.h"
#include "../spec/ucode.h"
#include "arch.h"

using namespace arch;

static void gen_ctl() {
    // FIXME THESE DON'T WORK BECAUSE THE PUSHED RIP GETS WRONG BY THE TIME WE GET TO THE JMP!
    // I actually think the best thing to do would be to modify the I_STRIP instruction to
    // take an offset for adding before write to the stack, i.e. direct RIP->ALU transfer, then ALU->[RSP]



    // FIXME allow nesting aliases? (thus, PUSH)
    reg_alias(alias("CALL", ARGS_1, {
        unbound_opcode(I_STRIP, { slot_constval(0x0008), slot_reg(REG_SP) }),
        unbound_opcode(I_ADD, { slot_constval(0x0002), slot_reg(REG_SP) }),
        unbound_opcode(I_JMP, { slot_arg(0) }),
    }));

    // This is just a POP RID; JMP RID, but we do it faster
    reg_alias(alias("RET", ARGS_0, {
        unbound_opcode(I_SUB, { slot_constval(0x0002), slot_reg(REG_SP) }),
        unbound_opcode(P_I_LDJMP | I_JMP, { slot_reg(REG_SP) }),
    }));
}

static void gen_mem() {
    // TODO consider a hardware increment on RSP

    reg_alias(alias("PUSH", ARGS_1_NOCONST, {
        unbound_opcode(I_STW, { slot_reg(REG_SP), slot_arg(0) }),
        unbound_opcode(I_ADD, { slot_constval(0x0002), slot_reg(REG_SP) }),
    }));

    reg_alias(alias("POP", ARGS_1_NOCONST, {
        unbound_opcode(I_SUB, { slot_constval(0x0002), slot_reg(REG_SP) }),
        unbound_opcode(I_LDW, { slot_reg(REG_SP), slot_arg(0) }),
    }));
}

static void gen_alu() {
    // XOR with all 1s
    reg_alias(alias("NOT", ARGS_1_NOCONST,
        unbound_opcode(I_XOR, { slot_reg(REG_ONE), slot_arg(0) })));

    // TODO allow nesting aliases? (thus, NOT)
    // Negate then add 1
    reg_alias(alias("NEG", ARGS_1_NOCONST, {
        unbound_opcode(I_XOR, { slot_reg(REG_ONE), slot_arg(0) }),
        unbound_opcode(I_ADD, { slot_constval(0x0001), slot_arg(0) }),
    }));

    // Subtract one operand from the other to perform the comparison.
    // e.g. FLAG_SIGN tells you which is greater.
    reg_alias(alias("CMP", ARGS_2_1CONST, {
        unbound_opcode(I_MOV, { slot_arg(0), slot_reg(REG_ID) }),
        unbound_opcode(I_SUB, { slot_arg(1), slot_reg(REG_ID) }),
    }));
}

void register_aliases() {
    gen_ctl();
    gen_mem();
    gen_alu();
}