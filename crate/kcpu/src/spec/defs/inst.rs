// RUSTFIX split the files in this module, and the `defs` in the `asm` module, into `std`?

use super::opclass::*;
use super::usig::*;
use crate::spec::{
    types::{
        hw::{UInst, Word},
        schema::{ArgKind, ConstPolicy, Half, InstDef, OpClass},
    },
    ucode::Builder,
};

// RUSTFIX Come up with a respresentation of e.g. SRC SRC DST operand codes, giving widths only,
// which will translate into ArgKind lists. TBH, we could probably just change ArgKinds into SRC and DST. Are there any cases of 2 args, no const?
// Make the syntax so it defaults to word, and then is special for a byte. This will make everything WAY better. :)

// RUSTIFIX find a way to get rid of all of the `to_owned()`s

// Note that active low bits stored in MASK_I_INVERT are toggled during registration,
// so we can treat them here as if they were active high---but we still use the xxx_N_xxxx
// notation to name them (despite them having an active high meaning here).

fn gen_sys(builder: &mut Builder) {
    builder.register(InstDef::with_0(
        "NOP",
        I_NOP,
        vec![
            MCTRL_MODE_FI_MO | MCTRL_BUSMODE_CONW_BUSM | ACTION_GCTRL_RIP_BUSA_O | GCTRL_FT_ENTER,
            MCTRL_MODE_FO | MCTRL_BUSMODE_CONW_BUSB | GCTRL_FT_MAYBEEXIT,
            MCTRL_MODE_FI_MO | MCTRL_BUSMODE_CONW_BUSM | ACTION_GCTRL_RIP_BUSA_O,
            // NOTE: the busmasking will ensure that IU1 = 0, i.e. REG_ID
            MCTRL_MODE_FO | MCTRL_BUSMODE_CONW_BUSB | RCTRL_IU1_BUSB_I | GCTRL_FT_EXIT,
        ],
    ));

    // FIXME create an ARGS_XXXX const which represents that this instruction should never
    // be used in code?
    builder.register(InstDef::with_0(
        "_DO_INT",
        I__DO_INT,
        vec![
            // Effectively: CALL IHPR [don't load next inst yet]; PUSHFG
            MCTRL_MODE_FI
                | MCTRL_BUSMODE_CONW_BUSB
                | RCTRL_IU3_BUSA_O
                | GCTRL_JM_P_RIP_BUSB_O
                | COMMAND_RCTRL_RSP_EARLY_DEC_IU3RSP,
            // NOTE There are a lot of bits here; we have to load the new RIP, but not jump, and decrement RSP.
            MCTRL_MODE_FO_MI
                | MCTRL_BUSMODE_CONW_BUSM
                | ACTION_GCTRL_USE_ALT
                | GCTRL_ALT_CREG_IHPR
                | GCTRL_CREG_O
                | GCTRL_JM_YES
                | COMMAND_INHIBIT_JMFT,
            // NOTE We need a second RSP decrement to have happened here
            MCTRL_MODE_FI
                | MCTRL_BUSMODE_CONW_BUSB
                | RCTRL_IU3_BUSA_O
                | ACTION_GCTRL_USE_ALT
                | GCTRL_ALT_CREG_FG
                | GCTRL_CREG_O
                | COMMAND_RCTRL_RSP_EARLY_DEC_IU3RSP,
            MCTRL_MODE_FO_MI | MCTRL_BUSMODE_CONW_BUSM | GCTRL_FT_ENTER,
        ],
    ));

    builder.register(InstDef::with_single_0("HLT", I_HLT, GCTRL_JM_HALT));
    builder.register(InstDef::with_single_0("ABRT", I_ABRT, GCTRL_JM_ABRT));
}

const FARPREFIX: &str = "FAR.";

fn mk_distanced_instruction(
    far: bool,
    farbit: Word,
    name: &str,
    op: OpClass,
    args: Vec<ArgKind>,
    mut uis: Vec<UInst>,
) -> InstDef {
    let n = format!("{}{}", if far { FARPREFIX } else { "" }, name);
    for ui in &mut uis {
        *ui |= if far { 0 } else { MCTRL_FLAG_MODE_N_FAR };
    }
    InstDef::with_vec(&*n, op.add_flag(if far { farbit } else { 0 }), args, uis)
}

fn ucode_memb_sh_step1(is_write: bool, lo_or_hi: bool, zero: bool) -> UInst {
    return MCTRL_MODE_FI_MO
        | MCTRL_BUSMODE_CONH
        | (if lo_or_hi { ACTION_MCTRL_BUSMODE_X } else { 0 })
        | (if is_write { MCTRL_BUSMODE_WRITE } else { 0 })
        | RCTRL_IU1_BUSA_O
        | (if zero {
            0 /* The bus is pulled low. */
        } else {
            RCTRL_IU2_BUSB_O
        });
}

