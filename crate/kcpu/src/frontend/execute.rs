use super::assets;
use crate::vm::{Bank, BankType, DebugExecInfo, Instance, Logger, State};
use derive_more::Constructor;
use std::{
    io,
    io::{BufRead, Write},
};

#[derive(Clone, Copy)]
pub enum Verbosity {
    Silent,
    MachineState,
    Disassemble,
    Custom(Logger),
}

impl Verbosity {
    fn to_logger(self) -> Logger {
        match self {
            Verbosity::Silent => Logger::silent(),
            Verbosity::MachineState => Logger::only_machine_state(),
            Verbosity::Disassemble => Logger::everything(),
            Verbosity::Custom(logger) => logger,
        }
    }
}

pub enum BreakMode {
    Noninteractive,
    OnInst,
    OnUCReset,
    OnUInst,
}

impl BreakMode {
    fn should_pause(&self, dbg: DebugExecInfo) -> bool {
        match self {
            BreakMode::Noninteractive => false,
            BreakMode::OnInst => dbg.uc_reset && !dbg.mask_active,
            BreakMode::OnUCReset => dbg.uc_reset,
            BreakMode::OnUInst => true,
        }
    }
}

pub enum AbortAction {
    Stop,
    Resume,
    Prompt,
}

pub struct Config {
    pub headless: bool,
    pub max_clocks: Option<u64>,

    pub verbosity: Verbosity,
    pub mode: BreakMode,
    pub abort_action: AbortAction,

    pub print_marginals: bool,
}

#[derive(Debug, Constructor)]
pub struct Summary {
    pub state: State,
    pub total_clocks: u64,
    pub real_ns_elapsed: u128,
}

impl Summary {
    pub fn to_effective_freq_megahertz(&self) -> f64 {
        ((self.total_clocks as f64) * 1000.0) / (self.real_ns_elapsed as f64)
    }
}

pub fn execute(cfg: Config, raw_bios: Option<&[u8]>, raw_prog: Option<&[u8]>) -> Summary {
    // RUSTFIX implement graphics
    // graphics::get_graphics().configure(self.headless);

    let bios = Bank::new(
        BankType::Bios,
        raw_bios
            .unwrap_or_else(|| assets::get_default_bios())
            .to_vec(),
    );
    let prog = Bank::new(
        BankType::Prog,
        raw_prog
            .unwrap_or_else(|| assets::get_default_prog())
            .to_vec(),
    );

    let logger = cfg.verbosity.to_logger();
    let mut vm = Instance::new(&logger, bios, prog);

    if cfg.print_marginals {
        println!("CPU Start");
    }

    let end_state = loop {
        let s = match cfg.mode {
            BreakMode::Noninteractive => vm.run(cfg.max_clocks),
            BreakMode::OnInst | BreakMode::OnUCReset => vm.step(),
            BreakMode::OnUInst => vm.ustep(),
        };

        match s {
            State::Running => (),
            State::Aborted => {
                match cfg.abort_action {
                    AbortAction::Stop => break None,
                    AbortAction::Resume => (),
                    AbortAction::Prompt => {
                        print!("CPU Aborted, continue(y)? ");
                        io::stdout().flush().unwrap();

                        let c = std::io::stdin().lock().lines().next().unwrap().unwrap();
                        if c == "n" || c == "N" {
                            println!("Stopping...");

                            vm.dump_registers();
                            break None;
                        }

                        println!("Continuing...");
                    }
                }

                vm.resume();
            }
            s => break Some(s),
        }

        if cfg.mode.should_pause(vm.get_debug_exec_info()) {
            let prompt_msg = "[ENTER to step]";
            println!("{}", prompt_msg);
            io::stdout().flush().unwrap();

            std::io::stdin().lock().lines().next();

            println!("\r{}\r", " ".repeat(prompt_msg.len()));
            io::stdout().flush().unwrap();
        }

        // FIXME manual timeout check (for step mode), bit of a hack returning
        // a VM code like this...
        if cfg
            .max_clocks
            .map(|mc| vm.get_total_clocks() >= mc)
            .unwrap_or(false)
        {
            break Some(State::Timeout);
        }
    };

    let summary = Summary::new(
        end_state.unwrap_or_else(|| vm.get_state()),
        vm.get_total_clocks(),
        vm.get_real_ns_elapsed(),
    );

    if cfg.print_marginals {
        println!(
            "CPU Stop ({}), {} uinstructions executed taking {}ms, @{}MHz",
            summary.state,
            summary.total_clocks,
            (summary.real_ns_elapsed / 1000 / 1000),
            summary.to_effective_freq_megahertz()
        );
    }

    summary
}
