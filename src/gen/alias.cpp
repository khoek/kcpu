#include <cstdlib>
#include <cstring>
#include <cstdio>

#include "../spec/inst.h"
#include "../spec/ucode.h"
#include "arch.h"

using namespace arch;

static void gen_ctl() {
    reg_alias(alias("CALL", ARGS_1, {
        unbound_opcode(I_X_CALL, { slot_reg(REG_SP), slot_arg(0) }),
    }));

    // This is just a POP RID; JMP RID, but we do it faster
    reg_alias(alias("RET", ARGS_0, {
        unbound_opcode(P_I_NOFGS | I_SUB, { slot_constval(0x0002), slot_reg(REG_SP) }),
        unbound_opcode(P_I_LDJMP | I_JMP, { slot_reg(REG_SP) }),
    }));
}

static void gen_mem() {
    reg_alias(alias("PUSH", ARGS_1, {
        unbound_opcode(I_X_PUSH, { slot_reg(REG_SP), slot_arg(0) }),
    }));

    reg_alias(alias("POP", ARGS_1_NOCONST, {
        unbound_opcode(P_I_NOFGS | I_SUB, { slot_constval(0x0002), slot_reg(REG_SP) }),
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

    reg_alias(alias("JE" , ARGS_1, unbound_opcode(I_JZ,  { slot_arg(0) })));
    reg_alias(alias("JGE", ARGS_1, unbound_opcode(I_JC,  { slot_arg(0) })));
    reg_alias(alias("JL" , ARGS_1, unbound_opcode(I_JNC, { slot_arg(0) })));
    reg_alias(alias("JLE", ARGS_1, unbound_opcode(I_JS,  { slot_arg(0) })));
    reg_alias(alias("JG" , ARGS_1, unbound_opcode(I_JNS, { slot_arg(0) })));
}

void register_aliases() {
    gen_ctl();
    gen_mem();
    gen_alu();
}