fn ucode_memb_ld_step2(lo_or_hi: bool, _zero: bool) -> UInst {
    return MCTRL_MODE_FO
        | MCTRL_BUSMODE_CONW_BUSB_MAYBEFLIP
        | (if lo_or_hi { ACTION_MCTRL_BUSMODE_X } else { 0 })
        | RCTRL_IU2_BUSB_I;
}

fn ucode_memb_st_step2(_lo_or_hi: bool, _zero: bool) -> UInst {
    return MCTRL_MODE_FO_MI | MCTRL_BUSMODE_CONW_BUSM;
}

fn mk_mem_byte_instruction(
    far: bool,
    farbit: Word,
    name: &str,
    op: OpClass,
    args: Vec<ArgKind>,
    is_write: bool,
    lo_or_hi: bool,
    zero: bool,
) -> InstDef {
    return mk_distanced_instruction(
        far,
        farbit,
        name,
        op,
        args,
        vec![
            ucode_memb_sh_step1(is_write, lo_or_hi, zero),
            (if is_write {
                ucode_memb_st_step2(lo_or_hi, zero)
            } else {
                ucode_memb_ld_step2(lo_or_hi, zero)
            }) | GCTRL_FT_ENTER,
        ],
    );
}

fn gen_mem_variants(builder: &mut Builder, far: bool) {
    builder.register(mk_distanced_instruction(
        far,
        ITFLAG_MEM_FAR,
        "LDW",
        I_LDW,
        vec![
            ArgKind::new_word(ConstPolicy::Allow),
            ArgKind::new_word(ConstPolicy::Never),
        ],
        vec![
            MCTRL_MODE_FI_MO | MCTRL_BUSMODE_CONW_BUSM | RCTRL_IU1_BUSA_O,
            MCTRL_MODE_FO | MCTRL_BUSMODE_CONW_BUSB | RCTRL_IU2_BUSB_I | GCTRL_FT_ENTER,
        ],
    ));

    builder.register(mk_mem_byte_instruction(
        far,
        ITFLAG_MEM_FAR,
        "LDBL",
        I_LDBL,
        vec![
            ArgKind::new_word(ConstPolicy::Allow),
            ArgKind::new_byte(Half::Lo, ConstPolicy::Never),
        ],
        false,
        false,
        false,
    ));
    builder.register(mk_mem_byte_instruction(
        far,
        ITFLAG_MEM_FAR,
        "LDBH",
        I_LDBH,
        vec![
            ArgKind::new_word(ConstPolicy::Allow),
            ArgKind::new_byte(Half::Hi, ConstPolicy::Never),
        ],
        false,
        true,
        false,
    ));
    builder.register(mk_mem_byte_instruction(
        far,
        ITFLAG_MEM_FAR,
        "LDBLZ",
        I_LDBLZ,
        vec![
            ArgKind::new_word(ConstPolicy::Allow),
            ArgKind::new_byte(Half::Lo, ConstPolicy::Never),
        ],
        false,
        false,
        true,
    ));
    builder.register(mk_mem_byte_instruction(
        far,
        ITFLAG_MEM_FAR,
        "LDBHZ",
        I_LDBHZ,
        vec![
            ArgKind::new_word(ConstPolicy::Allow),
            ArgKind::new_byte(Half::Hi, ConstPolicy::Never),
        ],
        false,
        true,
        true,
    ));

    builder.register(mk_distanced_instruction(
        far,
        ITFLAG_MEM_FAR,
        "STW",
        I_STW,
        vec![
            ArgKind::new_word(ConstPolicy::Allow),
            ArgKind::new_word(ConstPolicy::Allow),
        ],
        vec![
            MCTRL_MODE_FI | MCTRL_BUSMODE_CONW_BUSB | RCTRL_IU1_BUSA_O | RCTRL_IU2_BUSB_O,
            MCTRL_MODE_FO_MI | MCTRL_BUSMODE_CONW_BUSM | GCTRL_FT_ENTER,
        ],
    ));

    builder.register(mk_mem_byte_instruction(
        far,
        ITFLAG_MEM_FAR,
        "STBL",
        I_STBL,
        vec![
            ArgKind::new_word(ConstPolicy::Allow),
            ArgKind::new_byte(Half::Lo, ConstPolicy::Allow),
        ],
        true,
        false,
        false,
    ));
    builder.register(mk_mem_byte_instruction(
        far,
        ITFLAG_MEM_FAR,
        "STBH",
        I_STBH,
        vec![
            ArgKind::new_word(ConstPolicy::Allow),
            ArgKind::new_byte(Half::Hi, ConstPolicy::Allow),
        ],
        true,
        true,
        false,
    ));

    builder.register(mk_distanced_instruction(
        far,
        ICFLAG_MEM_IU3_FAR,
        "LDWO",
        I_LDWO,
        vec![
            ArgKind::new_word(ConstPolicy::Allow),
            ArgKind::new_word(ConstPolicy::Allow),
            ArgKind::new_word(ConstPolicy::Never),
        ],
        vec![
            ACTRL_INPUT_EN | ACTRL_MODE_ADD | RCTRL_IU1_BUSA_O | RCTRL_IU2_BUSB_O,
            MCTRL_MODE_FI_MO | MCTRL_BUSMODE_CONW_BUSM | ACTRL_DATA_OUT,
            MCTRL_MODE_FO | MCTRL_BUSMODE_CONW_BUSB | RCTRL_IU3_BUSB_I | GCTRL_FT_ENTER,
        ],
    ));

    builder.register(mk_distanced_instruction(
        far,
        ICFLAG_MEM_IU3_FAR,
        "STWO",
        I_STWO,
        vec![
            ArgKind::new_word(ConstPolicy::Allow),
            ArgKind::new_word(ConstPolicy::Allow),
            ArgKind::new_word(ConstPolicy::Allow),
        ],
        vec![
            ACTRL_INPUT_EN | ACTRL_MODE_ADD | RCTRL_IU1_BUSA_O | RCTRL_IU2_BUSB_O,
            MCTRL_MODE_FI | MCTRL_BUSMODE_CONW_BUSB | ACTRL_DATA_OUT | RCTRL_IU3_BUSB_O,
            MCTRL_MODE_FO_MI | MCTRL_BUSMODE_CONW_BUSM | GCTRL_FT_ENTER,
        ],
    ));

    // TODO did we ever envision "zero" variants for STBL and STBH?
}

