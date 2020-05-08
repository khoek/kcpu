use super::super::pic::Pic;
use crate::spec::types::hw::*;
use crate::vm::interface;
use crate::vm::io::types::*;

const PORT_BASE: Word = 0xD1;

pub struct SlowInts {
    pic: Handle<Pic>,

    count: [Word; 2],
}

impl SlowInts {
    const MASK_NMI_FLAG: Word = 0x8000;
    const NMI_NUM: u8 = 0;
    const INT_NUM: u8 = 3;

    pub fn new(pic: Handle<Pic>) -> Self {
        SlowInts { pic, count: [0, 0] }
    }

    fn process_halfcycle_register(&mut self, num: usize, int_num: u8) {
        if self.count[num] == 0 {
            return;
        }

        self.count[num] -= 1;

        if self.count[num] == 0 {
            interface::Pic::assert(&self.pic, interface::PicIrq::Num(int_num));
        }
    }
}

impl SinglePortDevice for SlowInts {
    fn reserved_port(&self) -> Word {
        PORT_BASE
    }

    fn write(&mut self, val: Word) -> HalfcycleCount {
        if val & SlowInts::MASK_NMI_FLAG == 0 {
            self.count[0] = (val & !SlowInts::MASK_NMI_FLAG) + 1;
        }

        if val & SlowInts::MASK_NMI_FLAG != 0 {
            self.count[1] = (val & !SlowInts::MASK_NMI_FLAG) + 1;
        }

        0
    }

    fn read(&mut self) -> (HalfcycleCount, Word) {
        panic!("read from SlowInts register");
    }

    fn process_halfcycle(&mut self, _: ClockedSignals) {
        self.process_halfcycle_register(0, SlowInts::NMI_NUM);
        self.process_halfcycle_register(1, SlowInts::INT_NUM);
    }
}
