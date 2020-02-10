#include <cstdlib>
#include <cstring>
#include <cstdio>

#include "../spec/inst.h"
#include "../spec/ucode.h"
#include "arch.h"

using namespace arch;

static uinst_t ucode_memb_sh_step1(bool is_write, bool lo_or_hi, bool zero) {
    return MCTRL_FIDD_STORE | MCTRL_N_FIDD_OUT | MCTRL_BUSMODE_CONH | (lo_or_hi ? MCTRL_BUSMODE_X : 0) | (is_write ? MCTRL_BUSMODE_WRITE : 0)
            | RCTRL_IU1_BUSA_O | (zero ? 0 /* The bus is pulled low. */ : RCTRL_IU2_BUSB_O);
}

static uinst_t ucode_memb_ld_step2(bool lo_or_hi, bool zero) {
    return MCTRL_N_MAIN_OUT | MCTRL_BUSMODE_CONW_BUSB_MAYBEFLIP | (lo_or_hi ? MCTRL_BUSMODE_X : 0)
            | RCTRL_IU2_BUSB_I;
}

static uinst_t ucode_memb_st_step2(bool lo_or_hi, bool zero) {
    return MCTRL_MAIN_STORE | MCTRL_N_MAIN_OUT | MCTRL_BUSMODE_CONW_BUSM;
}

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

static instruction mk_distanced_instruction(regval_t farbit, const char * const name, regval_t opcode, argtype args, std::vector<uinst_t> uis) {
    char * buff = (char *) malloc(strlen(FARPREFIX) + strlen(name) + 1);
    sprintf(buff, "%s%s", farbit ? FARPREFIX : "", name);
    for(auto ui = uis.begin(); ui != uis.end(); ui++) {
	    *ui |= farbit ? MCTRL_USE_PREFIX_FAR : 0;
    }
    return instruction(buff, opcode | farbit, args, uis);
}

static instruction mk_distanced_instruction(regval_t farbit, const char * const name, regval_t opcode, argtype args, uinst_t ui) {
    return mk_distanced_instruction(farbit, name, opcode, args, std::vector<uinst_t> { ui });
}

static void gen_mem_variants(regval_t farbit) {
    reg_inst(mk_distanced_instruction(farbit, "STPFX", I_STPFX, ARGS_1,
        MCTRL_PREFIX_STORE | MCTRL_N_MAIN_OUT | MCTRL_N_FIDD_OUT | RCTRL_IU1_BUSB_O | GCTRL_FT_ENTER));
    
    reg_inst(mk_distanced_instruction(farbit, "LDW", I_LDW, ARGS_2_1CONST, {
        MCTRL_FIDD_STORE | MCTRL_N_FIDD_OUT | MCTRL_BUSMODE_CONW_BUSM | RCTRL_IU1_BUSA_O,
                           MCTRL_N_MAIN_OUT | MCTRL_BUSMODE_CONW_BUSB | RCTRL_IU2_BUSB_I | GCTRL_FT_ENTER
    }));

    reg_inst(mk_distanced_instruction(farbit, "LDBL", I_LDBL, ARGS_2_1CONST, {
        ucode_memb_sh_step1(false, false, false),
        ucode_memb_ld_step2(       false, false) | GCTRL_FT_ENTER
    }));
    
    reg_inst(mk_distanced_instruction(farbit, "LDBH", I_LDBH, ARGS_2_1CONST, {
        ucode_memb_sh_step1(false, true , false),
        ucode_memb_ld_step2(       true , false) | GCTRL_FT_ENTER
    }));
    
    reg_inst(mk_distanced_instruction(farbit, "LDBLZ", I_LDBLZ, ARGS_2_1CONST, {
        ucode_memb_sh_step1(false, false, true ),
        ucode_memb_ld_step2(       false, true ) | GCTRL_FT_ENTER
    }));
    
    reg_inst(mk_distanced_instruction(farbit, "LDBHZ", I_LDBHZ, ARGS_2_1CONST, {
        ucode_memb_sh_step1(false, true , true ),
        ucode_memb_ld_step2(       true , true ) | GCTRL_FT_ENTER
    }));

    reg_inst(mk_distanced_instruction(farbit, "STW", I_STW, ARGS_2_1CONST, {
        MCTRL_FIDD_STORE | MCTRL_N_FIDD_OUT | MCTRL_N_MAIN_OUT | MCTRL_BUSMODE_CONW_BUSB | RCTRL_IU1_BUSA_O | RCTRL_IU2_BUSB_O,
        MCTRL_MAIN_STORE                    | MCTRL_N_MAIN_OUT | MCTRL_BUSMODE_CONW_BUSM | GCTRL_FT_ENTER
    }));

    reg_inst(mk_distanced_instruction(farbit, "STBL", I_STBL, ARGS_2_1CONST, {
        ucode_memb_sh_step1(true , false, false),
        ucode_memb_st_step2(       false, false) | GCTRL_FT_ENTER
    }));
    
    reg_inst(mk_distanced_instruction(farbit, "STBH", I_STBH, ARGS_2_1CONST, {
        ucode_memb_sh_step1(true , true , false),
        ucode_memb_st_step2(       true , false) | GCTRL_FT_ENTER
    }));
    
    reg_inst(mk_distanced_instruction(farbit, "STBLZ", I_STBLZ, ARGS_2_1CONST, {
        ucode_memb_sh_step1(true , false, true ),
        ucode_memb_st_step2(       false, true ) | GCTRL_FT_ENTER
    }));
    
    reg_inst(mk_distanced_instruction(farbit, "STBHZ", I_STBHZ, ARGS_2_1CONST, {
        ucode_memb_sh_step1(true , true , true ),
        ucode_memb_st_step2(       true , true ) | GCTRL_FT_ENTER
    }));
}