fn gen_mem(builder: &mut Builder) {
    gen_mem_variants(builder, false);
    gen_mem_variants(builder, true);

    // The "far" selection for STPFX is a bit hacky, and works differently to normal mem IO
    builder.register(InstDef::with_single_1(
        "STPFX",
        I_STPFX,
        ArgKind::new_word(ConstPolicy::Never),
        MCTRL_MODE_STPFX | MCTRL_BUSMODE_CONW_BUSB | RCTRL_IU1_BUSB_O | GCTRL_FT_ENTER,
    ));
    builder.register(InstDef::with_single_1(
        "FAR.STPFX",
        I_STPFX.add_flag(ITFLAG_MEM_FAR),
        ArgKind::new_word(ConstPolicy::Never),
        MCTRL_MODE_STPFX_FAR | MCTRL_BUSMODE_CONW_BUSB | RCTRL_IU1_BUSB_O | GCTRL_FT_ENTER,
    ));
}

const LDJMPPREFIX: &str = "LD";

// `second_arg` means the instruction will take 2 arguments instead of 1, and we will use the value of the second arg
// for the direct jump/load jump.
fn mk_loadable_instruction_with_preamble(
    ld: bool,
    ldbit: Word,
    name: &str,
    op: OpClass,
    second_arg: bool,
    jm_w_cond: UInst,
    mut preamble: Vec<UInst>,
) -> InstDef {
    let n = format!("{}{}", if ld { LDJMPPREFIX } else { "" }, name);
    if ld {
        preamble.push(
            MCTRL_MODE_FI_MO
                | MCTRL_BUSMODE_CONW_BUSM
                | (if second_arg {
                    RCTRL_IU2_BUSA_O
                } else {
                    RCTRL_IU1_BUSA_O
                }),
        );
        preamble.push(MCTRL_MODE_FO | MCTRL_BUSMODE_CONW_BUSB | jm_w_cond);
    } else {
        preamble.push(
            jm_w_cond
                | (if second_arg {
                    RCTRL_IU2_BUSB_O
                } else {
                    RCTRL_IU1_BUSB_O
                }),
        );
    }

    if second_arg {
        InstDef::with_2(
            &*n,
            op.add_flag(if ld { ldbit } else { 0 }),
            ArgKind::new_word(ConstPolicy::Allow),
            ArgKind::new_word(ConstPolicy::Never),
            preamble,
        )
    } else {
        InstDef::with_1(
            &*n,
            op.add_flag(if ld { ldbit } else { 0 }),
            ArgKind::new_word(ConstPolicy::Allow),
            preamble,
        )
    }
}

// `second_arg` means the instruction will take 2 arguments instead of 1, and we will use the value of the second arg
// for the direct jump/load jump.
fn mk_loadable_instruction(
    ld: bool,
    ldbit: Word,
    name: &str,
    op: OpClass,
    second_arg: bool,
    jm_w_cond: UInst,
) -> InstDef {
    mk_loadable_instruction_with_preamble(ld, ldbit, name, op, second_arg, jm_w_cond, vec![])
}

