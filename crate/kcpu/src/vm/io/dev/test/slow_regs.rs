use crate::spec::types::hw::Word;
use crate::vm::io::types::{ClockedSignals, HalfcycleCount, SinglePortDevice};

const PORT_BASE: Word = 0xF0;

pub struct SlowRegs {
    delay: HalfcycleCount,
    reg: Word,
}

impl SlowRegs {
    pub fn new(delay: HalfcycleCount) -> Self {
        SlowRegs { delay, reg: 0 }
    }
}

impl SinglePortDevice for SlowRegs {
    fn reserved_port(&self) -> Word {
        PORT_BASE + (self.delay as Word)
    }

    fn write(&mut self, val: Word) -> HalfcycleCount {
        self.reg = val;
        self.delay
    }

    fn read(&mut self) -> (HalfcycleCount, Word) {
        (self.delay, self.reg)
    }

    fn process_halfcycle(&mut self, _: ClockedSignals) {}
}
