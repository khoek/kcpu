use super::super::pic::Pic;
use crate::spec::types::hw::*;
use crate::vm::interface;
use crate::vm::io::types::*;

const PORT_BASE: Word = 0xD0;
// Connect the TUI line of CTL, ANDed with NOT_CLOCK, to the NMI assert of the PIC
const FLAG_TUI2NMI: Word = 0x0001;

pub struct Jumpers {
    pic: Handle<Pic>,

    flags: Word,
}

impl Jumpers {
    pub fn new(pic: Handle<Pic>) -> Self {
        Self { pic, flags: 0 }
    }
}

impl SinglePortDevice for Jumpers {
    fn reserved_port(&self) -> Word {
        PORT_BASE
    }

    fn write(&mut self, val: Word) -> HalfcycleCount {
        self.flags = val;
        0
    }

    fn read(&mut self) -> (HalfcycleCount, Word) {
        (0, self.flags)
    }

    fn process_halfcycle(&mut self, sigs: ClockedSignals) {
        if self.flags & FLAG_TUI2NMI != 0 {
            if let ClockedSignals::OffClock(_, true) = sigs {
                interface::Pic::assert(&self.pic, interface::PicIrq::Nmi);
            }
        }
    }
}
