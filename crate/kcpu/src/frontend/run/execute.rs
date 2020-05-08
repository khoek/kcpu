use super::super::assets;
use crate::vm::{Bank, BankType, Instance, LogLevel, State};
use derive_more::Constructor;
use std::convert::Infallible;
use std::io::{self, BufRead, Write};

// RUSTFIX probably remove this
#[derive(Debug, Clone, Copy)]
pub enum Verbosity {
    Silent,
    MachineState,
    Disassemble,
}

impl Verbosity {
    fn to_log_level(self) -> LogLevel {
        match self {
            Verbosity::Silent => LogLevel { internals: false },
            Verbosity::MachineState | Verbosity::Disassemble => LogLevel { internals: true },
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum AbortAction {
    Stop,
    Resume,
    Prompt,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub max_clocks: Option<u64>,
    pub abort_action: AbortAction,
    pub headless: bool,

    pub print_marginals: bool,
    pub verbosity: Verbosity,
}

#[derive(Debug, Constructor)]
pub struct Summary {
    pub state: State,
    pub timeout: bool,
    pub total_clocks: u64,
    pub real_ns_elapsed: u128,
}

impl Summary {
    pub fn to_effective_freq_megahertz(&self) -> f64 {
        ((self.total_clocks as f64) * 1000.0) / (self.real_ns_elapsed as f64)
    }
}

pub enum ExecutionMode {
    Continue,
    Stepping,
}

pub trait ExecutionHook<Error>: FnMut(&Instance) -> Result<ExecutionMode, Error> {}

impl<Error, F: FnMut(&Instance) -> Result<ExecutionMode, Error>> ExecutionHook<Error> for F {}

pub fn execute(cfg: Config, raw_bios: Option<&[u8]>, raw_prog: Option<&[u8]>) -> Summary {
    execute_with_hook(cfg, raw_bios, raw_prog, |_: &Instance| {
        Ok::<_, Infallible>(ExecutionMode::Continue)
    })
    .unwrap()
}

pub fn execute_with_hook<Error>(
    cfg: Config,
    raw_bios: Option<&[u8]>,
    raw_prog: Option<&[u8]>,
    mut hook: impl ExecutionHook<Error>,
) -> Result<Summary, Error> {
    let bios = Bank::new(
        BankType::Bios,
        raw_bios.unwrap_or_else(|| assets::default_bios()).to_vec(),
    );
    let prog = Bank::new(
        BankType::Prog,
        raw_prog.unwrap_or_else(|| assets::default_prog()).to_vec(),
    );

    // RUSTFIX implement graphics
    // graphics::graphics().configure(self.headless);

    if cfg.print_marginals {
        println!("CPU Start");
    }

    let log_level = cfg.verbosity.to_log_level();
    let mut vm = Instance::new(&log_level, bios, prog);

    let did_timeout: bool = loop {
        let clocks = match hook(&vm)? {
            ExecutionMode::Continue => cfg.max_clocks,
            ExecutionMode::Stepping => Some(1),
        };

        vm.run(clocks);

        match vm.state() {
            State::Running => (),
            State::Halted => break false,
            State::Aborted => {
                match cfg.abort_action {
                    AbortAction::Resume => (),
                    AbortAction::Stop => break false,
                    AbortAction::Prompt => {
                        print!("CPU Aborted, continue(y)? ");
                        io::stdout().flush().unwrap();

                        let c = std::io::stdin().lock().lines().next().unwrap().unwrap();
                        // RUSTFIX does this actually work?
                        if c == "n" || c == "N" {
                            println!("Stopping...");
                            println!("{}", vm);

                            break false;
                        }

                        println!("Continuing...");
                    }
                }

                vm.resume();
            }
        }

        if let Some(max_clocks) = cfg.max_clocks {
            if vm.total_clocks() >= max_clocks {
                break true;
            }
        }
    };

    let summary = Summary::new(
        vm.state(),
        did_timeout,
        vm.total_clocks(),
        vm.real_ns_elapsed(),
    );

    if cfg.print_marginals {
        println!(
            "CPU Stop (in state {}{}), {} μinstructions executed taking {}ms, @{}MHz",
            summary.state,
            if did_timeout { "Timeout/" } else { "" },
            summary.total_clocks,
            (summary.real_ns_elapsed / 1000 / 1000),
            summary.to_effective_freq_megahertz()
        );
    }

    Ok(summary)
}