fn gen_jmp_loadables(builder: &mut Builder, ld: bool) {
    builder.register(mk_loadable_instruction(
        ld,
        ITFLAG_JMP_LD,
        "JMP",
        I_JMP,
        false,
        GCTRL_JM_YES,
    ));

    builder.register(mk_loadable_instruction(
        ld,
        ITFLAG_JMP_LD,
        "JMP+DI",
        I_JMP_DI,
        false,
        ACTION_GCTRL_USE_ALT | GCTRL_ALT_P_IE | GCTRL_CREG_O | GCTRL_JM_YES,
    ));
    builder.register(mk_loadable_instruction(
        ld,
        ITFLAG_JMP_LD,
        "JMP+EI",
        I_JMP_EI,
        false,
        ACTION_GCTRL_USE_ALT | GCTRL_ALT_P_IE | GCTRL_CREG_I | GCTRL_JM_YES,
    ));

    builder.register(mk_loadable_instruction(
        ld,
        ITFLAG_JMP_LD,
        "JC",
        I_JC,
        false,
        GCTRL_JCOND_CARRY,
    ));
    builder.register(mk_loadable_instruction(
        ld,
        ITFLAG_JMP_LD,
        "JNC",
        I_JNC,
        false,
        GCTRL_JM_INVERTCOND | GCTRL_JCOND_CARRY,
    ));

    builder.register(mk_loadable_instruction(
        ld,
        ITFLAG_JMP_LD,
        "JZ",
        I_JZ,
        false,
        GCTRL_JM_INVERTCOND | GCTRL_JCOND_N_ZERO,
    ));
    builder.register(mk_loadable_instruction(
        ld,
        ITFLAG_JMP_LD,
        "JNZ",
        I_JNZ,
        false,
        GCTRL_JCOND_N_ZERO,
    ));

    builder.register(mk_loadable_instruction(
        ld,
        ITFLAG_JMP_LD,
        "JS",
        I_JS,
        false,
        GCTRL_JCOND_SIGN,
    ));
    builder.register(mk_loadable_instruction(
        ld,
        ITFLAG_JMP_LD,
        "JNS",
        I_JNS,
        false,
        GCTRL_JM_INVERTCOND | GCTRL_JCOND_SIGN,
    ));

    builder.register(mk_loadable_instruction(
        ld,
        ITFLAG_JMP_LD,
        "JO",
        I_JO,
        false,
        GCTRL_JM_INVERTCOND | GCTRL_JCOND_N_OVFLW,
    ));
    builder.register(mk_loadable_instruction(
        ld,
        ITFLAG_JMP_LD,
        "JNO",
        I_JNO,
        false,
        GCTRL_JCOND_N_OVFLW,
    ));

    builder.register(mk_loadable_instruction_with_preamble(
        ld,
        ITFLAG_JMP_LD,
        "LJMP",
        I_LJMP,
        true,
        GCTRL_JM_YES,
        vec![MCTRL_MODE_STPFX | MCTRL_BUSMODE_CONW_BUSB | RCTRL_IU1_BUSB_O],
    ));
}

fn gen_jmp(builder: &mut Builder) {
    gen_jmp_loadables(builder, false);
    gen_jmp_loadables(builder, true);
}

fn gen_reg(builder: &mut Builder) {
    builder.register(InstDef::with_single_2(
        "MOV",
        I_MOV,
        ArgKind::new_word(ConstPolicy::Allow),
        ArgKind::new_word(ConstPolicy::Never),
        RCTRL_IU1_BUSA_O | RCTRL_IU2_BUSA_I | GCTRL_FT_ENTER,
    ));
}

fn gen_ctl(builder: &mut Builder) {
    builder.register(InstDef::with_single_1(
        "LFG",
        I_LFG,
        ArgKind::new_word(ConstPolicy::Allow),
        RCTRL_IU1_BUSB_O | ACTION_GCTRL_USE_ALT | GCTRL_ALT_CREG_FG | GCTRL_CREG_I | GCTRL_FT_ENTER,
    ));
    builder.register(InstDef::with_single_1(
        "LIHP",
        I_LIHP,
        ArgKind::new_word(ConstPolicy::Allow),
        RCTRL_IU1_BUSB_O
            | ACTION_GCTRL_USE_ALT
            | GCTRL_ALT_CREG_IHPR
            | GCTRL_CREG_I
            | GCTRL_FT_ENTER,
    ));

    builder.register(InstDef::with_single_0(
        "DI",
        I_DI,
        ACTION_GCTRL_USE_ALT | GCTRL_ALT_P_IE | GCTRL_CREG_O | GCTRL_FT_ENTER,
    ));
    builder.register(InstDef::with_single_0(
        "EI",
        I_EI,
        ACTION_GCTRL_USE_ALT | GCTRL_ALT_P_IE | GCTRL_CREG_I | GCTRL_FT_ENTER,
    ));
}

