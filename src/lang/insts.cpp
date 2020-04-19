#include <sstream>

#include "../spec/inst.hpp"
#include "../spec/ucode.hpp"
#include "arch.hpp"

namespace kcpu {

#define reg_inst arch::self().reg_inst

// Note that active low bits stored in MASK_I_INVERT are toggled during registration,
// so we can treat them here as if they were active high---but we still use the xxx_N_xxxx
// notation to name them (despite them having an active high meaning here).

static void gen_sys() {
    reg_inst(instruction("NOP" , I_NOP, ARGS_0, {
        MCTRL_MODE_FI_MO | MCTRL_BUSMODE_CONW_BUSM | ACTION_GCTRL_RIP_BUSA_O | GCTRL_FT_ENTER,
        MCTRL_MODE_FO    | MCTRL_BUSMODE_CONW_BUSB | GCTRL_FT_MAYBEEXIT,
        MCTRL_MODE_FI_MO | MCTRL_BUSMODE_CONW_BUSM | ACTION_GCTRL_RIP_BUSA_O,
        // NOTE: the busmasking will ensure that IU1 = 0, i.e. REG_ID
        MCTRL_MODE_FO    | MCTRL_BUSMODE_CONW_BUSB | RCTRL_IU1_BUSB_I | GCTRL_FT_EXIT,
    }));

    // FIXME create an ARGS_XXXX const which represents that this instruction should never
    // be used in code?
    reg_inst(instruction("_DO_INT", I__DO_INT, ARGS_0, {
        // Effectively: CALL IHPR [don't load next inst yet]; PUSHFG
        MCTRL_MODE_FI    | MCTRL_BUSMODE_CONW_BUSB | RCTRL_IU3_BUSA_O | GCTRL_JM_P_RIP_BUSB_O | COMMAND_RCTRL_RSP_EARLY_DEC_IU3RSP,
        // NOTE There are a lot of bits here; we have to load the new RIP, but not jump, and decrement RSP.
        MCTRL_MODE_FO_MI | MCTRL_BUSMODE_CONW_BUSM | ACTION_GCTRL_USE_ALT | GCTRL_ALT_CREG_IHPR | GCTRL_CREG_O | GCTRL_JM_YES | COMMAND_INHIBIT_JMFT,
        // NOTE We need a second RSP decrement to have happened here
        MCTRL_MODE_FI    | MCTRL_BUSMODE_CONW_BUSB | RCTRL_IU3_BUSA_O | ACTION_GCTRL_USE_ALT | GCTRL_ALT_CREG_FG | GCTRL_CREG_O | COMMAND_RCTRL_RSP_EARLY_DEC_IU3RSP,
        MCTRL_MODE_FO_MI | MCTRL_BUSMODE_CONW_BUSM | GCTRL_FT_ENTER,
    }));

    reg_inst(instruction("HLT" , I_HLT , ARGS_0, GCTRL_JM_HALT));
    reg_inst(instruction("ABRT", I_ABRT, ARGS_0, GCTRL_JM_ABRT));
}

#define FARPREFIX "FAR "

static instruction mk_distanced_instruction(bool far, regval_t farbit, const char * const name, opclass op, argtype args, std::vector<uinst_t> uis) {
    std::stringstream ss;
    ss << (far ? FARPREFIX : "") << name;
    for(auto ui = uis.begin(); ui != uis.end(); ui++) {
	    *ui |= far ? 0 : MCTRL_FLAG_MODE_N_FAR;
    }
    return instruction(ss.str(), op.add_flag(far ? farbit : 0), args, uis);
}

static instruction mk_distanced_instruction(bool far, regval_t farbit, const char * const name, opclass op, argtype args, uinst_t ui) {
    return mk_distanced_instruction(far, farbit, name, op, args, std::vector<uinst_t> { ui });
}

static uinst_t ucode_memb_sh_step1(bool is_write, bool lo_or_hi, bool zero) {
    return MCTRL_MODE_FI_MO | MCTRL_BUSMODE_CONH | (lo_or_hi ? ACTION_MCTRL_BUSMODE_X : 0) | (is_write ? MCTRL_BUSMODE_WRITE : 0)
            | RCTRL_IU1_BUSA_O | (zero ? 0 /* The bus is pulled low. */ : RCTRL_IU2_BUSB_O);
}

static uinst_t ucode_memb_ld_step2(bool lo_or_hi, bool zero) {
    return MCTRL_MODE_FO | MCTRL_BUSMODE_CONW_BUSB_MAYBEFLIP | (lo_or_hi ? ACTION_MCTRL_BUSMODE_X : 0) | RCTRL_IU2_BUSB_I;
}

static uinst_t ucode_memb_st_step2(bool lo_or_hi, bool zero) {
    return MCTRL_MODE_FO_MI | MCTRL_BUSMODE_CONW_BUSM;
}

static instruction mk_mem_byte_instruction(bool far, regval_t farbit, const char * const name, opclass op, argtype args,
    bool is_write, bool lo_or_hi, bool zero) {
    return mk_distanced_instruction(far, farbit, name, op, args, {
        ucode_memb_sh_step1(is_write, lo_or_hi, zero),
        (is_write ? ucode_memb_st_step2(lo_or_hi, zero) : ucode_memb_ld_step2(lo_or_hi, zero)) | GCTRL_FT_ENTER,
    });
}

static void gen_mem_variants(bool far) {
    reg_inst(mk_distanced_instruction(far, ITFLAG_MEM_FAR, "LDW", I_LDW, ARGS_2_1CONST, {
        MCTRL_MODE_FI_MO | MCTRL_BUSMODE_CONW_BUSM | RCTRL_IU1_BUSA_O,
        MCTRL_MODE_FO    | MCTRL_BUSMODE_CONW_BUSB | RCTRL_IU2_BUSB_I | GCTRL_FT_ENTER,
    }));

    reg_inst(mk_mem_byte_instruction(far, ITFLAG_MEM_FAR, "LDBL" , I_LDBL , ARGS_2_1CONST, false, false, false));
    reg_inst(mk_mem_byte_instruction(far, ITFLAG_MEM_FAR, "LDBH" , I_LDBH , ARGS_2_1CONST, false, true , false));
    reg_inst(mk_mem_byte_instruction(far, ITFLAG_MEM_FAR, "LDBLZ", I_LDBLZ, ARGS_2_1CONST, false, false, true ));
    reg_inst(mk_mem_byte_instruction(far, ITFLAG_MEM_FAR, "LDBHZ", I_LDBHZ, ARGS_2_1CONST, false, true , true ));

    reg_inst(mk_distanced_instruction(far, ITFLAG_MEM_FAR, "STW", I_STW, ARGS_2_1CONST, {
        MCTRL_MODE_FI    | MCTRL_BUSMODE_CONW_BUSB | RCTRL_IU1_BUSA_O | RCTRL_IU2_BUSB_O,
        MCTRL_MODE_FO_MI | MCTRL_BUSMODE_CONW_BUSM | GCTRL_FT_ENTER,
    }));

    reg_inst(mk_mem_byte_instruction(far, ITFLAG_MEM_FAR, "STBL" , I_STBL , ARGS_2_1CONST, true , false, false));
    reg_inst(mk_mem_byte_instruction(far, ITFLAG_MEM_FAR, "STBH" , I_STBH , ARGS_2_1CONST, true , true , false));

    reg_inst(mk_distanced_instruction(far, ICFLAG_MEM_IU3_FAR, "LDWO", I_LDWO, ARGS_3_2CONST, {
        ACTRL_INPUT_EN | ACTRL_MODE_ADD | RCTRL_IU1_BUSA_O | RCTRL_IU2_BUSB_O,
        MCTRL_MODE_FI_MO | MCTRL_BUSMODE_CONW_BUSM | ACTRL_DATA_OUT,
        MCTRL_MODE_FO    | MCTRL_BUSMODE_CONW_BUSB | RCTRL_IU3_BUSB_I | GCTRL_FT_ENTER,
    }));

    reg_inst(mk_distanced_instruction(far, ICFLAG_MEM_IU3_FAR, "STWO", I_STWO, ARGS_3_2CONST, {
        ACTRL_INPUT_EN | ACTRL_MODE_ADD | RCTRL_IU1_BUSA_O | RCTRL_IU2_BUSB_O,
        MCTRL_MODE_FI    | MCTRL_BUSMODE_CONW_BUSB | ACTRL_DATA_OUT | RCTRL_IU3_BUSB_O,
        MCTRL_MODE_FO_MI | MCTRL_BUSMODE_CONW_BUSM | GCTRL_FT_ENTER,
    }));

    // TODO did we ever envision "zero" variants for STBL and STBH?
}

static void gen_mem() {
    gen_mem_variants(false);
    gen_mem_variants(true);

    // The "far" selection for STPFX is a bit hacky, and works differently to normal mem IO
    reg_inst(instruction("STPFX", I_STPFX, ARGS_1,
        MCTRL_MODE_STPFX | MCTRL_BUSMODE_CONW_BUSB | RCTRL_IU1_BUSB_O | GCTRL_FT_ENTER));
    reg_inst(instruction("FAR STPFX", I_STPFX.add_flag(ITFLAG_MEM_FAR), ARGS_1,
        MCTRL_MODE_STPFX_FAR | MCTRL_BUSMODE_CONW_BUSB | RCTRL_IU1_BUSB_O | GCTRL_FT_ENTER));
}

#define LDJMPPREFIX "LD"

// `second_arg` means the instruction will take 2 arguments instead of 1, and we will use the value of the second arg
// for the direct jump/load jump.
static instruction mk_loadable_instruction(regval_t ld, regval_t ldbit, const char * const name, opclass op,
    bool second_arg, uinst_t jm_w_cond, std::vector<uinst_t> preamble = {}) {
    std::stringstream ss;
    ss << (ld ? LDJMPPREFIX : "") << name;

    if(ld) {
        preamble.push_back(MCTRL_MODE_FI_MO | MCTRL_BUSMODE_CONW_BUSM | (second_arg ? RCTRL_IU2_BUSA_O : RCTRL_IU1_BUSA_O));
        preamble.push_back(MCTRL_MODE_FO    | MCTRL_BUSMODE_CONW_BUSB | jm_w_cond);
    } else {
        preamble.push_back(jm_w_cond | (second_arg ? RCTRL_IU2_BUSB_O : RCTRL_IU1_BUSB_O));
    }
    return instruction(ss.str(), op.add_flag(ld ? ldbit : 0), second_arg ? ARGS_2_1CONST : ARGS_1, preamble);
}

static void gen_jmp_loadables(bool ld) {
    reg_inst(mk_loadable_instruction(ld, ITFLAG_JMP_LD, "JMP"   , I_JMP   , false, GCTRL_JM_YES));

    reg_inst(mk_loadable_instruction(ld, ITFLAG_JMP_LD, "JMP+DI", I_JMP_DI, false, ACTION_GCTRL_USE_ALT | GCTRL_ALT_P_IE | GCTRL_CREG_O | GCTRL_JM_YES));
    reg_inst(mk_loadable_instruction(ld, ITFLAG_JMP_LD, "JMP+EI", I_JMP_EI, false, ACTION_GCTRL_USE_ALT | GCTRL_ALT_P_IE | GCTRL_CREG_I | GCTRL_JM_YES));

    reg_inst(mk_loadable_instruction(ld, ITFLAG_JMP_LD, "JC"    , I_JC    , false,                       GCTRL_JCOND_CARRY  ));
    reg_inst(mk_loadable_instruction(ld, ITFLAG_JMP_LD, "JNC"   , I_JNC   , false, GCTRL_JM_INVERTCOND | GCTRL_JCOND_CARRY  ));

    reg_inst(mk_loadable_instruction(ld, ITFLAG_JMP_LD, "JZ"    , I_JZ    , false, GCTRL_JM_INVERTCOND | GCTRL_JCOND_N_ZERO ));
    reg_inst(mk_loadable_instruction(ld, ITFLAG_JMP_LD, "JNZ"   , I_JNZ   , false,                       GCTRL_JCOND_N_ZERO ));

    reg_inst(mk_loadable_instruction(ld, ITFLAG_JMP_LD, "JS"    , I_JS    , false,                       GCTRL_JCOND_SIGN   ));
    reg_inst(mk_loadable_instruction(ld, ITFLAG_JMP_LD, "JNS"   , I_JNS   , false, GCTRL_JM_INVERTCOND | GCTRL_JCOND_SIGN   ));

    reg_inst(mk_loadable_instruction(ld, ITFLAG_JMP_LD, "JO"    , I_JO    , false, GCTRL_JM_INVERTCOND | GCTRL_JCOND_N_OVFLW));
    reg_inst(mk_loadable_instruction(ld, ITFLAG_JMP_LD, "JNO"   , I_JNO   , false,                       GCTRL_JCOND_N_OVFLW));

    reg_inst(mk_loadable_instruction(ld, ITFLAG_JMP_LD, "LJMP", I_LJMP, true , GCTRL_JM_YES,
        { MCTRL_MODE_STPFX | MCTRL_BUSMODE_CONW_BUSB | RCTRL_IU1_BUSB_O, }));
}

static void gen_jmp() {
    gen_jmp_loadables(false);
    gen_jmp_loadables(true);
}

static void gen_reg() {
    reg_inst(instruction("MOV", I_MOV, ARGS_2_1CONST, RCTRL_IU1_BUSA_O | RCTRL_IU2_BUSA_I | GCTRL_FT_ENTER));
}

static void gen_ctl() {
    reg_inst(instruction("LFG"  , I_LFG , ARGS_1, RCTRL_IU1_BUSB_O | ACTION_GCTRL_USE_ALT | GCTRL_ALT_CREG_FG   | GCTRL_CREG_I | GCTRL_FT_ENTER));
    reg_inst(instruction("LIHP" , I_LIHP, ARGS_1, RCTRL_IU1_BUSB_O | ACTION_GCTRL_USE_ALT | GCTRL_ALT_CREG_IHPR | GCTRL_CREG_I | GCTRL_FT_ENTER));

    reg_inst(instruction("DI", I_DI  , ARGS_0,                    ACTION_GCTRL_USE_ALT | GCTRL_ALT_P_IE | GCTRL_CREG_O | GCTRL_FT_ENTER));
    reg_inst(instruction("EI", I_EI  , ARGS_0,                    ACTION_GCTRL_USE_ALT | GCTRL_ALT_P_IE | GCTRL_CREG_I | GCTRL_FT_ENTER));
}

#define NOFLAGSUFFIX "NF"

static instruction mk_alu_inst(const char *name, opclass op, argtype args, uinst_t alu_mode, bool backward, regval_t oc_flag, const char *suffix, uinst_t out_mode) {
    if(backward && args.count != 2) {
        throw arch_error("can only reverse 2 args");
    }

    uinst_t srcs;
    uinst_t tgt;
    switch(args.count) {
        case 0: throw arch_error("zero arg arith instruction");
        case 1: {
            srcs = RCTRL_IU1_BUSA_O;
            tgt = RCTRL_IU1_BUSA_I;
            break;
        }
        case 2: {
            if(backward) {
                srcs = RCTRL_IU2_BUSA_O | RCTRL_IU1_BUSB_O;
                tgt = RCTRL_IU2_BUSA_I;
            } else {
                srcs = RCTRL_IU1_BUSA_O | RCTRL_IU2_BUSB_O;
                tgt = RCTRL_IU2_BUSA_I;
            }
            break;
        }
        case 3: {
            srcs = RCTRL_IU1_BUSA_O | RCTRL_IU2_BUSB_O;
            tgt = RCTRL_IU3_BUSA_I;
            break;
        }
        default: throw arch_error("too many args!");
    }

    std::stringstream ss;
    ss << name << suffix;

    return instruction(ss.str(), op.add_flag(oc_flag), args, {
        ACTRL_INPUT_EN | alu_mode | srcs | RCTRL_IU1_BUSA_O,
                         out_mode | ((out_mode & ACTRL_DATA_OUT) ? tgt : 0) | GCTRL_FT_ENTER,
    });
}

static void gen_alu_variant_group(const char *suffix, regval_t oc_flag, uinst_t out_mode) {
    reg_inst(mk_alu_inst("ADD2", I_ADD2, ARGS_2_1CONST , ACTRL_MODE_ADD , false, oc_flag, suffix, out_mode)); // c.f. the ADD3 variant
    reg_inst(mk_alu_inst("SUB" , I_SUB , ARGS_2_1CONST , ACTRL_MODE_SUB , false, oc_flag, suffix, out_mode));
    reg_inst(mk_alu_inst("BSUB", I_BSUB, ARGS_2_1CONST , ACTRL_MODE_SUB , true , oc_flag, suffix, out_mode));
    reg_inst(mk_alu_inst("AND" , I_AND , ARGS_2_1CONST , ACTRL_MODE_AND , false, oc_flag, suffix, out_mode));
    reg_inst(mk_alu_inst("OR"  , I_OR  , ARGS_2_1CONST , ACTRL_MODE_OR  , false, oc_flag, suffix, out_mode));
    reg_inst(mk_alu_inst("XOR" , I_XOR , ARGS_2_1CONST , ACTRL_MODE_XOR , false, oc_flag, suffix, out_mode));
    reg_inst(mk_alu_inst("LSFT", I_LSFT, ARGS_1_NOCONST, ACTRL_MODE_LSFT, false, oc_flag, suffix, out_mode));
    reg_inst(mk_alu_inst("RSFT", I_RSFT, ARGS_1_NOCONST, ACTRL_MODE_RSFT, false, oc_flag, suffix, out_mode));
}

#define ALU_OUTMODE_NORMAL    (ACTRL_DATA_OUT | ACTRL_FLAGS_OUT | ACTION_GCTRL_USE_ALT | GCTRL_ALT_P_O_CHNMI_OR_I_ALUFG | GCTRL_CREG_I)
#define ALU_OUTMODE_NOFLAGS   (ACTRL_DATA_OUT)
#define ALU_OUTMODE_FLAGSONLY (                 ACTRL_FLAGS_OUT | ACTION_GCTRL_USE_ALT | GCTRL_ALT_P_O_CHNMI_OR_I_ALUFG | GCTRL_CREG_I)

static void gen_alu() {
    gen_alu_variant_group(""  ,                 0, ALU_OUTMODE_NORMAL );
    gen_alu_variant_group("NF", ICFLAG_ALU1_NOFGS, ALU_OUTMODE_NOFLAGS);

    // The ADD3[NF] instructions live in a different ITYPE to those above, so we won't reuse ICFLAG_ALU1_NOFGS
    // in order to pass between the variants (currently, this works by coincidence).
    reg_inst(mk_alu_inst("ADD3"  , I_ADD3  , ARGS_3_1CONST, ACTRL_MODE_ADD, false, 0, "", ALU_OUTMODE_NORMAL));
    reg_inst(mk_alu_inst("ADD3NF", I_ADD3NF, ARGS_3_1CONST, ACTRL_MODE_ADD, false, 0, "", ALU_OUTMODE_NOFLAGS));

    // The TST instruction has no NOFLAGS (NF) variant, since then it would have no effect.
    reg_inst(mk_alu_inst("TST", I_TST, ARGS_1, ACTRL_MODE_TST, false, 0, "", ALU_OUTMODE_NORMAL));

    // Subtract one operand from the other to perform the comparison.
    // e.g. FLAG_SIGN tells you which is greater.
    reg_inst(mk_alu_inst("CMP", I_CMP, ARGS_2_1CONST, ACTRL_MODE_SUB, true, 0, "", ALU_OUTMODE_FLAGSONLY));
}

static void gen_stk() {
    // Faster version of: PUSH rbp; MOV rsp rbp;, i.e. (PUSH rbp; MOV rsp rbp;)
    reg_inst(instruction("ENTER1", I_ENTER1, ARGS_1_NOCONST, {
        // IU1 = MUST BE RBP
        MCTRL_MODE_FI    | MCTRL_BUSMODE_CONW_BUSB | RCTRL_IU3_BUSA_O | GCTRL_NRM_IU3_OVERRIDE_O_SELECT_RSP_I__UNUSED | RCTRL_IU1_BUSB_O | COMMAND_RCTRL_RSP_EARLY_DEC_IU3RSP,
        MCTRL_MODE_FO_MI | MCTRL_BUSMODE_CONW_BUSM | RCTRL_IU3_BUSA_O | GCTRL_NRM_IU3_OVERRIDE_O_SELECT_RSP_I__UNUSED | RCTRL_IU1_BUSA_I | GCTRL_FT_ENTER,
    }));

    // IU1 must be RBP, IU2 = $CONST or reg, IU3 is forced to RSP
    // Faster version of: PUSH rbp; MOV rsp rbp; SUBNF $CONST, rsp;
    reg_inst(instruction("ENTERFR2", I_ENTERFR2, ARGS_2_2CONST, {
        // IU1 = MUST BE RBP
        // PUSH rbp; MOV rsp rbp;
        MCTRL_MODE_FI    | MCTRL_BUSMODE_CONW_BUSB | RCTRL_IU3_BUSA_O | RCTRL_IU1_BUSB_O | COMMAND_RCTRL_RSP_EARLY_DEC_IU3RSP,
        MCTRL_MODE_FO_MI | MCTRL_BUSMODE_CONW_BUSM | RCTRL_IU3_BUSB_O | GCTRL_NRM_IU3_OVERRIDE_O_SELECT_RSP_I__UNUSED | RCTRL_IU1_BUSB_I
        // SUBNF $CONST, rsp
                | RCTRL_IU2_BUSA_O | ACTRL_INPUT_EN | ACTRL_MODE_SUB,
        ACTRL_DATA_OUT | RCTRL_IU3_BUSA_I | GCTRL_NRM_IU3_OVERRIDE_O_SELECT_RSP_I__UNUSED | GCTRL_FT_ENTER,
    }));

    // Faster version of: MOV rbp rsp; POP rbp, i.e. (MOV rbp rsp; POP rbp;)
    // instead we do them both simultaneously.
    reg_inst(instruction("LEAVE1", I_LEAVE1, ARGS_1_NOCONST, {
        // IU1 = MUST BE RBP
        MCTRL_MODE_FI_MO | MCTRL_BUSMODE_CONW_BUSM | RCTRL_IU3_BUSA_I | GCTRL_NRM_IU3_OVERRIDE_O_SELECT_RSP_I__UNUSED | RCTRL_IU1_BUSA_O | COMMAND_RCTRL_RSP_EARLY_DEC_IU3RSP,
        MCTRL_MODE_FO    | MCTRL_BUSMODE_CONW_BUSB | RCTRL_IU1_BUSB_I | COMMAND_RCTRL_RSP_EARLY_INC | GCTRL_FT_ENTER,
    }));

    // NOTE `PUSH %rsp` writes the NEW %rsp to the NEW address. (This happens to be the old 8086 behaviour, but not 286 and beyond.)
    reg_inst(instruction("PUSH", I_PUSH, ARGS_1, {
        MCTRL_MODE_FI    | MCTRL_BUSMODE_CONW_BUSB | RCTRL_IU3_BUSA_O | GCTRL_NRM_IU3_OVERRIDE_O_SELECT_RSP_I__UNUSED | RCTRL_IU1_BUSB_O | COMMAND_RCTRL_RSP_EARLY_DEC_IU3RSP,
        MCTRL_MODE_FO_MI | MCTRL_BUSMODE_CONW_BUSM | GCTRL_FT_ENTER,
    }));

    // NOTE `POP %rsp` writes the OLD TOP OF STACK to the NEW %rsp (unchanged).
    reg_inst(instruction("POP", I_POP, ARGS_1_NOCONST, {
        MCTRL_MODE_FI_MO | MCTRL_BUSMODE_CONW_BUSM | RCTRL_IU3_BUSA_O | GCTRL_NRM_IU3_OVERRIDE_O_SELECT_RSP_I__UNUSED,
        MCTRL_MODE_FO    | MCTRL_BUSMODE_CONW_BUSB | RCTRL_IU1_BUSB_I | COMMAND_RCTRL_RSP_EARLY_INC | GCTRL_FT_ENTER,
    }));

    reg_inst(instruction("PUSHFG", I_PUSHFG, ARGS_0, {
        MCTRL_MODE_FI    | MCTRL_BUSMODE_CONW_BUSB | RCTRL_IU3_BUSA_O | ACTION_GCTRL_USE_ALT | GCTRL_ALT_CREG_FG | GCTRL_CREG_O | COMMAND_RCTRL_RSP_EARLY_DEC_IU3RSP,
        MCTRL_MODE_FO_MI | MCTRL_BUSMODE_CONW_BUSM | GCTRL_FT_ENTER,
    }));

    reg_inst(instruction("POPFG", I_POPFG, ARGS_0, {
        MCTRL_MODE_FI_MO | MCTRL_BUSMODE_CONW_BUSM | RCTRL_IU3_BUSA_O | GCTRL_NRM_IU3_OVERRIDE_O_SELECT_RSP_I__UNUSED,
        MCTRL_MODE_FO    | MCTRL_BUSMODE_CONW_BUSB | ACTION_GCTRL_USE_ALT | GCTRL_ALT_CREG_FG | GCTRL_CREG_I | COMMAND_RCTRL_RSP_EARLY_INC | GCTRL_FT_ENTER,
    }));

    // Effectively `PUSH RIP`
    reg_inst(instruction("CALL", I_CALL, ARGS_1, {
        MCTRL_MODE_FI    | MCTRL_BUSMODE_CONW_BUSB | RCTRL_IU3_BUSA_O | GCTRL_NRM_IU3_OVERRIDE_O_SELECT_RSP_I__UNUSED | GCTRL_JM_P_RIP_BUSB_O | COMMAND_RCTRL_RSP_EARLY_DEC_IU3RSP,
        MCTRL_MODE_FO_MI | MCTRL_BUSMODE_CONW_BUSM | RCTRL_IU1_BUSB_O | GCTRL_JM_YES,
    }));

    // Effectively `POP RIP`
    reg_inst(instruction("RET", I_RET, ARGS_0, {
        MCTRL_MODE_FI_MO | MCTRL_BUSMODE_CONW_BUSM | RCTRL_IU3_BUSA_O | GCTRL_NRM_IU3_OVERRIDE_O_SELECT_RSP_I__UNUSED,
        MCTRL_MODE_FO    | MCTRL_BUSMODE_CONW_BUSB | COMMAND_RCTRL_RSP_EARLY_INC | GCTRL_JM_YES,
    }));

    // Effectively `POPFG; RET [+ clear CBHIT_HNMI]`
    reg_inst(instruction("IRET", I_IRET, ARGS_0, {
        MCTRL_MODE_FI_MO | MCTRL_BUSMODE_CONW_BUSM | RCTRL_IU3_BUSA_O | GCTRL_NRM_IU3_OVERRIDE_O_SELECT_RSP_I__UNUSED,
        MCTRL_MODE_FO    | MCTRL_BUSMODE_CONW_BUSB | COMMAND_RCTRL_RSP_EARLY_INC | ACTION_GCTRL_USE_ALT | GCTRL_ALT_CREG_FG              | GCTRL_CREG_I,
        MCTRL_MODE_FI_MO | MCTRL_BUSMODE_CONW_BUSM | RCTRL_IU3_BUSA_O | GCTRL_NRM_IU3_OVERRIDE_O_SELECT_RSP_I__UNUSED,
        MCTRL_MODE_FO    | MCTRL_BUSMODE_CONW_BUSB | COMMAND_RCTRL_RSP_EARLY_INC | ACTION_GCTRL_USE_ALT | GCTRL_ALT_P_O_CHNMI_OR_I_ALUFG | GCTRL_CREG_O | GCTRL_JM_YES,
    }));
}

static void gen_io() {
    reg_inst(instruction("IOR", I_IOR, ARGS_2_1CONST, RCTRL_IU1_BUSA_O | RCTRL_IU2_BUSB_I | GCTRL_NRM_IO_READWRITE | GCTRL_CREG_I | GCTRL_FT_ENTER));
    reg_inst(instruction("IOW", I_IOW, ARGS_2_1CONST, RCTRL_IU1_BUSA_O | RCTRL_IU2_BUSB_O | GCTRL_NRM_IO_READWRITE | GCTRL_CREG_O | GCTRL_FT_ENTER));
}

static void gen_optimizations() {
    reg_inst(instruction("PUSHx2", I_PUSHx2, ARGS_2_1CONST, {
        MCTRL_MODE_FI    | MCTRL_BUSMODE_CONW_BUSB | RCTRL_IU3_BUSA_O | GCTRL_NRM_IU3_OVERRIDE_O_SELECT_RSP_I__UNUSED | RCTRL_IU1_BUSB_O | COMMAND_RCTRL_RSP_EARLY_DEC_IU3RSP,
        MCTRL_MODE_FO_MI | MCTRL_BUSMODE_CONW_BUSM,
        MCTRL_MODE_FI    | MCTRL_BUSMODE_CONW_BUSB | RCTRL_IU3_BUSA_O | GCTRL_NRM_IU3_OVERRIDE_O_SELECT_RSP_I__UNUSED | RCTRL_IU2_BUSB_O | COMMAND_RCTRL_RSP_EARLY_DEC_IU3RSP,
        MCTRL_MODE_FO_MI | MCTRL_BUSMODE_CONW_BUSM | GCTRL_FT_ENTER,
    }));

    reg_inst(instruction("POPx2", I_POPx2, ARGS_2_NOCONST, {
        MCTRL_MODE_FI_MO | MCTRL_BUSMODE_CONW_BUSM | RCTRL_IU3_BUSA_O | GCTRL_NRM_IU3_OVERRIDE_O_SELECT_RSP_I__UNUSED,
        MCTRL_MODE_FO    | MCTRL_BUSMODE_CONW_BUSB | RCTRL_IU1_BUSB_I | COMMAND_RCTRL_RSP_EARLY_INC,
        MCTRL_MODE_FI_MO | MCTRL_BUSMODE_CONW_BUSM | RCTRL_IU3_BUSA_O | GCTRL_NRM_IU3_OVERRIDE_O_SELECT_RSP_I__UNUSED,
        MCTRL_MODE_FO    | MCTRL_BUSMODE_CONW_BUSB | RCTRL_IU2_BUSB_I | COMMAND_RCTRL_RSP_EARLY_INC | GCTRL_FT_ENTER,
    }));
}

void kcpu::internal::register_insts() {
    /* If we want to go to 8~10 general purpose registers, I think we could make do with only
       7 instruction bits compared to 9. */

    gen_sys();
    gen_jmp();
    gen_reg();
    gen_ctl();
    gen_mem();
    gen_alu();
    gen_io();
    gen_stk();
    gen_optimizations();
}

}