static void gen_mem() {
    gen_mem_variants(0);
    gen_mem_variants(P_I_FAR);
}

#define LDJMPPREFIX "LD"

// `second_arg` means the instruction will take 2 arguments instead of 1, and we will use the value of the second arg
// for the direct jump/load jump.
static instruction mk_loadable_instruction(regval_t ldbit, const char * const name, regval_t opcode,\
    bool second_arg, uinst_t jm_w_cond, std::vector<uinst_t> preamble) {
    char * buff = (char *) malloc(strlen(LDJMPPREFIX) + strlen(name) + 1);
    sprintf(buff, "%s%s", ldbit ? LDJMPPREFIX : "", name);

    if(ldbit) {
        preamble.push_back(MCTRL_FIDD_STORE | MCTRL_N_FIDD_OUT | MCTRL_BUSMODE_CONW_BUSM | (second_arg ? RCTRL_IU2_BUSA_O : RCTRL_IU1_BUSA_O));
        preamble.push_back(MCTRL_N_MAIN_OUT | MCTRL_BUSMODE_CONW_BUSB | jm_w_cond);
    } else {
        preamble.push_back(MCTRL_N_FIDD_OUT | MCTRL_N_MAIN_OUT | jm_w_cond | (second_arg ? RCTRL_IU2_BUSB_O : RCTRL_IU1_BUSB_O));
    }
    return instruction(buff, opcode | ldbit, second_arg ? ARGS_2_1CONST : ARGS_1, preamble);
}