fn mk_alu_inst<'a>(
    name: &str,
    op: OpClass,
    args: Vec<ArgKind>,
    alu_mode: UInst,
    backward: bool,
    oc_flag: Word,
    use_oc_flag: bool,
    suffix: &str,
    out_mode: UInst,
) -> InstDef {
    if backward && args.len() != 2 {
        panic!("can only reverse 2 args");
    }

    let (srcs, tgt) = match args.len() {
        0 => panic!("zero arg arith instruction"),
        1 => (RCTRL_IU1_BUSA_O, RCTRL_IU1_BUSA_I),
        2 => {
            if backward {
                (RCTRL_IU2_BUSA_O | RCTRL_IU1_BUSB_O, RCTRL_IU2_BUSA_I)
            } else {
                (RCTRL_IU1_BUSA_O | RCTRL_IU2_BUSB_O, RCTRL_IU2_BUSA_I)
            }
        }
        3 => (RCTRL_IU1_BUSA_O | RCTRL_IU2_BUSB_O, RCTRL_IU3_BUSA_I),
        _ => panic!("too many args!"),
    };

    InstDef::with_vec(
        &*format!("{}{}", name, suffix),
        op.add_flag(if use_oc_flag { oc_flag } else { 0 }),
        args,
        vec![
            ACTRL_INPUT_EN | alu_mode | srcs | RCTRL_IU1_BUSA_O,
            out_mode
                | (if (out_mode & ACTRL_DATA_OUT) != 0 {
                    tgt
                } else {
                    0
                })
                | GCTRL_FT_ENTER,
        ],
    )
}

fn gen_alu_possible_noflag_variant(
    builder: &mut Builder,
    suffix: &str,
    out_mode: UInst,
    use_oc_flag: bool,
) {
    builder.register(mk_alu_inst(
        "ADD2",
        I_ADD2,
        vec![
            ArgKind::new_word(ConstPolicy::Allow),
            ArgKind::new_word(ConstPolicy::Never),
        ],
        ACTRL_MODE_ADD,
        false,
        ICFLAG_ALU1_NOFGS,
        use_oc_flag,
        suffix,
        out_mode,
    )); // c.f. the ADD3 variant
    builder.register(mk_alu_inst(
        "SUB",
        I_SUB,
        vec![
            ArgKind::new_word(ConstPolicy::Allow),
            ArgKind::new_word(ConstPolicy::Never),
        ],
        ACTRL_MODE_SUB,
        false,
        ICFLAG_ALU1_NOFGS,
        use_oc_flag,
        suffix,
        out_mode,
    ));
    builder.register(mk_alu_inst(
        "BSUB",
        I_BSUB,
        vec![
            ArgKind::new_word(ConstPolicy::Allow),
            ArgKind::new_word(ConstPolicy::Never),
        ],
        ACTRL_MODE_SUB,
        true,
        ICFLAG_ALU1_NOFGS,
        use_oc_flag,
        suffix,
        out_mode,
    ));
    builder.register(mk_alu_inst(
        "AND",
        I_AND,
        vec![
            ArgKind::new_word(ConstPolicy::Allow),
            ArgKind::new_word(ConstPolicy::Never),
        ],
        ACTRL_MODE_AND,
        false,
        ICFLAG_ALU1_NOFGS,
        use_oc_flag,
        suffix,
        out_mode,
    ));
    builder.register(mk_alu_inst(
        "OR",
        I_OR,
        vec![
            ArgKind::new_word(ConstPolicy::Allow),
            ArgKind::new_word(ConstPolicy::Never),
        ],
        ACTRL_MODE_OR,
        false,
        ICFLAG_ALU1_NOFGS,
        use_oc_flag,
        suffix,
        out_mode,
    ));
    builder.register(mk_alu_inst(
        "XOR",
        I_XOR,
        vec![
            ArgKind::new_word(ConstPolicy::Allow),
            ArgKind::new_word(ConstPolicy::Never),
        ],
        ACTRL_MODE_XOR,
        false,
        ICFLAG_ALU1_NOFGS,
        use_oc_flag,
        suffix,
        out_mode,
    ));
    builder.register(mk_alu_inst(
        "LSFT",
        I_LSFT,
        vec![ArgKind::new_word(ConstPolicy::Never)],
        ACTRL_MODE_LSFT,
        false,
        ICFLAG_ALU1_NOFGS,
        use_oc_flag,
        suffix,
        out_mode,
    ));
    builder.register(mk_alu_inst(
        "RSFT",
        I_RSFT,
        vec![ArgKind::new_word(ConstPolicy::Never)],
        ACTRL_MODE_RSFT,
        false,
        ICFLAG_ALU1_NOFGS,
        use_oc_flag,
        suffix,
        out_mode,
    ));

    builder.register(mk_alu_inst(
        "ADD3",
        I_ADD3,
        vec![
            ArgKind::new_word(ConstPolicy::Allow),
            ArgKind::new_word(ConstPolicy::Allow),
            ArgKind::new_word(ConstPolicy::Never),
        ],
        ACTRL_MODE_ADD,
        false,
        ICFLAG_ADD3_IU3_NF,
        use_oc_flag,
        suffix,
        out_mode,
    ));
}

const ALU_OUTMODE_NORMAL: UInst = ACTRL_DATA_OUT
    | ACTRL_FLAGS_OUT
    | ACTION_GCTRL_USE_ALT
    | GCTRL_ALT_P_O_CHNMI_OR_I_ALUFG
    | GCTRL_CREG_I;
