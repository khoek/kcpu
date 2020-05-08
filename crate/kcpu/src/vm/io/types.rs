use super::super::interface;
use crate::spec::types::hw::*;
use std::cell::RefCell;
use std::{fmt::Display, rc::Rc};

pub(in crate::vm::io) type HalfcycleCount = u32;

#[derive(Clone, Copy)]
pub enum ClockedSignals {
    OnClock(/* is_aint_active */ bool),
    OffClock(
        /* is_aint_active */ bool,
        /* is_tui_active */ bool,
    ),
}

impl ClockedSignals {
    pub fn with_onclock(ctl: &dyn interface::Ctl) -> ClockedSignals {
        ClockedSignals::OnClock(ctl.is_aint_active())
    }

    pub fn with_offclock(ctl: &dyn interface::Ctl) -> ClockedSignals {
        ClockedSignals::OffClock(ctl.is_aint_active(), ctl.is_tui_active())
    }
}

// RUSTFIX implement
// RUSTFIX better API using rusts enums/etc.?
pub trait Device {
    // RUSTFIX move this to an associated constant (in another trait, like `PortAddressed`, and make `add_device` accept something which is both `Device` and `PortAddressed`)
    fn get_reserved_ports(&self) -> Vec<Word>;

    fn write(&mut self, port: Word, val: Word) -> HalfcycleCount;
    fn read(&mut self, port: Word) -> (HalfcycleCount, Word);

    /* We intentionally do not encode the `offclock` state into the enum */
    fn process_halfcycle(&mut self, sigs: ClockedSignals);
}

pub trait SinglePortDevice {
    // RUSTFIX move this to an associated constant once we move `get_reserved_ports()`
    fn get_reserved_port(&self) -> Word;

    fn write(&mut self, val: Word) -> HalfcycleCount;
    fn read(&mut self) -> (HalfcycleCount, Word);
    fn process_halfcycle(&mut self, sigs: ClockedSignals);
}

impl<T: SinglePortDevice> Device for T {
    // RUSTFIX move this to an associated constant (in another trait, like `PortAddressed`, and make `add_device` accept something which is both `Device` and `PortAddressed`)
    fn get_reserved_ports(&self) -> Vec<Word> {
        vec![self.get_reserved_port()]
    }

    fn write(&mut self, port: Word, val: Word) -> HalfcycleCount {
        assert_eq!(port, self.get_reserved_port());
        self.write(val)
    }

    fn read(&mut self, port: Word) -> (HalfcycleCount, Word) {
        assert_eq!(port, self.get_reserved_port());
        self.read()
    }

    fn process_halfcycle(&mut self, sigs: ClockedSignals) {
        self.process_halfcycle(sigs)
    }
}

// RUSTFIX macro-ify this concept?
pub struct Handle<T: Device + ?Sized> {
    pub rc: Rc<RefCell<T>>,
}

impl<T: Device + ?Sized + Display> Display for Handle<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.rc.borrow())
    }
}

impl<T: Device + ?Sized> Handle<T> {
    pub fn new(dev: T) -> Handle<T>
    where
        T: Sized,
    {
        Handle {
            rc: Rc::new(RefCell::new(dev)),
        }
    }

    pub fn clone(&self) -> Self {
        Handle {
            rc: self.rc.clone(),
        }
    }

    /* RUSTFIX why does this need `Sized`? */
    pub fn forget<'a>(&self) -> Handle<dyn Device + 'a>
    where
        T: 'a + Sized,
    {
        Handle {
            rc: self.rc.clone(),
        }
    }
}

impl<T: Device + ?Sized> Handle<T> {
    pub(super) fn get_reserved_ports(&self) -> Vec<Word> {
        self.rc.borrow().get_reserved_ports()
    }

    pub(super) fn write(&self, port: Word, val: Word) -> HalfcycleCount {
        self.rc.borrow_mut().write(port, val)
    }

    pub(super) fn read(&self, port: Word) -> (HalfcycleCount, Word) {
        self.rc.borrow_mut().read(port)
    }

    pub(super) fn process_halfcycle(&self, sigs: ClockedSignals) {
        self.rc.borrow_mut().process_halfcycle(sigs)
    }
}