static instruction mk_loadable_instruction(regval_t ldbit, const char * const name, regval_t opcode,
    bool second_arg, uinst_t jm_w_cond) {
    return mk_loadable_instruction(ldbit, name, opcode, second_arg, jm_w_cond, {});
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

static instruction mk_arith_inst_arg1(uinst_t flagbits, const char *name, regval_t opcode, uinst_t alu_mode, bool const_allowed) {
    char * buff = (char *) malloc(strlen(NOFLAGSUFFIX) + strlen(name) + 1);
    sprintf(buff, "%s%s", name, flagbits ? "" : NOFLAGSUFFIX);

    return instruction(buff, opcode | (flagbits ? 0 : P_I_NOFGS), const_allowed ? ARGS_1 : ARGS_1_NOCONST, {
        MCTRL_N_FIDD_OUT | MCTRL_N_MAIN_OUT | ACTRL_INPUT_EN | alu_mode | RCTRL_IU1_BUSA_O,
        MCTRL_N_FIDD_OUT | MCTRL_N_MAIN_OUT | ACTRL_DATA_OUT | flagbits | RCTRL_IU1_BUSA_I | GCTRL_FT_ENTER
    });
}

static instruction mk_arith_inst_arg2(uinst_t flagbits, const char *name, regval_t opcode, uinst_t alu_mode) {
    char * buff = (char *) malloc(strlen(NOFLAGSUFFIX) + strlen(name) + 1);
    sprintf(buff, "%s%s", name, flagbits ? "" : NOFLAGSUFFIX);

    return instruction(buff, opcode | (flagbits ? 0 : P_I_NOFGS), ARGS_2_1CONST, {
        MCTRL_N_FIDD_OUT | MCTRL_N_MAIN_OUT | ACTRL_INPUT_EN | alu_mode | RCTRL_IU1_BUSA_O | RCTRL_IU2_BUSB_O,
        MCTRL_N_FIDD_OUT | MCTRL_N_MAIN_OUT | ACTRL_DATA_OUT | flagbits | RCTRL_IU2_BUSA_I | GCTRL_FT_ENTER
    });
}

static void gen_alu_flagables(uinst_t flagbits) {
    reg_inst(mk_arith_inst_arg2(flagbits, "ADD" , I_ADD , ACTRL_MODE_ADD ));
    reg_inst(mk_arith_inst_arg2(flagbits, "SUB" , I_SUB , ACTRL_MODE_SUB ));
    reg_inst(mk_arith_inst_arg2(flagbits, "AND" , I_AND , ACTRL_MODE_AND ));
    reg_inst(mk_arith_inst_arg2(flagbits, "OR"  , I_OR  , ACTRL_MODE_OR  ));
    reg_inst(mk_arith_inst_arg2(flagbits, "XOR" , I_XOR , ACTRL_MODE_XOR ));
    reg_inst(mk_arith_inst_arg1(flagbits, "LSFT", I_LSFT, ACTRL_MODE_LSFT, false));
    reg_inst(mk_arith_inst_arg1(flagbits, "RSFT", I_RSFT, ACTRL_MODE_RSFT, false));
    reg_inst(mk_arith_inst_arg1(flagbits, "TST" , I_TST , ACTRL_MODE_TST , true ));
}

static void gen_alu() {
    gen_alu_flagables(0);
    gen_alu_flagables(ACTRL_FLAGS_OUT | GCTRL_ACTION_RFG_BUSB_I);
}

static void gen_x() {
    // FIXME Note that right now bad things will happen if the first argument of these is not RSP
    // Probably we should make the assembler prohibit doing anything else.

    reg_inst(instruction("X_PUSH", I_X_PUSH, ARGS_2_2CONST, {
        //IU1 = MUST BE RSP, IU2 = REG to PUSH
        MCTRL_FIDD_STORE | MCTRL_N_FIDD_OUT | MCTRL_N_MAIN_OUT | MCTRL_BUSMODE_CONW_BUSB | RCTRL_IU1_BUSA_O | RCTRL_IU2_BUSB_O,
        MCTRL_MAIN_STORE                    | MCTRL_N_MAIN_OUT | MCTRL_BUSMODE_CONW_BUSM | RCTRL_RSP_INC | GCTRL_FT_ENTER
    }));

    reg_inst(instruction("X_CALL", I_X_CALL, ARGS_2_2CONST, {
        // IU1 = MUST BE RSP, IU2 = CALL ADDRESS
        // Effectively: PUSH RIP RSP | JMP IU2
        MCTRL_FIDD_STORE | MCTRL_N_FIDD_OUT | MCTRL_N_MAIN_OUT | MCTRL_BUSMODE_CONW_BUSB | RCTRL_IU1_BUSA_O | GCTRL_JM_P_RIP_BUSB_O,
        MCTRL_MAIN_STORE                    | MCTRL_N_MAIN_OUT | MCTRL_BUSMODE_CONW_BUSM | RCTRL_IU2_BUSB_O | RCTRL_RSP_INC | GCTRL_JM_YES
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