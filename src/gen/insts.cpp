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

    reg_inst(instruction("NOP", I_NOP, ARGS_0, {
        MCTRL_FIDD_STORE | MCTRL_N_FIDD_OUT | MCTRL_BUSMODE_CONW_BUSM | GCTRL_ACTION_RIP_BUSA_O | GCTRL_FT_ENTER,
                           MCTRL_N_MAIN_OUT | MCTRL_BUSMODE_CONW_BUSB | GCTRL_FT_MAYBEEXIT,
        MCTRL_FIDD_STORE | MCTRL_N_FIDD_OUT | MCTRL_BUSMODE_CONW_BUSM | GCTRL_ACTION_RIP_BUSA_O,
        // NOTE: the busmasking will ensure that IU1 = 0, i.e. REG_ID
                           MCTRL_N_MAIN_OUT | MCTRL_BUSMODE_CONW_BUSB | RCTRL_IU1_BUSB_I | GCTRL_FT_EXIT,
    }));

    reg_inst(instruction("HLT", I_HLT, ARGS_0, {
        MCTRL_N_FIDD_OUT | MCTRL_N_MAIN_OUT | GCTRL_ACTION_HALT | GCTRL_FT_ENTER,
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

    // FIXME Note that right now bad things will happen if the first argument of STRIP is not a constant
    // Probably we should make the assembler prohibit doing anything else.
    reg_inst(mk_distanced_instruction(farbit, "STRIP", I_STRIP, ARGS_2_1CONST, {
        // (1 - 2) Add IU1 to RIP and store in IU1, and (3 - 4) store the result in the address pointed to by IU2
        MCTRL_N_FIDD_OUT | MCTRL_N_MAIN_OUT | ACTRL_INPUT_EN | ACTRL_MODE_ADD  | RCTRL_IU1_BUSA_O | GCTRL_JM_P_RIP_BUSB_O,
        MCTRL_N_FIDD_OUT | MCTRL_N_MAIN_OUT | ACTRL_DATA_OUT | RCTRL_IU1_BUSA_I,
        MCTRL_FIDD_STORE | MCTRL_N_FIDD_OUT | MCTRL_N_MAIN_OUT | MCTRL_BUSMODE_CONW_BUSB | RCTRL_IU1_BUSB_O | RCTRL_IU2_BUSA_O,
        MCTRL_MAIN_STORE                    | MCTRL_N_MAIN_OUT | MCTRL_BUSMODE_CONW_BUSM | GCTRL_FT_ENTER
    }));
    
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
    gen_mem_variants(0b00010000);
}

#define LDJMPPREFIX "LD"

static instruction mk_loadable_instruction(regval_t ldbit, const char * const name, regval_t opcode, argtype args, uinst_t ui) {
    char * buff = (char *) malloc(strlen(FARPREFIX) + strlen(name) + 1);
    sprintf(buff, "%s%s", ldbit ? FARPREFIX : "", name);

    std::vector<uinst_t> uis;
    if(ldbit) {
        uis.push_back(MCTRL_FIDD_STORE | MCTRL_N_FIDD_OUT | MCTRL_BUSMODE_CONW_BUSM | RCTRL_IU1_BUSA_O);
        uis.push_back(MCTRL_N_MAIN_OUT | MCTRL_BUSMODE_CONW_BUSB | ui);
    } else {
        uis.push_back(MCTRL_N_FIDD_OUT | MCTRL_N_MAIN_OUT | ui | RCTRL_IU1_BUSB_O);
    }
    return instruction(buff, opcode | ldbit, args, uis);
}

void gen_ctl_loadables(regval_t ldbit) {
    reg_inst(mk_loadable_instruction(ldbit, "JMP", I_JMP  , ARGS_1,
        GCTRL_JM_YES));

    reg_inst(mk_loadable_instruction(ldbit, "JC" , I_JC   , ARGS_1,
        GCTRL_JM_ON_TRUE  | GCTRL_JCOND_CARRY   ));
    reg_inst(mk_loadable_instruction(ldbit, "JNC", I_JNC  , ARGS_1,
        GCTRL_JM_ON_FALSE | GCTRL_JCOND_CARRY ));

    reg_inst(mk_loadable_instruction(ldbit, "JZ" , I_JZ   , ARGS_1,
        GCTRL_JM_ON_FALSE | GCTRL_JCOND_N_ZERO));
    reg_inst(mk_loadable_instruction(ldbit, "JNZ", I_JNZ  , ARGS_1,
        GCTRL_JM_ON_TRUE  | GCTRL_JCOND_N_ZERO));

    reg_inst(mk_loadable_instruction(ldbit, "JS" , I_JS   , ARGS_1,
        GCTRL_JM_ON_TRUE  | GCTRL_JCOND_SIGN  ));
    reg_inst(mk_loadable_instruction(ldbit, "JNS", I_JNS  , ARGS_1,
        GCTRL_JM_ON_FALSE | GCTRL_JCOND_SIGN  ));

    reg_inst(mk_loadable_instruction(ldbit, "JO" , I_JO   , ARGS_1,
        GCTRL_JM_ON_FALSE | GCTRL_JCOND_N_OVFLW));
    reg_inst(mk_loadable_instruction(ldbit, "JNO", I_JNO  , ARGS_1,
        GCTRL_JM_ON_TRUE  | GCTRL_JCOND_N_OVFLW));
}

void gen_ctl() {
    gen_ctl_loadables(0);
    gen_ctl_loadables(P_I_LDJMP);

    // TODO implement
    // reg_inst("LJMP", 0b00000001, 0, MCTRL_PREFIX_STORE | MCTRL_N_MAIN_OUT | MCTRL_N_FIDD_OUT | RCTRL_IU1_BUSA_O | GCTRL_FT_ENTER);
}

static void gen_reg() {
    //FIXME handle N_XXXXX values....

    reg_inst(instruction("MOV", I_MOV, ARGS_2_1CONST,
        MCTRL_N_FIDD_OUT | MCTRL_N_MAIN_OUT | RCTRL_IU1_BUSA_O | RCTRL_IU2_BUSA_I | GCTRL_FT_ENTER));
}

static instruction mk_arith_inst_arg1(const char *name, regval_t opcode, uinst_t alu_mode, bool const_allowed) {
    return instruction(name, opcode, const_allowed ? ARGS_1 : ARGS_1_NOCONST, {
        MCTRL_N_FIDD_OUT | MCTRL_N_MAIN_OUT | ACTRL_INPUT_EN | alu_mode        | RCTRL_IU1_BUSA_O,
        MCTRL_N_FIDD_OUT | MCTRL_N_MAIN_OUT | ACTRL_DATA_OUT | ACTRL_FLAGS_OUT | RCTRL_IU1_BUSA_I | GCTRL_ACTION_RFG_BUSB_I | GCTRL_FT_ENTER
    });
}

static instruction mk_arith_inst_arg2(const char *name, regval_t opcode, uinst_t alu_mode) {
    return instruction(name, opcode, ARGS_2_1CONST, {
        MCTRL_N_FIDD_OUT | MCTRL_N_MAIN_OUT | ACTRL_INPUT_EN | alu_mode        | RCTRL_IU1_BUSA_O | RCTRL_IU2_BUSB_O,
        MCTRL_N_FIDD_OUT | MCTRL_N_MAIN_OUT | ACTRL_DATA_OUT | ACTRL_FLAGS_OUT | RCTRL_IU2_BUSA_I | GCTRL_ACTION_RFG_BUSB_I | GCTRL_FT_ENTER
    });
}

static void gen_alu() {
    reg_inst(mk_arith_inst_arg2("ADD" , I_ADD , ACTRL_MODE_ADD ));
    reg_inst(mk_arith_inst_arg2("SUB" , I_SUB , ACTRL_MODE_SUB ));
    reg_inst(mk_arith_inst_arg2("AND" , I_AND , ACTRL_MODE_AND ));
    reg_inst(mk_arith_inst_arg2("OR"  , I_OR  , ACTRL_MODE_OR  ));
    reg_inst(mk_arith_inst_arg2("XOR" , I_XOR , ACTRL_MODE_XOR ));
    reg_inst(mk_arith_inst_arg1("LSFT", I_LSFT, ACTRL_MODE_LSFT, false));
    reg_inst(mk_arith_inst_arg1("RSFT", I_RSFT, ACTRL_MODE_RSFT, false));
    reg_inst(mk_arith_inst_arg1("TST" , I_TST , ACTRL_MODE_TST , true ));
}

void register_insts() {
    /* If we want to go to 8~10 general purpose registers, I think we could make do with only 
       7 instruction bits compared to 9. */
       
    gen_sys();
    gen_ctl();
    gen_reg();
    gen_mem();
    gen_alu();
}