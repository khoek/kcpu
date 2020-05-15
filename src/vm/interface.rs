use crate::spec::types::hw::*;
use std::cell::Ref;

/*
    Rust elegantly captures the "out" and "in" functions of these traits
    by requring immutable or mutable self references in order to invoke
    the repsective member functions.
*/

// RUSTFIX bring this trait into use, IU decoder (and other decoding that we have decided to do off-module?)
pub trait Dcd {
    // fn ius(IU) -> PReg;
}

pub trait Ctl {
    /*
        HARDWARE NOTE: the first boolean below is morally
        neccesary, but as of writing we never latch pint unless the
        INSTMASK is going high, and the pint latch is always cleared
        by the time INSTMASK is cleared.

        (So, at least right now, it can be safely commented.)
    */
    fn is_aint_active(&self) -> bool;

    /*
        "True μinstruction". High on the falling edge before
        a clock where a uinst which is part of a "true instruction",
        i.e. not an instruction fetch or interrupt handling.

        HARDWARE NOTE: This signal should only be inspected when
        the clock is going LOW.
    */
    fn is_tui_active(&self) -> bool;

    // RUSTFIX Implement IU decoding (move this out of Reg), so that we can remove `inst()`,
    fn inst(&self) -> Word;
}

pub trait Ioc {
    fn is_io_done(&self) -> bool;
}

pub enum PicIrq {
    // Convenience constant for asserting the NMI (irq=0).
    Nmi,
    Num(u8),
}

impl PicIrq {
    pub fn to_num(&self) -> u8 {
        match self {
            PicIrq::Nmi => 0,
            PicIrq::Num(n) => *n,
        }
    }
}

pub trait Pic {
    fn is_pint_active(&self) -> bool;
    fn is_pnmi_active(&self) -> bool;

    fn assert(&self, irq: PicIrq);
}

pub const VIDEO_WIDTH: usize = 160;
pub const VIDEO_HEIGHT: usize = 120;

pub trait Video {
    fn vram(&self) -> Ref<[Word]>;
}
