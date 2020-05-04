use crate::assembler;
use crate::spec::types::hw;
use std::path::Path;

pub fn assemble(prog: &str) -> Result<Vec<u8>, assembler::Error> {
    Ok(hw::words_to_bytes(assembler::assemble(prog)?))
}

pub fn assemble_path(path: &Path) -> Result<Vec<u8>, assembler::Error> {
    // RUSTFIX proper IO error handling
    let prog_src = std::fs::read_to_string(path).unwrap();
    assemble(&prog_src)
}
