use super::super::types::*;
use crate::spec::types::hw::*;

const PORT_BASE: Word = 0x00;

pub struct Probe {
    target_port: Word,
    // RUSTFIX expose a way to access this data being stored in the manager, so that we can change these at runtime
    ports: Vec<Word>,
}

impl Probe {
    pub fn new(ports: Vec<Word>) -> Self {
        Probe {
            target_port: 0,
            ports,
        }
    }
}

impl SinglePortDevice for Probe {
    fn reserved_port(&self) -> Word {
        PORT_BASE
    }

    fn write(&mut self, val: Word) -> HalfcycleCount {
        self.target_port = val;
        0
    }

    fn read(&mut self) -> (HalfcycleCount, Word) {
        // This assertion is not formally neccesary, but is useful to detect bad port IO.
        // i.e. when we read from port 0 by accident, when we aren't using the probe function.
        assert_ne!(self.target_port, 0);
        (
            0,
            if self.ports.contains(&self.target_port) {
                1
            } else {
                0
            },
        )
    }

    fn process_halfcycle(&mut self, _: ClockedSignals) {}
}
