use crate::assembler::{lang::Builder, model::Family};

// RUSTFIX the comment below is old, but it gives a reason for not switching
// `ArgKind` to just `Src` and `Dst` values; const only might have optimization benefits?

// NOTE We could just enforce the "noconst" information from the underlying aliases---
// instead of tagging it again here, but the upshot is that we can define a different
// alias for a const argument if we want, which I think can save a few uops in a few
// places.

fn gen_ctl(builder: &mut Builder) {
    builder.register_family(Family::with("ENTER", vec!["ENTER0", "ENTER1"]));
    builder.register_family(Family::with("ENTERFR", vec!["ENTERFR1", "ENTERFR2"]));
    builder.register_family(Family::with("LEAVE", vec!["LEAVE0", "LEAVE1"]));
}

fn gen_mem(builder: &mut Builder) {
    builder.register_family(Family::with("LD", vec!["LDW", "LDBL", "LDBH", "LDWO"]));
    builder.register_family(Family::with("LDZ", vec!["LDBLZ", "LDBHZ"]));
    builder.register_family(Family::with("ST", vec!["STW", "STBL", "STBH", "STWO"]));
}

fn gen_alu(builder: &mut Builder) {
    builder.register_family(Family::with("ADD", vec!["ADD2", "ADD3"]));
    builder.register_family(Family::with("ADDNF", vec!["ADD2NF", "ADD3NF"]));
}

pub(crate) fn register(builder: &mut Builder) {
    gen_ctl(builder);
    gen_mem(builder);
    gen_alu(builder);
}
