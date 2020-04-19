#include "../spec/inst.hpp"
#include "../spec/ucode.hpp"
#include "arch.hpp"

namespace kcpu {

#define reg_alias arch::self().reg_alias

static void gen_ctl() {
    // The usual versions of ENTER[FR]/LEAVE which use RBP as the base pointer.
    reg_alias(alias("ENTER0"  , ARGS_0, virtual_instruction(I_ENTER1  , { slot_reg(REG_BP) })));
    reg_alias(alias("ENTERFR1", ARGS_1, virtual_instruction(I_ENTERFR2, { slot_reg(REG_BP), slot_arg(0) })));
    reg_alias(alias("LEAVE0"  , ARGS_0, virtual_instruction(I_LEAVE1  , { slot_reg(REG_BP) })));
}

static void gen_mem() {
    reg_alias(alias("PUSHA" , ARGS_0, {
        /********** Don't forget to save RID! **********/
        virtual_instruction(I_PUSHx2, { slot_reg(REG_ID), slot_reg(REG_A ) }),
        virtual_instruction(I_PUSHx2, { slot_reg(REG_B ), slot_reg(REG_C ) }),
        virtual_instruction(I_PUSHx2, { slot_reg(REG_D ), slot_reg(REG_BP) }),
    }));
    reg_alias(alias("POPA" , ARGS_0, {
        virtual_instruction(I_POPx2 , { slot_reg(REG_BP), slot_reg(REG_D ) }),
        virtual_instruction(I_POPx2 , { slot_reg(REG_C ), slot_reg(REG_B ) }),
        virtual_instruction(I_POPx2 , { slot_reg(REG_A ), slot_reg(REG_ID) }),
        /********** Don't forget to restore RID! **********/
    }));
}

static void gen_alu() {
    // XOR the oprand with 0xFFFF
    reg_alias(alias("NOT", ARGS_1_NOCONST,
        virtual_instruction(I_XOR , { slot_constval(0xFFFF), slot_arg(0) })));

    // Subtract the operand from zero
    reg_alias(alias("NEG", ARGS_1_NOCONST, {
        virtual_instruction(I_BSUB, { slot_constval(0x0000), slot_arg(0) }),
    }));

    // Add one to the operand
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

    reg_alias(alias("LDJE" , ARGS_1, virtual_instruction(I_JZ.add_flag(ITFLAG_JMP_LD),  { slot_arg(0) })));
    reg_alias(alias("LDJNE", ARGS_1, virtual_instruction(I_JNZ.add_flag(ITFLAG_JMP_LD),  { slot_arg(0) })));
    reg_alias(alias("LDJL" , ARGS_1, virtual_instruction(I_JNC.add_flag(ITFLAG_JMP_LD), { slot_arg(0) })));
    reg_alias(alias("LDJNL", ARGS_1, virtual_instruction(I_JC.add_flag(ITFLAG_JMP_LD), { slot_arg(0) })));
    reg_alias(alias("LDJGE", ARGS_1, virtual_instruction(I_JC.add_flag(ITFLAG_JMP_LD), { slot_arg(0) })));

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