const ALU_OUTMODE_NOFLAGS: UInst = ACTRL_DATA_OUT;
const ALU_OUTMODE_FLAGSONLY: UInst =
    ACTRL_FLAGS_OUT | ACTION_GCTRL_USE_ALT | GCTRL_ALT_P_O_CHNMI_OR_I_ALUFG | GCTRL_CREG_I;

fn gen_alu(builder: &mut Builder) {
    gen_alu_possible_noflag_variant(builder, "", ALU_OUTMODE_NORMAL, false);
    gen_alu_possible_noflag_variant(builder, "NF", ALU_OUTMODE_NOFLAGS, true);

    // The TST instruction has no NOFLAGS (NF) variant, since then it would have no effect.
    builder.register(mk_alu_inst(
        "TST",
        I_TST,
        vec![ArgKind::new_word(ConstPolicy::Allow)],
        ACTRL_MODE_TST,
        false,
        0,
        false,
        "",
        ALU_OUTMODE_NORMAL,
    ));

    // Subtract one operand from the other to perform the comparison.
    // e.g. FLAG_SIGN tells you which is greater.
    builder.register(mk_alu_inst(
        "CMP",
        I_CMP,
        vec![
            ArgKind::new_word(ConstPolicy::Allow),
            ArgKind::new_word(ConstPolicy::Never),
        ],
        ACTRL_MODE_SUB,
        true,
        0,
        false,
        "",
        ALU_OUTMODE_FLAGSONLY,
    ));
}

