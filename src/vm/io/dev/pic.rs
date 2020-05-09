use super::super::types::*;
use crate::spec::types::hw::*;
use crate::vm::interface;
use bitintr::Tzcnt;
use std::fmt::Display;

// HARDWARE NOTE: Since the int_mask bits have to be high to enable an interrupt,
// the computer can start safely with interrupts disabled so long as this register
// is reset upon boot.

const PORT_BASE: Word = 0x01;

const MASK_CMD: Word = 0xC000;
const MASK_VAL: Word = 0x3FFF;
const SHIFT_CMD: Word = 14;

const CMD_EOI: Word = 0b01 << SHIFT_CMD;
const CMD_SET_MASK: Word = 0b10 << SHIFT_CMD;
// HARDWARE NOTE: In hardware we don't have to implement this one; and a CMD_CLEAR_PEND is probably sufficient.
// But it does allow us to raise interrupts from software, which is great for testing. (So maybe do it?)
const CMD_SET_PEND: Word = 0b11 << SHIFT_CMD;

// HARDWARE NOTE: ASK_NMIS assert the additional PNMI line when pending BUT DO NOT ignore the irq_mask field,
// else we could have NMIs being recursively handled.
const MASK_NMIS: Word = 0x0001;

pub struct Pic {
    aint_prev: bool,

    irq_mask: Word,
    irq_serv: Word,
    irq_pend: Word,
}

impl Display for Pic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[IRQs] mask => {:#06X} pend => {:#06X} serv => {:#06X} (ap:{})",
            self.irq_mask, self.irq_pend, self.irq_serv, self.aint_prev
        )
    }
}

impl Pic {
    pub fn new() -> Self {
        Pic {
            aint_prev: false,
            irq_mask: 0,
            irq_serv: 0,
            irq_pend: 0,
        }
    }

    /*
        PINT is higher if: 1) there is any interrupt pending and no interrupt
        is being serviced, or 2) the NMI is pending and is not currently being serviced.
    */
    fn is_pint_active(&self) -> bool {
        (self.irq_serv == 0 && self.next_pending_bit(false, false) != 0) || self.is_pnmi_active()
    }

    fn is_pnmi_active(&self) -> bool {
        (self.irq_serv & MASK_NMIS) == 0 && self.next_pending_bit(false, true) != 0
    }

    // HARDWARE NOTE: Let's only set an interrupt pending in the PIC on the rising edge of an interrupt line, so we can implement a hard "reset button" for example.
    fn assert(&mut self, irq: interface::PicIrq) {
        let bit = irq.to_num();

        // NOTE in practice this condition can arise, but for testing purposes in the simulator it likely indicates a bug.
        assert!(self.irq_serv & (1 << bit) == 0);
        self.irq_pend |= 1 << bit;
    }

    fn lowest_bit(bitmask: Word) -> Word {
        if bitmask != 0 {
            1 << bitmask.tzcnt()
        } else {
            0
        }
    }

    // HARDWARE NOTE: NMI enable jumper?
    // HARDWARE NOTE: This function ignores the masked bits!
    fn next_pending_bit(&self, expect_nonzero: bool, nmi_only: bool) -> Word {
        let irqs_masked: Word = self.irq_pend
            & ((self.irq_mask | MASK_NMIS) & (if nmi_only { MASK_NMIS } else { 0xFFFF }));
        if irqs_masked == 0 && expect_nonzero {
            panic!("irq_ACK with no active interrupt");
        }
        Pic::lowest_bit(irqs_masked)
    }
}

impl SinglePortDevice for Pic {
    fn reserved_port(&self) -> Word {
        PORT_BASE
    }

    fn write(&mut self, val: Word) -> HalfcycleCount {
        match val & MASK_CMD {
            CMD_EOI => {
                if self.irq_serv == 0 {
                    panic!("EOI with no active interrupt");
                }
                self.irq_serv &= !Pic::lowest_bit(self.irq_serv);
            }
            CMD_SET_MASK => {
                self.irq_mask = val & MASK_VAL;
            }
            CMD_SET_PEND => {
                self.irq_pend = val & MASK_VAL;
            }
            _ => panic!("unknown pic command"),
        }

        0
    }

    fn read(&mut self) -> (HalfcycleCount, Word) {
        (0, self.irq_serv)
    }

    // HARDWARE NOTE: This implementation is a bit of a hack since the PIC
    // handles aint asynchronously (at least I think that is how it will be implemented).
    fn process_halfcycle(&mut self, sigs: ClockedSignals) {
        //////////////////

        ///// RUSTFIX NOTE: To prevent cycles, iodevices simply can't retain references to big modules like Ctl.
        /////               Though, I suppose I can let the jumper iodev hold a reference to the Pic.
        /////               At least right now, this only means that the jumper iodev and the pic iodev need to read values from the Ctl.
        /////               So, the Ctl can just publish a snapshot of its values.
        /////               In general though, there is a general problem of two modules commuicating bidirectionally down channel, avoiding
        /////               duplicate references. I think the solution is to do some sort of message, passing thing where one person sets flags
        /////               in the pipe (e.g. that they want to assert some signal lines), and then the other module reads this out of the pipe.

        ////////////////////

        let aint_active = match sigs {
            ClockedSignals::OnClock(aint) => aint,
            ClockedSignals::OffClock(aint, _) => aint,
        };

        // Primitive rising edge detection
        if self.aint_prev == aint_active {
            return;
        }
        self.aint_prev = aint_active;

        if !self.aint_prev {
            return;
        }

        // HARDWARE NOTE: Implement this in hardware using daisy-chaining.
        let pending_bit = self.next_pending_bit(true, false);
        self.irq_serv |= pending_bit;
        // HARDWARE NOTE: It is important that we clear the pending bit, and record it in the in-service register
        // at this point, so that we can recieve further copies of that interrupt while it is being serviced.
        self.irq_pend &= !pending_bit;

        // Consequently, since irq_serv is now nonzero, the PINT line will go low.
        // FIXME HARDWARE NOTE: (thought of during rust port, probably a mistake: what if an NMI comes in right away after this? does it matter if PINT doesn't go low?)
    }
}

impl interface::Pic for Handle<Pic> {
    fn is_pint_active(&self) -> bool {
        self.rc.borrow().is_pint_active()
    }

    fn is_pnmi_active(&self) -> bool {
        self.rc.borrow().is_pnmi_active()
    }

    fn assert(&self, irq: interface::PicIrq) {
        self.rc.borrow_mut().assert(irq)
    }
}
