use super::{
    alu, ctl, interface, io, mem, reg,
    types::{BusState, LogLevel},
};
use crate::spec::types::hw::*;
use strum_macros::Display;

use super::{
    alu::Alu,
    ctl::{CBit, Ctl, SReg},
    io::Ioc,
    mem::Mem,
    reg::Reg,
};
use std::fmt::Display;

pub struct DebugExecInfo {
    pub uc_reset: bool,
    pub mask_active: bool,
}

impl DebugExecInfo {
    pub fn is_true_inst_beginning(&self) -> bool {
        self.uc_reset && !self.mask_active
    }
}

#[derive(Debug, Display, PartialEq, Eq)]
pub enum State {
    Running,
    Halted,
    Aborted,
}

pub struct Instance<'a> {
    log_level: &'a LogLevel,
    total_clocks: u64,
    real_ns_elapsed: u128,

    ctl: ctl::Ctl<'a>,
    reg: reg::Reg<'a>,
    mem: mem::Mem<'a>,
    alu: alu::Alu<'a>,
    ioc: io::Ioc<'a>,
}

impl<'a> Display for Instance<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let i = interface::Ctl::get_inst(&self.ctl);
        let ui = self.ctl.read_uinst_latch();

        writeln!(
            f,
            "IP/UC @ I/UI: {:#06X}/{:#06X} @ {:#06X}/{:#X} {}",
            self.ctl.regs[SReg::IP],
            self.ctl.regs[SReg::UC],
            i,
            ui,
            if self.ioc.get_pic().is_pint_active() {
                "(PINT)"
            } else {
                ""
            }
        )?;

        writeln!(f, "{}", self.ctl)?;
        writeln!(f, "{}", self.mem)?;
        writeln!(f, "{}", self.reg)?;
        writeln!(f, "{}", self.alu)?;
        write!(f, "{}", self.ioc)?;

        Ok(())
    }
}

impl<'a> Instance<'a> {
    pub fn new(log_level: &'a LogLevel, bios: mem::Bank, prog: mem::Bank) -> Self {
        Self {
            log_level,
            total_clocks: 0,
            real_ns_elapsed: 0,

            ctl: Ctl::new(&log_level),
            reg: Reg::new(&log_level),
            mem: Mem::new(&log_level, bios, prog),
            alu: Alu::new(&log_level),
            ioc: Ioc::new(&log_level),
        }
    }

    pub fn get_total_clocks(&self) -> u64 {
        self.total_clocks
    }

    pub fn get_real_ns_elapsed(&self) -> u128 {
        self.real_ns_elapsed
    }

    pub fn get_debug_exec_info(&self) -> DebugExecInfo {
        DebugExecInfo {
            uc_reset: self.ctl.regs[SReg::UC] == 0,
            mask_active: self.ctl.cbits[CBit::Instmask],
        }
    }

    pub fn get_state(&self) -> State {
        if self.ctl.cbits[CBit::Halted] {
            return if self.ctl.cbits[CBit::Aborted] {
                State::Aborted
            } else {
                State::Halted
            };
        }

        State::Running
    }

    pub fn ustep(&mut self) {
        let then = std::time::Instant::now();

        self.total_clocks += 1;

        if self.ctl.cbits[CBit::Halted] {
            panic!("cpu already halted!");
        }

        {
            let ui = self.ctl.read_uinst_latch();

            self.reg.offclock_pulse(ui);

            let mut state = BusState::new(self.log_level);

            self.ctl.clock_outputs(ui, &mut state);
            self.alu.clock_outputs(ui, &mut state);
            self.reg.clock_outputs(ui, &mut state, &self.ctl);
            self.mem.clock_outputs(ui, &mut state);
            self.ioc.clock_outputs(ui, &mut state, &self.ctl);

            self.mem.clock_connects(ui, &mut state);

            state.freeze();

            self.ioc.clock_inputs(ui, &state, &self.ctl);
            self.mem.clock_inputs(ui, &state);
            self.reg.clock_inputs(ui, &state, &self.ctl);
            self.alu.clock_inputs(ui, &state);
            self.ctl.clock_inputs(ui, &state, self.ioc.get_pic());

            if !self.ctl.cbits[CBit::Halted] {
                self.ctl.offclock_pulse(&self.ioc);
                self.ioc.offclock_pulse(&self.ctl);
            }
        }

        self.real_ns_elapsed += then.elapsed().as_nanos();
    }

    /// Returns `true` if the VM ran for `max_clock`s, or
    /// `false` if it was interrupted for another reason.
    pub fn run(&mut self, max_clocks: Option<u64>) -> bool {
        let mut clocks = 0;
        while !self.ctl.cbits[CBit::Halted] {
            if let Some(max_clocks) = max_clocks {
                if clocks >= max_clocks {
                    return true;
                }
            }

            self.ustep();
            clocks += 1;
        }

        return false;
    }

    pub fn resume(&mut self) {
        if self.get_state() != State::Aborted {
            panic!("cannot resume, cpu not aborted");
        }

        self.ctl.cbits[CBit::Halted] = false;
        self.ctl.cbits[CBit::Aborted] = false;
    }

    pub fn is_halted(&self) -> bool {
        self.ctl.cbits[CBit::Halted]
    }

    pub fn is_aborted(&self) -> bool {
        self.ctl.cbits[CBit::Aborted]
    }

    pub fn iter_at_ip(&self) -> impl Iterator<Item = Word> + '_ {
        self.mem.iter_at(false, self.ctl.regs[SReg::IP])
    }
}