fn gen_stk(builder: &mut Builder) {
    // Faster version of: PUSH rbp; MOV rsp rbp;, i.e. (PUSH rbp; MOV rsp rbp;)
    builder.register(InstDef::with_1(
        "ENTER1",
        I_ENTER1,
        ArgKind::new_word(ConstPolicy::Never),
        vec![
            // IU1 = MUST BE RBP
            MCTRL_MODE_FI
                | MCTRL_BUSMODE_CONW_BUSB
                | RCTRL_IU3_BUSA_O
                | GCTRL_NRM_IU3_OVERRIDE_O_SELECT_RSP_I__UNUSED
                | RCTRL_IU1_BUSB_O
                | COMMAND_RCTRL_RSP_EARLY_DEC_IU3RSP,
            MCTRL_MODE_FO_MI
                | MCTRL_BUSMODE_CONW_BUSM
                | RCTRL_IU3_BUSA_O
                | GCTRL_NRM_IU3_OVERRIDE_O_SELECT_RSP_I__UNUSED
                | RCTRL_IU1_BUSA_I
                | GCTRL_FT_ENTER,
        ],
    ));

    // IU1 must be RBP, IU2 = $CONST or reg, IU3 is forced to RSP
    // Faster version of: PUSH rbp; MOV rsp rbp; SUBNF $CONST, rsp;
    builder.register(InstDef::with_2("ENTERFR2", I_ENTERFR2, ArgKind::new_word(ConstPolicy::Never), ArgKind::new_word(ConstPolicy::Allow), vec![
        // IU1 = MUST BE RBP
        // PUSH rbp; MOV rsp rbp;
        MCTRL_MODE_FI    | MCTRL_BUSMODE_CONW_BUSB | RCTRL_IU3_BUSA_O | RCTRL_IU1_BUSB_O | COMMAND_RCTRL_RSP_EARLY_DEC_IU3RSP,
        MCTRL_MODE_FO_MI | MCTRL_BUSMODE_CONW_BUSM | RCTRL_IU3_BUSB_O | GCTRL_NRM_IU3_OVERRIDE_O_SELECT_RSP_I__UNUSED | RCTRL_IU1_BUSB_I
        // SUBNF $CONST, rsp
                | RCTRL_IU2_BUSA_O | ACTRL_INPUT_EN | ACTRL_MODE_SUB,
        ACTRL_DATA_OUT | RCTRL_IU3_BUSA_I | GCTRL_NRM_IU3_OVERRIDE_O_SELECT_RSP_I__UNUSED | GCTRL_FT_ENTER,
    ]));

    // Faster version of: MOV rbp rsp; POP rbp, i.e. (MOV rbp rsp; POP rbp;)
    // instead we do them both simultaneously.
    builder.register(InstDef::with_1(
        "LEAVE1",
        I_LEAVE1,
        ArgKind::new_word(ConstPolicy::Never),
        vec![
            // IU1 = MUST BE RBP
            MCTRL_MODE_FI_MO
                | MCTRL_BUSMODE_CONW_BUSM
                | RCTRL_IU3_BUSA_I
                | GCTRL_NRM_IU3_OVERRIDE_O_SELECT_RSP_I__UNUSED
                | RCTRL_IU1_BUSA_O
                | COMMAND_RCTRL_RSP_EARLY_DEC_IU3RSP,
            MCTRL_MODE_FO
                | MCTRL_BUSMODE_CONW_BUSB
                | RCTRL_IU1_BUSB_I
                | COMMAND_RCTRL_RSP_EARLY_INC
                | GCTRL_FT_ENTER,
        ],
    ));

    // NOTE `PUSH %rsp` writes the NEW %rsp to the NEW address. (This happens to be the old 8086 behaviour, but not 286 and beyond.)
    builder.register(InstDef::with_1(
        "PUSH",
        I_PUSH,
        ArgKind::new_word(ConstPolicy::Allow),
        vec![
            MCTRL_MODE_FI
                | MCTRL_BUSMODE_CONW_BUSB
                | RCTRL_IU3_BUSA_O
                | GCTRL_NRM_IU3_OVERRIDE_O_SELECT_RSP_I__UNUSED
                | RCTRL_IU1_BUSB_O
                | COMMAND_RCTRL_RSP_EARLY_DEC_IU3RSP,
            MCTRL_MODE_FO_MI | MCTRL_BUSMODE_CONW_BUSM | GCTRL_FT_ENTER,
        ],
    ));

    // NOTE `POP %rsp` writes the OLD TOP OF STACK to the NEW %rsp (unchanged).
    builder.register(InstDef::with_1(
        "POP",
        I_POP,
        ArgKind::new_word(ConstPolicy::Never),
        vec![
            MCTRL_MODE_FI_MO
                | MCTRL_BUSMODE_CONW_BUSM
                | RCTRL_IU3_BUSA_O
                | GCTRL_NRM_IU3_OVERRIDE_O_SELECT_RSP_I__UNUSED,
            MCTRL_MODE_FO
                | MCTRL_BUSMODE_CONW_BUSB
                | RCTRL_IU1_BUSB_I
                | COMMAND_RCTRL_RSP_EARLY_INC
                | GCTRL_FT_ENTER,
        ],
    ));

    builder.register(InstDef::with_0(
        "PUSHFG",
        I_PUSHFG,
        vec![
            MCTRL_MODE_FI
                | MCTRL_BUSMODE_CONW_BUSB
                | RCTRL_IU3_BUSA_O
                | ACTION_GCTRL_USE_ALT
                | GCTRL_ALT_CREG_FG
                | GCTRL_CREG_O
                | COMMAND_RCTRL_RSP_EARLY_DEC_IU3RSP,
            MCTRL_MODE_FO_MI | MCTRL_BUSMODE_CONW_BUSM | GCTRL_FT_ENTER,
        ],
    ));

    builder.register(InstDef::with_0(
        "POPFG",
        I_POPFG,
        vec![
            MCTRL_MODE_FI_MO
                | MCTRL_BUSMODE_CONW_BUSM
                | RCTRL_IU3_BUSA_O
                | GCTRL_NRM_IU3_OVERRIDE_O_SELECT_RSP_I__UNUSED,
            MCTRL_MODE_FO
                | MCTRL_BUSMODE_CONW_BUSB
                | ACTION_GCTRL_USE_ALT
                | GCTRL_ALT_CREG_FG
                | GCTRL_CREG_I
                | COMMAND_RCTRL_RSP_EARLY_INC
                | GCTRL_FT_ENTER,
        ],
    ));

    // Effectively `PUSH RIP`
    builder.register(InstDef::with_1(
        "CALL",
        I_CALL,
        ArgKind::new_word(ConstPolicy::Allow),
        vec![
            MCTRL_MODE_FI
                | MCTRL_BUSMODE_CONW_BUSB
                | RCTRL_IU3_BUSA_O
                | GCTRL_NRM_IU3_OVERRIDE_O_SELECT_RSP_I__UNUSED
                | GCTRL_JM_P_RIP_BUSB_O
                | COMMAND_RCTRL_RSP_EARLY_DEC_IU3RSP,
            MCTRL_MODE_FO_MI | MCTRL_BUSMODE_CONW_BUSM | RCTRL_IU1_BUSB_O | GCTRL_JM_YES,
        ],
    ));

    // Effectively `POP RIP`
    builder.register(InstDef::with_0(
        "RET",
        I_RET,
        vec![
            MCTRL_MODE_FI_MO
                | MCTRL_BUSMODE_CONW_BUSM
                | RCTRL_IU3_BUSA_O
                | GCTRL_NRM_IU3_OVERRIDE_O_SELECT_RSP_I__UNUSED,
            MCTRL_MODE_FO | MCTRL_BUSMODE_CONW_BUSB | COMMAND_RCTRL_RSP_EARLY_INC | GCTRL_JM_YES,
        ],
    ));

    // Effectively `POPFG; RET [+ clear CBHIT_HNMI]`
    builder.register(InstDef::with_0(
        "IRET",
        I_IRET,
        vec![
            MCTRL_MODE_FI_MO
                | MCTRL_BUSMODE_CONW_BUSM
                | RCTRL_IU3_BUSA_O
                | GCTRL_NRM_IU3_OVERRIDE_O_SELECT_RSP_I__UNUSED,
            MCTRL_MODE_FO
                | MCTRL_BUSMODE_CONW_BUSB
                | COMMAND_RCTRL_RSP_EARLY_INC
                | ACTION_GCTRL_USE_ALT
                | GCTRL_ALT_CREG_FG
                | GCTRL_CREG_I,
            MCTRL_MODE_FI_MO
                | MCTRL_BUSMODE_CONW_BUSM
                | RCTRL_IU3_BUSA_O
                | GCTRL_NRM_IU3_OVERRIDE_O_SELECT_RSP_I__UNUSED,
            MCTRL_MODE_FO
                | MCTRL_BUSMODE_CONW_BUSB
                | COMMAND_RCTRL_RSP_EARLY_INC
                | ACTION_GCTRL_USE_ALT
                | GCTRL_ALT_P_O_CHNMI_OR_I_ALUFG
                | GCTRL_CREG_O
                | GCTRL_JM_YES,
        ],
    ));
}

