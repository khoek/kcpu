pub(super) mod types;

pub(super) mod generate;
pub(super) mod parse;
pub(super) mod resolve;
pub(super) mod tokenize;

pub use types::Error;

use crate::spec::types::hw::Word;

// RUSTFIX ERROR OVERHAUL:    Exception overhaul, just use `format!()` in-place to generate the messages,
//                            since we are just doing `to_owned` spam everywhere now and the slices were
//                            limiting in some places when I was originally writing the messages.

pub fn assemble(source: &str) -> Result<Vec<Word>, Error> {
    let tokens = tokenize::tokenize(source)?;
    let statements = parse::parse(tokens)?;
    let elems = generate::generate(statements)?;
    let bins = resolve::resolve(elems)?;

    Ok(bins)
}