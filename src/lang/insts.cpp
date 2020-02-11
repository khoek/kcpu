#include <sstream>

#include "../spec/inst.h"
#include "../spec/ucode.h"
#include "lang.h"

namespace kcpu {

using namespace arch;

static void gen_sys() {
    //FIXME handle N_XXXXX values....

    reg_inst(instruction("NOP" , I_NOP, ARGS_0, {
        MCTRL_FIDD_STORE | MCTRL_N_FIDD_OUT | MCTRL_BUSMODE_CONW_BUSM | GCTRL_ACTION_RIP_BUSA_O | GCTRL_FT_ENTER,
                           MCTRL_N_MAIN_OUT | MCTRL_BUSMODE_CONW_BUSB | GCTRL_FT_MAYBEEXIT,
        MCTRL_FIDD_STORE | MCTRL_N_FIDD_OUT | MCTRL_BUSMODE_CONW_BUSM | GCTRL_ACTION_RIP_BUSA_O,
        // NOTE: the busmasking will ensure that IU1 = 0, i.e. REG_ID
                           MCTRL_N_MAIN_OUT | MCTRL_BUSMODE_CONW_BUSB | RCTRL_IU1_BUSB_I | GCTRL_FT_EXIT,
    }));

    reg_inst(instruction("HLT" , I_HLT, ARGS_0, {
        MCTRL_N_FIDD_OUT | MCTRL_N_MAIN_OUT | GCTRL_ACTION_STOP,
    }));

    reg_inst(instruction("ABRT", I_ABRT, ARGS_0, {
        MCTRL_N_FIDD_OUT | MCTRL_N_MAIN_OUT | GCTRL_ACTION_STOP | GCTRL_FT_ENTER,
    }));
}

#define FARPREFIX "FAR "

static instruction mk_distanced_instruction(regval_t farbit, const char * const name, opclass op, argtype args, std::vector<uinst_t> uis) {
    std::stringstream ss;
    ss << (farbit ? FARPREFIX : "") << name;
    for(auto ui = uis.begin(); ui != uis.end(); ui++) {
	    *ui |= farbit ? MCTRL_USE_PREFIX_FAR : 0;
    }
    return instruction(ss.str(), op.add_flag(farbit), args, uis);
}

static instruction mk_distanced_instruction(regval_t farbit, const char * const name, opclass op, argtype args, uinst_t ui) {
    return mk_distanced_instruction(farbit, name, op, args, std::vector<uinst_t> { ui });
}

static uinst_t ucode_memb_sh_step1(bool is_write, bool lo_or_hi, bool zero) {
    return MCTRL_FIDD_STORE | MCTRL_N_FIDD_OUT | MCTRL_BUSMODE_CONH | (lo_or_hi ? MCTRL_BUSMODE_X : 0) | (is_write ? MCTRL_BUSMODE_WRITE : 0)
            | RCTRL_IU1_BUSA_O | (zero ? 0 /* The bus is pulled low. */ : RCTRL_IU2_BUSB_O);
}

static uinst_t ucode_memb_ld_step2(bool lo_or_hi, bool zero) {
    return MCTRL_N_MAIN_OUT | MCTRL_BUSMODE_CONW_BUSB_MAYBEFLIP | (lo_or_hi ? MCTRL_BUSMODE_X : 0) | RCTRL_IU2_BUSB_I;
}

static uinst_t ucode_memb_st_step2(bool lo_or_hi, bool zero) {
    return MCTRL_N_MAIN_OUT | MCTRL_MAIN_STORE | MCTRL_BUSMODE_CONW_BUSM;
}

static instruction mk_mem_byte_instruction(regval_t farbit, const char * const name, opclass op, argtype args,
    bool is_write, bool lo_or_hi, bool zero) {
    return mk_distanced_instruction(farbit, name, op, args, {
        ucode_memb_sh_step1(is_write, lo_or_hi, zero),
        (is_write ? ucode_memb_st_step2(lo_or_hi, zero) : ucode_memb_ld_step2(lo_or_hi, zero)) | GCTRL_FT_ENTER
    });
}

static void gen_mem_variants(regval_t farbit) {
    reg_inst(mk_distanced_instruction(farbit, "STPFX", I_STPFX, ARGS_1,
        MCTRL_PREFIX_STORE | MCTRL_N_MAIN_OUT | MCTRL_N_FIDD_OUT | RCTRL_IU1_BUSB_O | GCTRL_FT_ENTER));
    
    reg_inst(mk_distanced_instruction(farbit, "LDW", I_LDW, ARGS_2_1CONST, {
        MCTRL_FIDD_STORE | MCTRL_N_FIDD_OUT | MCTRL_BUSMODE_CONW_BUSM | RCTRL_IU1_BUSA_O,
                           MCTRL_N_MAIN_OUT | MCTRL_BUSMODE_CONW_BUSB | RCTRL_IU2_BUSB_I | GCTRL_FT_ENTER
    }));
    
    reg_inst(mk_distanced_instruction(farbit, "LDWO", I_LDWO, ARGS_3_2CONST, {
        MCTRL_N_FIDD_OUT | MCTRL_N_MAIN_OUT | ACTRL_INPUT_EN | ACTRL_MODE_ADD | RCTRL_IU1_BUSA_O | RCTRL_IU2_BUSB_O,
        MCTRL_FIDD_STORE | MCTRL_N_FIDD_OUT | MCTRL_BUSMODE_CONW_BUSM | ACTRL_DATA_OUT,
                           MCTRL_N_MAIN_OUT | MCTRL_BUSMODE_CONW_BUSB | RCTRL_IU3_BUSB_I | GCTRL_FT_ENTER
    }));

    reg_inst(mk_mem_byte_instruction(farbit, "LDBL" , I_LDBL , ARGS_2_1CONST, false, false, false));
    reg_inst(mk_mem_byte_instruction(farbit, "LDBH" , I_LDBH , ARGS_2_1CONST, false, true , false));
    reg_inst(mk_mem_byte_instruction(farbit, "LDBLZ", I_LDBLZ, ARGS_2_1CONST, false, false, true ));
    reg_inst(mk_mem_byte_instruction(farbit, "LDBHZ", I_LDBHZ, ARGS_2_1CONST, false, true , true ));
    
    reg_inst(mk_distanced_instruction(farbit, "STWO", I_STWO, ARGS_3_2CONST, {
        MCTRL_N_FIDD_OUT | MCTRL_N_MAIN_OUT | ACTRL_INPUT_EN | ACTRL_MODE_ADD | RCTRL_IU1_BUSA_O | RCTRL_IU2_BUSB_O,
        MCTRL_FIDD_STORE | MCTRL_N_FIDD_OUT | MCTRL_N_MAIN_OUT | MCTRL_BUSMODE_CONW_BUSB | ACTRL_DATA_OUT | RCTRL_IU3_BUSB_O,
        MCTRL_MAIN_STORE                    | MCTRL_N_MAIN_OUT | MCTRL_BUSMODE_CONW_BUSM | GCTRL_FT_ENTER
    }));

    reg_inst(mk_distanced_instruction(farbit, "STW", I_STW, ARGS_2_1CONST, {
        MCTRL_FIDD_STORE | MCTRL_N_FIDD_OUT | MCTRL_N_MAIN_OUT | MCTRL_BUSMODE_CONW_BUSB | RCTRL_IU1_BUSA_O | RCTRL_IU2_BUSB_O,
        MCTRL_MAIN_STORE                    | MCTRL_N_MAIN_OUT | MCTRL_BUSMODE_CONW_BUSM | GCTRL_FT_ENTER
    }));

    reg_inst(mk_mem_byte_instruction(farbit, "STBL" , I_STBL , ARGS_2_1CONST, true , false, false));
    reg_inst(mk_mem_byte_instruction(farbit, "STBH" , I_STBH , ARGS_2_1CONST, true , true , false));

    // TODO did we ever envision "zero" variants for STBL and STBH?
}

static void gen_mem() {
    gen_mem_variants(0);
    gen_mem_variants(P_I_FAR);
}

#define LDJMPPREFIX "LD"

// `second_arg` means the instruction will take 2 arguments instead of 1, and we will use the value of the second arg
// for the direct jump/load jump.
static instruction mk_loadable_instruction(regval_t ldbit, const char * const name, regval_t opcode,
    bool second_arg, uinst_t jm_w_cond, std::vector<uinst_t> preamble = {}) {
    std::stringstream ss;
    ss << (ldbit ? LDJMPPREFIX : "") << name;

    if(ldbit) {
        preamble.push_back(MCTRL_FIDD_STORE | MCTRL_N_FIDD_OUT | MCTRL_BUSMODE_CONW_BUSM | (second_arg ? RCTRL_IU2_BUSA_O : RCTRL_IU1_BUSA_O));
        preamble.push_back(MCTRL_N_MAIN_OUT | MCTRL_BUSMODE_CONW_BUSB | jm_w_cond);
    } else {
        preamble.push_back(MCTRL_N_FIDD_OUT | MCTRL_N_MAIN_OUT | jm_w_cond | (second_arg ? RCTRL_IU2_BUSB_O : RCTRL_IU1_BUSB_O));
    }
    return instruction(ss.str(), opcode | ldbit, second_arg ? ARGS_2_1CONST : ARGS_1, preamble);
}

static void gen_ctl_loadables(regval_t ldbit) {
    reg_inst(mk_loadable_instruction(ldbit, "JMP" , I_JMP , false, GCTRL_JM_YES));

    reg_inst(mk_loadable_instruction(ldbit, "JC"  , I_JC  , false, GCTRL_JM_ON_TRUE  | GCTRL_JCOND_CARRY  ));
    reg_inst(mk_loadable_instruction(ldbit, "JNC" , I_JNC , false, GCTRL_JM_ON_FALSE | GCTRL_JCOND_CARRY  ));

    reg_inst(mk_loadable_instruction(ldbit, "JZ"  , I_JZ  , false, GCTRL_JM_ON_FALSE | GCTRL_JCOND_N_ZERO ));
    reg_inst(mk_loadable_instruction(ldbit, "JNZ" , I_JNZ , false, GCTRL_JM_ON_TRUE  | GCTRL_JCOND_N_ZERO ));

    reg_inst(mk_loadable_instruction(ldbit, "JS"  , I_JS  , false, GCTRL_JM_ON_TRUE  | GCTRL_JCOND_SIGN   ));
    reg_inst(mk_loadable_instruction(ldbit, "JNS" , I_JNS , false, GCTRL_JM_ON_FALSE | GCTRL_JCOND_SIGN   ));

    reg_inst(mk_loadable_instruction(ldbit, "JO"  , I_JO  , false, GCTRL_JM_ON_FALSE | GCTRL_JCOND_N_OVFLW));
    reg_inst(mk_loadable_instruction(ldbit, "JNO" , I_JNO , false, GCTRL_JM_ON_TRUE  | GCTRL_JCOND_N_OVFLW));

    reg_inst(mk_loadable_instruction(ldbit, "LJMP", I_LJMP, true , GCTRL_JM_YES, {
        MCTRL_PREFIX_STORE | MCTRL_N_FIDD_OUT | MCTRL_N_MAIN_OUT | RCTRL_IU1_BUSB_O,
    }));
}

static void gen_ctl() {
    gen_ctl_loadables(0);
    gen_ctl_loadables(P_I_LDJMP);
}

static void gen_reg() {
    //FIXME handle N_XXXXX values....

    reg_inst(instruction("MOV", I_MOV, ARGS_2_1CONST,
        MCTRL_N_FIDD_OUT | MCTRL_N_MAIN_OUT | RCTRL_IU1_BUSA_O | RCTRL_IU2_BUSA_I | GCTRL_FT_ENTER));
}

#define NOFLAGSUFFIX "NF"

static instruction mk_arith_inst(uinst_t flagbits, const char *name, opclass op, argtype args, uinst_t alu_mode) {
    std::stringstream ss;
    ss << name << (flagbits ? "" : NOFLAGSUFFIX);

    uinst_t tgt, srcs;
    switch(args.count) {
        case 0: throw "zero arg arith instruction";
        case 1: {
            srcs = RCTRL_IU1_BUSA_O;
            tgt = RCTRL_IU1_BUSA_I;
            break;
        }
        case 2: {
            srcs = RCTRL_IU1_BUSA_O | RCTRL_IU2_BUSB_O;
            tgt = RCTRL_IU2_BUSA_I;
            break;
        }
        case 3: {
            srcs = RCTRL_IU1_BUSA_O | RCTRL_IU2_BUSB_O;
            tgt = RCTRL_IU3_BUSA_I;
            break;
        }
        default: throw "too many args!";
    }

    return instruction(ss.str(), op.add_flag(flagbits ? 0 : P_I_NOFGS), args, {
        MCTRL_N_FIDD_OUT | MCTRL_N_MAIN_OUT | ACTRL_INPUT_EN | alu_mode | srcs | RCTRL_IU1_BUSA_O,
        MCTRL_N_FIDD_OUT | MCTRL_N_MAIN_OUT | ACTRL_DATA_OUT | flagbits | tgt  | GCTRL_FT_ENTER,
    });
}

static void gen_alu_flagables(uinst_t flagbits) {
    reg_inst(mk_arith_inst(flagbits, "ADD" , I_ADD , ARGS_2_1CONST , ACTRL_MODE_ADD ));
    reg_inst(mk_arith_inst(flagbits, "SUB" , I_SUB , ARGS_2_1CONST , ACTRL_MODE_SUB ));
    reg_inst(mk_arith_inst(flagbits, "AND" , I_AND , ARGS_2_1CONST , ACTRL_MODE_AND ));
    reg_inst(mk_arith_inst(flagbits, "OR"  , I_OR  , ARGS_2_1CONST , ACTRL_MODE_OR  ));
    reg_inst(mk_arith_inst(flagbits, "XOR" , I_XOR , ARGS_2_1CONST , ACTRL_MODE_XOR ));
    reg_inst(mk_arith_inst(flagbits, "LSFT", I_LSFT, ARGS_1_NOCONST, ACTRL_MODE_LSFT));
    reg_inst(mk_arith_inst(flagbits, "RSFT", I_RSFT, ARGS_1_NOCONST, ACTRL_MODE_RSFT));
    reg_inst(mk_arith_inst(flagbits, "TST" , I_TST , ARGS_1        , ACTRL_MODE_TST ));

    reg_inst(mk_arith_inst(flagbits, "ADD3", I_ADD3, ARGS_3_1CONST , ACTRL_MODE_ADD));
}

static void gen_alu() {
    gen_alu_flagables(0);
    gen_alu_flagables(ACTRL_FLAGS_OUT | GCTRL_ACTION_RFG_BUSB_I);
}

static void gen_x() {
    // FIXME as below, args should be RSP, RBP

    // Faster version of: PUSH rbp; MOV rsp rbp;, i.e. (X_PUSH rsp rbp; MOV rsp rbp;)
    reg_inst(instruction("X_ENTER", I_X_ENTER, ARGS_2_NOCONST, {
                           MCTRL_N_FIDD_OUT | MCTRL_N_MAIN_OUT | RCTRL_RSP_DEC,
        MCTRL_FIDD_STORE | MCTRL_N_FIDD_OUT | MCTRL_N_MAIN_OUT | MCTRL_BUSMODE_CONW_BUSB | RCTRL_IU1_BUSA_O | RCTRL_IU2_BUSB_O,
        MCTRL_MAIN_STORE                    | MCTRL_N_MAIN_OUT | MCTRL_BUSMODE_CONW_BUSM | RCTRL_IU1_BUSA_O | RCTRL_IU2_BUSA_I | GCTRL_FT_ENTER,
    }));

    // IU1 must be RBP, IU2 = $CONST or reg, IU3 is forced to RSP
    // Faster version of: PUSH rbp; MOV rsp rbp; SUBNF $CONST, rsp;
    // FIXME if we move to non-hardcoded IU3s, then change this to 3 args.
    reg_inst(instruction("X_ENTERFR", I_X_ENTERFR, ARGS_2_2CONST, {
        //PUSH rbp; MOV rsp rbp;
                           MCTRL_N_FIDD_OUT | MCTRL_N_MAIN_OUT | RCTRL_RSP_DEC,
        MCTRL_FIDD_STORE | MCTRL_N_FIDD_OUT | MCTRL_N_MAIN_OUT | MCTRL_BUSMODE_CONW_BUSB | RCTRL_IU3_BUSA_O | RCTRL_IU1_BUSB_O,
        MCTRL_MAIN_STORE                    | MCTRL_N_MAIN_OUT | MCTRL_BUSMODE_CONW_BUSM | RCTRL_IU3_BUSB_O | RCTRL_IU1_BUSB_I
        //SUBNF $CONST, rsp
                | RCTRL_IU2_BUSA_O | ACTRL_INPUT_EN | ACTRL_MODE_SUB,
        MCTRL_N_FIDD_OUT | MCTRL_N_MAIN_OUT | ACTRL_DATA_OUT | RCTRL_IU3_BUSA_I | GCTRL_FT_ENTER,
    }));

    // Faster version of: MOV rbp rsp; POP rbp, i.e. (MOV rbp rsp; X_POP rsp rbp;)
    // instead we do them both simultaneously.
    reg_inst(instruction("X_LEAVE", I_X_LEAVE, ARGS_2_NOCONST, {
        MCTRL_FIDD_STORE | MCTRL_N_FIDD_OUT |                    MCTRL_BUSMODE_CONW_BUSM | RCTRL_IU1_BUSA_I | RCTRL_IU2_BUSA_O,
                                              MCTRL_N_MAIN_OUT | MCTRL_BUSMODE_CONW_BUSB | RCTRL_IU2_BUSB_I | RCTRL_RSP_INC | GCTRL_FT_ENTER,
    }));

    // FIXME Note that right now bad things will happen if the first argument of these is not RSP
    // Probably we should make the assembler prohibit doing anything else.

    reg_inst(instruction("X_PUSH", I_X_PUSH, ARGS_2_2CONST, {
        //IU1 = MUST BE RSP, IU2 = REG to PUSH
                           MCTRL_N_FIDD_OUT | MCTRL_N_MAIN_OUT | RCTRL_RSP_DEC,
        MCTRL_FIDD_STORE | MCTRL_N_FIDD_OUT | MCTRL_N_MAIN_OUT | MCTRL_BUSMODE_CONW_BUSB | RCTRL_IU1_BUSA_O | RCTRL_IU2_BUSB_O,
        MCTRL_MAIN_STORE                    | MCTRL_N_MAIN_OUT | MCTRL_BUSMODE_CONW_BUSM | GCTRL_FT_ENTER,
    }));

    reg_inst(instruction("X_POP", I_X_POP, ARGS_2_NOCONST, {
        //IU1 = MUST BE RSP, IU2 = REG to POP
        MCTRL_FIDD_STORE | MCTRL_N_FIDD_OUT |                    MCTRL_BUSMODE_CONW_BUSM | RCTRL_IU1_BUSA_O,
                                              MCTRL_N_MAIN_OUT | MCTRL_BUSMODE_CONW_BUSB | RCTRL_IU2_BUSB_I | RCTRL_RSP_INC | GCTRL_FT_ENTER,
    }));

    reg_inst(instruction("X_CALL", I_X_CALL, ARGS_2_2CONST, {
        // IU1 = MUST BE RSP, IU2 = CALL ADDRESS
        // Effectively: X_PUSH RSP RIP
                           MCTRL_N_FIDD_OUT | MCTRL_N_MAIN_OUT | RCTRL_RSP_DEC,
        MCTRL_FIDD_STORE | MCTRL_N_FIDD_OUT | MCTRL_N_MAIN_OUT | MCTRL_BUSMODE_CONW_BUSB | RCTRL_IU1_BUSA_O | GCTRL_JM_P_RIP_BUSB_O,
        MCTRL_MAIN_STORE                    | MCTRL_N_MAIN_OUT | MCTRL_BUSMODE_CONW_BUSM | RCTRL_IU2_BUSB_O | GCTRL_JM_YES,
    }));

    reg_inst(instruction("X_RET", I_X_RET, ARGS_1_NOCONST, {
        // IU1 = MUST BE RSP, IU2 = CALL ADDRESS
        // Effectively: X_POP RSP RIP          
        MCTRL_FIDD_STORE | MCTRL_N_FIDD_OUT |                    MCTRL_BUSMODE_CONW_BUSM | RCTRL_IU1_BUSA_O,
                                              MCTRL_N_MAIN_OUT | MCTRL_BUSMODE_CONW_BUSB | RCTRL_RSP_INC | GCTRL_JM_YES,
    }));
}

void register_insts() {
    /* If we want to go to 8~10 general purpose registers, I think we could make do with only 
       7 instruction bits compared to 9. */

    // TODO none of our instructions have more than 4 microcode instructions! Trim the "dead zone".
       
    gen_sys();
    gen_ctl();
    gen_reg();
    gen_mem();
    gen_alu();
    gen_x();
}

}