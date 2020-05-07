use super::{types::*, *};
use crate::spec::types::hw::*;
use strum_macros::Display;

use super::{
    alu::Alu,
    ctl::{CBit, Ctl, SReg},
    io::Ioc,
    mem::Mem,
    reg::Reg,
};

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

    // Not a real state, just returned by `run()` when it times out
    Timeout,
}

pub struct Instance<'a> {
    logger: &'a Logger,
    total_clocks: u64,
    real_ns_elapsed: u128,

    ctl: ctl::Ctl<'a>,
    reg: reg::Reg<'a>,
    mem: mem::Mem<'a>,
    alu: alu::Alu<'a>,
    ioc: io::Ioc<'a>,
}

impl<'a> Instance<'a> {
    pub fn new(logger: &'a Logger, bios: mem::Bank, prog: mem::Bank) -> Self {
        Self {
            logger,
            total_clocks: 0,
            real_ns_elapsed: 0,

            ctl: Ctl::new(&logger),
            reg: Reg::new(&logger),
            mem: Mem::new(&logger, bios, prog),
            alu: Alu::new(&logger),
            ioc: Ioc::new(&logger),
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

    pub fn dump_registers(&self) {
        println!("---------------------");
        self.ctl.dump_registers();
        self.mem.dump_registers();
        self.reg.dump_registers();
        self.alu.dump_registers();
        self.ioc.dump_registers();
        println!();
    }

    pub fn disassemble_current(&self) {
        // RUSTFIX
        // let ip = ctl.reg[REG_IP] - ((ctl.cbits[CBit::Instmask] as u32) * (2 + 2 * (INST_GET_LOADDATA(self.ctl.regs[SReg::IR] as u32))));
        // codegen::bound_instruction b = codegen::disassemble_peek(ip, mem.get_bank(false));

        // std::stringstream ss;
        // ss << "(0x" << std::hex << std::uppercase << ip << ")  " << codegen::pretty_print(b) << std::endl;

        // println!("---------------------");
        // println!(ss.str());
        // if(!logger.dump_registers) {
        //     dump_registers();
        // }
    }

    pub fn print_debug_info(&self, i: Word, ui: UInst, pint: bool) {
        if self.logger.disassemble && !self.ctl.cbits[CBit::Instmask] {
            self.disassemble_current();
        }

        if self.logger.dump_bus {
            println!(
                "IP/UC @ I/UI: {:#04X}/{:#04X} @ {:#04X}/{:#X} {}",
                self.ctl.regs[SReg::IP],
                self.ctl.regs[SReg::UC],
                i,
                ui,
                if pint { "(PINT)" } else { "" }
            );
        }

        if self.logger.dump_registers {
            self.dump_registers();
        }
    }

    pub fn ustep(&mut self) -> State {
        let then = std::time::Instant::now();

        self.total_clocks += 1;

        if self.ctl.cbits[CBit::Halted] {
            panic!("cpu already halted!");
        }

        // This must be called before `ctl.get_uinst()`, as it sets up the value of the uinst latch.
        self.ctl.offclock_pulse(&self.ioc);
        self.ioc.offclock_pulse(&self.ctl);

        let i = interface::Ctl::get_inst(&self.ctl);
        let ui = self.ctl.get_uinst();

        self.print_debug_info(i, ui, self.ioc.get_pic().is_pint_active());

        self.reg.offclock_pulse(ui);

        let mut state = BusState::new(self.logger);

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

        self.real_ns_elapsed += then.elapsed().as_nanos();

        self.get_state()
    }

    pub fn step(&mut self) -> State {
        if self.ctl.cbits[CBit::Halted] {
            panic!("cpu already halted!");
        }

        loop {
            self.ustep();

            if self.ctl.regs[SReg::UC] == 0 || self.ctl.cbits[CBit::Halted] {
                break;
            }
        }

        self.get_state()
    }

    // RUSTFIX FIXME it is silly that all these functions just return `get_state()`, but it happens right now in order to return TIMEOUT in one case.
    pub fn run(&mut self, max_clocks: Option<u64>) -> State {
        let then = self.total_clocks;

        while !self.ctl.cbits[CBit::Halted] {
            if let Some(max_clocks) = max_clocks {
                if max_clocks < (self.total_clocks - then) {
                    return State::Timeout;
                }
            }

            self.step();
        }

        self.get_state()
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
