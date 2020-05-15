pub mod disasm;
pub mod lang;
pub mod model;
pub mod phases;

mod defs;

pub use phases::types::Error;

use crate::spec::types::hw::{self, Byte, Word};

// RUSTFIX ERROR OVERHAUL:    Exception overhaul, just use `format!()` in-place to generate the messages,
//                            since we are just doing `to_owned` spam everywhere now and the slices were
//                            limiting in some places when I was originally writing the messages.

pub fn assemble(source: &str) -> Result<Vec<Word>, Error> {
    let tokens = phases::tokenize(source)?;
    let statements = phases::parse(tokens)?;
    let elems = phases::generate(statements)?;
    let bins = phases::resolve(elems)?;

    Ok(bins)
}

pub fn assemble_bytes(prog: &str) -> Result<Vec<Byte>, Error> {
    Ok(hw::words_to_bytes(assemble(prog)?))
}