fn gen_io(builder: &mut Builder) {
    builder.register(InstDef::with_single_2(
        "IOR",
        I_IOR,
        ArgKind::new_word(ConstPolicy::Allow),
        ArgKind::new_word(ConstPolicy::Never),
        RCTRL_IU1_BUSA_O
            | RCTRL_IU2_BUSB_I
            | GCTRL_NRM_IO_READWRITE
            | GCTRL_CREG_I
            | GCTRL_FT_ENTER,
    ));
    builder.register(InstDef::with_single_2(
        "IOW",
        I_IOW,
        ArgKind::new_word(ConstPolicy::Allow),
        ArgKind::new_word(ConstPolicy::Allow),
        RCTRL_IU1_BUSA_O
            | RCTRL_IU2_BUSB_O
            | GCTRL_NRM_IO_READWRITE
            | GCTRL_CREG_O
            | GCTRL_FT_ENTER,
    ));
}

fn gen_optimizations(builder: &mut Builder) {
    builder.register(InstDef::with_2(
        "PUSHx2",
        I_PUSHX2,
        ArgKind::new_word(ConstPolicy::Allow),
        ArgKind::new_word(ConstPolicy::Allow),
        vec![
            MCTRL_MODE_FI
                | MCTRL_BUSMODE_CONW_BUSB
                | RCTRL_IU3_BUSA_O
                | GCTRL_NRM_IU3_OVERRIDE_O_SELECT_RSP_I__UNUSED
                | RCTRL_IU1_BUSB_O
                | COMMAND_RCTRL_RSP_EARLY_DEC_IU3RSP,
            MCTRL_MODE_FO_MI | MCTRL_BUSMODE_CONW_BUSM,
            MCTRL_MODE_FI
                | MCTRL_BUSMODE_CONW_BUSB
                | RCTRL_IU3_BUSA_O
                | GCTRL_NRM_IU3_OVERRIDE_O_SELECT_RSP_I__UNUSED
                | RCTRL_IU2_BUSB_O
                | COMMAND_RCTRL_RSP_EARLY_DEC_IU3RSP,
            MCTRL_MODE_FO_MI | MCTRL_BUSMODE_CONW_BUSM | GCTRL_FT_ENTER,
        ],
    ));

    builder.register(InstDef::with_2(
        "POPx2",
        I_POPX2,
        ArgKind::new_word(ConstPolicy::Never),
        ArgKind::new_word(ConstPolicy::Never),
        vec![
            MCTRL_MODE_FI_MO
                | MCTRL_BUSMODE_CONW_BUSM
                | RCTRL_IU3_BUSA_O
                | GCTRL_NRM_IU3_OVERRIDE_O_SELECT_RSP_I__UNUSED,
            MCTRL_MODE_FO
                | MCTRL_BUSMODE_CONW_BUSB
                | RCTRL_IU1_BUSB_I
                | COMMAND_RCTRL_RSP_EARLY_INC,
            MCTRL_MODE_FI_MO
                | MCTRL_BUSMODE_CONW_BUSM
                | RCTRL_IU3_BUSA_O
                | GCTRL_NRM_IU3_OVERRIDE_O_SELECT_RSP_I__UNUSED,
            MCTRL_MODE_FO
                | MCTRL_BUSMODE_CONW_BUSB
                | RCTRL_IU2_BUSB_I
                | COMMAND_RCTRL_RSP_EARLY_INC
                | GCTRL_FT_ENTER,
        ],
    ));
}

pub(in super::super) fn register(builder: &mut Builder) {
    /* If we want to go to 8~10 general purpose registers, I think we could make do with only
    7 instruction bits compared to 9. */

    gen_sys(builder);
    gen_jmp(builder);
    gen_reg(builder);
    gen_ctl(builder);
    gen_mem(builder);
    gen_alu(builder);
    gen_io(builder);
    gen_stk(builder);
    gen_optimizations(builder);
}
