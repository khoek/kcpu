use super::assets;
use crate::assembler::disasm::{self, RelativePos, SteppingDisassembler};
use crate::vm::{Bank, BankType, DebugExecInfo, Instance, Logger, State};
use derive_more::Constructor;
use std::{
    io,
    io::{BufRead, Write},
};

#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Clone, Copy)]
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
            BreakMode::OnInst => dbg.is_true_inst_beginning(),
            BreakMode::OnUCReset => dbg.uc_reset,
            BreakMode::OnUInst => true,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum AbortAction {
    Stop,
    Resume,
    Prompt,
}

#[derive(Debug, Clone, Copy)]
pub struct ExecFlags {
    pub headless: bool,
    pub max_clocks: Option<u64>,
    pub mode: BreakMode,
    pub abort_action: AbortAction,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub flags: ExecFlags,

    pub print_marginals: bool,
    pub verbosity: Verbosity,
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

// RUSTFIX remove `cfg` from being passed directly, encapsulate better
fn debug_hook(
    vm: &Instance,
    state: &mut (ExecFlags, SteppingDisassembler),
) -> Result<Option<State>, disasm::Error> {
    let (flags, disasm) = state;

    let dbg = vm.get_debug_exec_info();

    if dbg.is_true_inst_beginning() {
        let (ctx, pos) = disasm.step(vm.iter_at_ip())?;
        if let RelativePos::AliasBoundary = pos {
            println!("----------------------------------");
            println!("\t{}", format!("{}", ctx).replace("\n", "\t"));
            println!("----------------------------------");
        }
    }

    // RUSTFIX how much does this hurt performance?
    if flags.mode.should_pause(dbg) {
        let prompt_msg = "[ENTER to step]";
        println!("{}", prompt_msg);
        io::stdout().flush().unwrap();

        std::io::stdin().lock().lines().next();

        println!("\r{}\r", " ".repeat(prompt_msg.len()));
        io::stdout().flush().unwrap();
    }

    // RUSTFIX this doesn't belong here anymore....
    // FIXME manual timeout check (for step mode), bit of a hack returning
    // a VM code like this...
    if flags
        .max_clocks
        .map(|mc| vm.get_total_clocks() >= mc)
        .unwrap_or(false)
    {
        return Ok(Some(State::Timeout));
    }

    Ok(None)
}

pub fn execute(
    cfg: Config,
    raw_bios: Option<&[u8]>,
    raw_prog: Option<&[u8]>,
) -> Result<Summary, disasm::Error> {
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

    // RUSTFIX I suppose we shouldn't really be reaching into the flags here... but, is this an optimization? (TEST!)
    let summary = match cfg.flags.mode {
        BreakMode::Noninteractive => execute_with_iterhook(&mut vm, cfg.flags, (), None),
        _ => execute_with_iterhook(
            &mut vm,
            cfg.flags,
            (cfg.flags, SteppingDisassembler::new()),
            Some(debug_hook),
        ),
    }?;

    if cfg.print_marginals {
        println!(
            "CPU Stop ({}), {} uinstructions executed taking {}ms, @{}MHz",
            summary.state,
            summary.total_clocks,
            (summary.real_ns_elapsed / 1000 / 1000),
            summary.to_effective_freq_megahertz()
        );
    }

    Ok(summary)
}

// RUSTFIX remove `cfg` from being passed directly, encapsulate better
fn execute_with_iterhook<IterState, Error>(
    vm: &mut Instance,
    flags: ExecFlags,
    iter_initial: IterState,
    iter_hook: Option<fn(&Instance, &mut IterState) -> Result<Option<State>, Error>>,
) -> Result<Summary, Error> {
    // RUSTFIX implement graphics
    // graphics::get_graphics().configure(self.headless);

    let mut iter_state = iter_initial;

    let end_state = loop {
        // RUSTFIX try optimize (remove branches, using closures)
        // this core loop AFTER we do the C++ comparison.
        let s = match flags.mode {
            BreakMode::Noninteractive => vm.run(flags.max_clocks),
            BreakMode::OnInst | BreakMode::OnUCReset => vm.step(),
            BreakMode::OnUInst => vm.ustep(),
        };

        match s {
            State::Running => (),
            State::Aborted => {
                match flags.abort_action {
                    AbortAction::Resume => (),
                    AbortAction::Stop => break None,
                    AbortAction::Prompt => {
                        print!("CPU Aborted, continue(y)? ");
                        io::stdout().flush().unwrap();

                        let c = std::io::stdin().lock().lines().next().unwrap().unwrap();
                        // RUSTFIX does this actually work?
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

        if let Some(iter_hook) = iter_hook {
            let res = iter_hook(&vm, &mut iter_state)?;
            if res.is_some() {
                break res;
            }
        }
    };

    let summary = Summary::new(
        end_state.unwrap_or_else(|| vm.get_state()),
        vm.get_total_clocks(),
        vm.get_real_ns_elapsed(),
    );

    Ok(summary)
}
