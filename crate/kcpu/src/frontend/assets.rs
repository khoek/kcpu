use super::assemble;
use once_cell::sync::Lazy;
use std::path::PathBuf;

pub const DEFAULT_BINARY_EXT: &str = "kb";

// RUSTFIX make this const once `PathBuf` is.
pub fn get_default_testsuite_dir() -> PathBuf {
    ["asm", "test"].iter().collect()
}

// RUSTFIX remove duplication once `const fn`s become more powerful.
static DEFAULT_BIOS_SRC: &str = include_str!("assets/default.bios.ks");
static DEFAULT_PROG_SRC: &str = include_str!("assets/default.prog.ks");
static DEFAULT_BIOS_BIN: Lazy<Vec<u8>> = Lazy::new(assemble_default_bios);
static DEFAULT_PROG_BIN: Lazy<Vec<u8>> = Lazy::new(assemble_default_prog);

fn assemble_default_bios() -> Vec<u8> {
    assemble::assemble(DEFAULT_BIOS_SRC)
        .expect("Could not compile binary-packaged default BIOS source file")
}

fn assemble_default_prog() -> Vec<u8> {
    assemble::assemble(DEFAULT_PROG_SRC)
        .expect("Could not compile binary-packaged default PROGRAM source file")
}

pub fn get_default_bios() -> &'static Vec<u8> {
    Lazy::force(&DEFAULT_BIOS_BIN)
}

pub fn get_default_prog() -> &'static Vec<u8> {
    Lazy::force(&DEFAULT_PROG_BIN)
}
