use super::ctl::{CBit, SReg};
use super::{
    alu, ctl, interface, io, mem, reg,
    types::{BusState, LogLevel},
};
use crate::spec::types::hw::{UCVal, Word};
use std::fmt::Display;
use strum_macros::Display;

pub mod debug {
    use crate::spec::types::hw::UCVal;

    #[derive(Debug, PartialEq, Eq)]
    pub enum ExecPhase {
        TrueInst(UCVal),
        Load(UCVal),
        DispatchInterrupt(UCVal),
        IoWait(bool),
    }

    impl ExecPhase {
        pub fn is_first_uop(&self) -> bool {
            match self {
                ExecPhase::TrueInst(0) => true,
                ExecPhase::Load(0) => true,
                ExecPhase::DispatchInterrupt(0) => true,
                ExecPhase::IoWait(true) => true,
                _ => false,
            }
        }
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
        let i = interface::Ctl::inst(&self.ctl);
        let ui = self.ctl.read_uinst_latch();

        writeln!(
            f,
            "IP/UC @ I/UI: {:#06X}/{:#04X} @ {:#06X}/{:#010X} {}",
            self.ctl.regs[SReg::IP],
            self.ctl.regs[SReg::UC],
            i,
            ui,
            if self.ioc.pic().is_pint_active() {
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

            ctl: ctl::Ctl::new(&log_level),
            reg: reg::Reg::new(&log_level),
            mem: mem::Mem::new(&log_level, bios, prog),
            alu: alu::Alu::new(&log_level),
            ioc: io::Ioc::new(&log_level),
        }
    }

    pub fn total_clocks(&self) -> u64 {
        self.total_clocks
    }

    pub fn real_ns_elapsed(&self) -> u128 {
        self.real_ns_elapsed
    }

    pub fn debug_exec_phase(&self) -> debug::ExecPhase {
        let uc = self.ctl.regs[SReg::UC] as UCVal;

        if self.ctl.cbits[CBit::IoWait] {
            // RUSTFIX/HARDWARE NOTE Is it really true that `InstMask` is low on the first
            // IoWait lock and then high on successive clocks?
            return debug::ExecPhase::IoWait(!self.ctl.cbits[CBit::Instmask]);
        } else if self.ctl.cbits[CBit::Instmask] {
            if self.ctl.cbits[CBit::PintLatch] {
                return debug::ExecPhase::DispatchInterrupt(uc);
            } else {
                return debug::ExecPhase::Load(uc);
            }
        }

        debug::ExecPhase::TrueInst(uc)
    }

    pub fn state(&self) -> State {
        if self.ctl.cbits[CBit::Halted] {
            return if self.ctl.cbits[CBit::Aborted] {
                State::Aborted
            } else {
                State::Halted
            };
        }

        State::Running
    }

    fn ustep_untimed(&mut self) {
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
            self.ctl.clock_inputs(ui, &state, self.ioc.pic());

            if !self.ctl.cbits[CBit::Halted] {
                self.ctl.offclock_pulse(&self.ioc);
                self.ioc.offclock_pulse(&self.ctl);
            }
        }
    }

    /// Returns `true` if the VM ran for `max_clock`s, or
    /// `false` if it was interrupted for another reason.
    fn run_untimed(&mut self, max_clocks: Option<u64>) -> bool {
        let mut clocks = 0;
        while !self.ctl.cbits[CBit::Halted] {
            if let Some(max_clocks) = max_clocks {
                if clocks >= max_clocks {
                    return true;
                }
            }

            self.ustep_untimed();
            clocks += 1;
        }

        false
    }

    /// Returns `true` if the VM ran for `max_clock`s, or
    /// `false` if it was interrupted for another reason.
    pub fn run(&mut self, max_clocks: Option<u64>) -> bool {
        let then = std::time::Instant::now();
        let ret = self.run_untimed(max_clocks);
        self.real_ns_elapsed += then.elapsed().as_nanos();
        ret
    }

    pub fn resume(&mut self) {
        if self.state() != State::Aborted {
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
