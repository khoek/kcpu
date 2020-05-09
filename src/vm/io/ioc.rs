use super::super::{interface, types::*};
use super::dev::test::{jumpers::Jumpers, slow_ints::SlowInts, slow_regs::SlowRegs};
use super::dev::{pic::Pic, probe::Probe, uid::Uid};
use super::{
    manager::{Command, Manager},
    types::*,
};
use crate::spec::{defs::usig, types::hw::*};
use std::fmt::Display;

pub struct Ioc<'a> {
    manager: Manager<'a>,
    pic: Handle<Pic>,
}

impl<'a> Display for Ioc<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.manager)?;
        write!(f, "{}", self.pic)?;

        Ok(())
    }
}

impl<'a> Ioc<'a> {
    pub fn new(log_level: &'a LogLevel) -> Self {
        let mut manager = Manager::new(log_level);
        let pic = manager.add_device(Pic::new());

        manager.add_device(Uid::new());

        // // FIXME implement external memory?
        // // io_manager.register_io(id_external_memory);

        // io_manager.register_io(id_video);
        // // devices.push_back(<a serial thing? :D>); (this one would be disabled by default.)

        manager.add_device(Jumpers::new(pic.clone()));
        manager.add_device(SlowInts::new(pic.clone()));

        for delay in 0..5 as HalfcycleCount {
            manager.add_device(SlowRegs::new(delay));
        }

        // RUSTFIX unfortunately for now, must be last.
        manager.add_device(Probe::new(manager.registered_ports()));

        Ioc { manager, pic }
    }

    pub fn pic(&self) -> &dyn interface::Pic {
        &self.pic
    }

    pub fn clock_outputs(&mut self, ui: UInst, s: &mut BusState, ctl: &dyn interface::Ctl) {
        let cmd = if !usig::is_gctrl_nrm_io_readwrite(ui) {
            None
        } else if ui & usig::MASK_GCTRL_DIR == usig::GCTRL_CREG_I {
            Some(Command::Read {
                port: s.early_read(Bus::A),
            })
        } else {
            Some(Command::Write {
                port: s.early_read(Bus::A),
                value: s.early_read(Bus::B),
            })
        };

        self.manager.before_clock_outputs(cmd);

        self.manager
            .process_halfcycle(ClockedSignals::with_onclock(ctl));

        if let Some(Command::Read { port: _ }) = cmd {
            if self.manager.is_io_done() {
                if let Some(result) = self.manager.read_result() {
                    s.assign(Bus::B, result);
                }
            }
        }

        self.manager.after_clock_outputs(cmd);
    }

    pub fn clock_inputs(&mut self, _: UInst, _: &BusState, _: &dyn interface::Ctl) {}

    pub fn offclock_pulse(&mut self, ctl: &dyn interface::Ctl) {
        self.manager
            .process_halfcycle(ClockedSignals::with_offclock(ctl));
    }
}

impl<'a> interface::Ioc for Ioc<'a> {
    fn is_io_done(&self) -> bool {
        self.manager.is_io_done()
    }
}
