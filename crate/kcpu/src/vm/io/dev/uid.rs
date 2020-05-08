use super::super::types::*;
use crate::spec::types::hw::*;

const PORT_BASE: Word = 0xA0;
const UID: Word = 0xBEEF;

pub struct Uid {}

impl Uid {
    pub fn new() -> Self {
        Uid {}
    }
}

impl SinglePortDevice for Uid {
    fn reserved_port(&self) -> Word {
        PORT_BASE
    }

    fn write(&mut self, _: Word) -> HalfcycleCount {
        panic!("writing to UID register");
    }

    fn read(&mut self) -> (HalfcycleCount, Word) {
        (0, UID)
    }

    fn process_halfcycle(&mut self, _: ClockedSignals) {}
}
