use crate::assembler::{
    lang::Builder,
    model::{Alias, Slot, Virtual},
};
use crate::spec::{defs::opclass::*, types::hw::PReg};

/*
    IMPORTANT NOTE: If we re-add aliases which share PReg::ID between multiple instructions,
    we need to save/restore it in PUSHA/POPA.
*/

pub(crate) fn gen_ctl(builder: &mut Builder) {
    // The usual versions of ENTER[FR]/LEAVE which use RBP as the base pointer.
    builder.register_alias(Alias::with_single(
        "ENTER0",
        Virtual::with_1(I_ENTER1, Slot::with_wreg(PReg::BP)),
    ));
    builder.register_alias(Alias::with_single(
        "ENTERFR1",
        Virtual::with_2(I_ENTERFR2, Slot::with_wreg(PReg::BP), Slot::with_arg(0)),
    ));
    builder.register_alias(Alias::with_single(
        "LEAVE0",
        Virtual::with_1(I_LEAVE1, Slot::with_wreg(PReg::BP)),
    ));
}

pub(crate) fn gen_mem(builder: &mut Builder) {
    builder.register_alias(Alias::with(
        "PUSHA",
        vec![
            Virtual::with_2(I_PUSHX2, Slot::with_wreg(PReg::A), Slot::with_wreg(PReg::B)),
            Virtual::with_2(I_PUSHX2, Slot::with_wreg(PReg::C), Slot::with_wreg(PReg::D)),
            Virtual::with_2(
                I_PUSHX2,
                Slot::with_wreg(PReg::E),
                Slot::with_wreg(PReg::BP),
            ),
        ],
    ));
    builder.register_alias(Alias::with(
        "POPA",
        vec![
            Virtual::with_2(I_POPX2, Slot::with_wreg(PReg::BP), Slot::with_wreg(PReg::E)),
            Virtual::with_2(I_POPX2, Slot::with_wreg(PReg::D), Slot::with_wreg(PReg::C)),
            Virtual::with_2(I_POPX2, Slot::with_wreg(PReg::B), Slot::with_wreg(PReg::A)),
        ],
    ));
}

pub(crate) fn gen_alu(builder: &mut Builder) {
    // XOR the oprand with 0xFFFF
    builder.register_alias(Alias::with_single(
        "NOT",
        Virtual::with_2(I_XOR, Slot::with_wconst(0xFFFF), Slot::with_arg(0)),
    ));

    // Subtract the operand from zero
    builder.register_alias(Alias::with_single(
        "NEG",
        Virtual::with_2(I_BSUB, Slot::with_wconst(0x0000), Slot::with_arg(0)),
    ));

    // Add one to the operand
    builder.register_alias(Alias::with_single(
        "INC",
        Virtual::with_2(I_ADD2, Slot::with_wconst(0x0001), Slot::with_arg(0)),
    ));

    builder.register_alias(Alias::with_single(
        "JE",
        Virtual::with_1(I_JZ, Slot::with_arg(0)),
    ));
    builder.register_alias(Alias::with_single(
        "JNE",
        Virtual::with_1(I_JNZ, Slot::with_arg(0)),
    ));
    // RUSTFIX test these three: (and commented ones at the bottom)
    builder.register_alias(Alias::with_single(
        "JL",
        Virtual::with_1(I_JNC, Slot::with_arg(0)),
    ));
    builder.register_alias(Alias::with_single(
        "JNL",
        Virtual::with_1(I_JC, Slot::with_arg(0)),
    ));
    builder.register_alias(Alias::with_single(
        "JGE",
        Virtual::with_1(I_JC, Slot::with_arg(0)),
    ));

    builder.register_alias(Alias::with_single(
        "LDJE",
        Virtual::with_1(I_JZ.with_flag(ITFLAG_JMP_LD), Slot::with_arg(0)),
    ));
    builder.register_alias(Alias::with_single(
        "LDJNE",
        Virtual::with_1(I_JNZ.with_flag(ITFLAG_JMP_LD), Slot::with_arg(0)),
    ));
    builder.register_alias(Alias::with_single(
        "LDJL",
        Virtual::with_1(I_JNC.with_flag(ITFLAG_JMP_LD), Slot::with_arg(0)),
    ));
    builder.register_alias(Alias::with_single(
        "LDJNL",
        Virtual::with_1(I_JC.with_flag(ITFLAG_JMP_LD), Slot::with_arg(0)),
    ));
    builder.register_alias(Alias::with_single(
        "LDJGE",
        Virtual::with_1(I_JC.with_flag(ITFLAG_JMP_LD), Slot::with_arg(0)),
    ));

    // TODO check these, I think they are just wrong
    // builder.register_alias(Alias::new("JLE", unbound_opcode(I_JS,  Slot::with_arg(0) )));
    // builder.register_alias(Alias::new("JG" , unbound_opcode(I_JNS, Slot::with_arg(0) )));
}

pub(crate) fn register(builder: &mut Builder) {
    gen_ctl(builder);
    gen_mem(builder);
    gen_alu(builder);
}
