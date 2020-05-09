use super::{
    assemble, assets,
    run::{
        debug::{self, BreakOn},
        execute::{self, AbortAction, Config, Summary, Verbosity},
    },
    suite,
};
use crate::assembler::disasm;
use std::ffi::OsString;
use std::{fmt::Display, path::PathBuf, str::FromStr};
use structopt::StructOpt;

#[cfg(windows)]
pub fn terminal_init() {
    ansi_term::enable_ansi_support().expect("Could enable terminal ANSI support");
}

#[cfg(not(windows))]
pub fn terminal_init() {}

#[derive(StructOpt, Debug)]
#[structopt(name = "kcpu")]
pub enum CommandRoot {
    Vm(SubcommandVm),
    Asm(SubcommandAsm),
    Run(SubcommandRun),
    Suite(SubcommandSuite),
}

#[derive(StructOpt, Debug)]
#[structopt(name = "kasm")]
pub struct SubcommandAsm {
    #[structopt(name = "in.ks", parse(from_os_str))]
    in_src: PathBuf,

    #[structopt(name = "out.kb", parse(from_os_str))]
    out_bin: Option<PathBuf>,
}

#[derive(StructOpt, Debug)]
struct VmOpts {
    #[structopt(short, long, name = "max-clocks")]
    max_clocks: Option<ClockLimit>,

    #[structopt(short, long)]
    verbose: bool,

    #[structopt(short, long)]
    debugger: bool,

    #[structopt(short, long)]
    headless: bool,
}

#[derive(StructOpt, Debug)]
#[structopt(name = "kcpu-vm")]
pub struct SubcommandVm {
    #[structopt(flatten)]
    vm_opts: VmOpts,

    #[structopt(name = "prog.kb", parse(from_os_str))]
    in_prog_bin: PathBuf,

    #[structopt(name = "bios.kb", parse(from_os_str))]
    in_bios_bin: Option<PathBuf>,
}

#[derive(StructOpt, Debug)]
pub struct SubcommandRun {
    #[structopt(flatten)]
    vm_opts: VmOpts,

    #[structopt(name = "prog.ks", parse(from_os_str))]
    in_prog_src: PathBuf,

    #[structopt(name = "bios.ks", parse(from_os_str))]
    in_bios_src: Option<PathBuf>,
}

#[derive(StructOpt, Debug)]
pub struct SubcommandSuite {
    #[structopt(name = "suite_name", parse(from_os_str))]
    suite_name: OsString,

    #[structopt(flatten)]
    opts: SuiteOpts,
}

#[derive(StructOpt, Debug)]
pub struct SuiteOpts {
    #[structopt(name = "suite/root/dir", parse(from_os_str))]
    suite_root_dir: Option<PathBuf>,

    #[structopt(short = "only", long, parse(from_os_str))]
    only: Option<OsString>,

    #[structopt(short, long, name = "max-clocks")]
    max_clocks: Option<ClockLimit>,
}

#[derive(Debug)]
pub struct ClockLimit(Option<u64>);

impl Display for ClockLimit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.0.map(|lim| lim.to_string()).as_deref().unwrap_or("∞")
        )
    }
}

impl FromStr for ClockLimit {
    type Err = <u64 as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.eq_ignore_ascii_case("unlimited") || s.eq_ignore_ascii_case("infinity") || s.eq("∞")
        {
            Ok(ClockLimit(None))
        } else {
            Ok(ClockLimit(Some(u64::from_str(s)?)))
        }
    }
}

impl Default for ClockLimit {
    fn default() -> Self {
        ClockLimit(Some(50_000_000))
    }
}

impl ClockLimit {
    pub fn into_option(self) -> Option<u64> {
        self.0
    }
}

pub fn root(cmd: CommandRoot) -> ! {
    // RUSTFIX proper error handling in all of these, instead of just calling `unwrap()`.
    match cmd {
        CommandRoot::Asm(scmd) => asm(scmd),
        CommandRoot::Vm(scmd) => vm(scmd),
        CommandRoot::Run(scmd) => run(scmd),
        CommandRoot::Suite(scmd) => suite(scmd),
    };
}

pub fn asm(cmd: SubcommandAsm) -> ! {
    // RUSTFIX proper error handling, instead of just calling `unwrap()`.
    let out_bin = assemble::assemble_path(&cmd.in_src).unwrap();

    let out_name = match cmd.out_bin {
        Some(outfile) => outfile,
        None => PathBuf::from(cmd.in_src.file_stem().unwrap())
            .with_extension(assets::DEFAULT_BINARY_EXT),
    };

    std::fs::write(out_name, out_bin).unwrap();

    std::process::exit(0);
}

pub fn vm(cmd: SubcommandVm) -> ! {
    // RUSTFIX proper error handling in all of these, instead of just calling `unwrap()`.
    let bios_bin = cmd
        .in_bios_bin
        .map(|bios_bin| std::fs::read(bios_bin).unwrap());
    let prog_bin = std::fs::read(cmd.in_prog_bin).unwrap();
    let summary = execute_prog_with_opts(bios_bin.as_deref(), &prog_bin, cmd.vm_opts).unwrap();

    std::process::exit(summary_to_exit_code(&summary));
}

pub fn run(cmd: SubcommandRun) -> ! {
    // RUSTFIX proper error handling, instead of just calling `unwrap()`.
    let bios_bin = cmd
        .in_bios_src
        .as_ref()
        .map(|path| assemble::assemble_path(path))
        .transpose()
        .unwrap();
    let prog_bin = assemble::assemble_path(&cmd.in_prog_src).unwrap();

    let summary = match execute_prog_with_opts(bios_bin.as_deref(), &prog_bin, cmd.vm_opts) {
        Ok(summary) => summary,
        Err(err) => panic!("Error: {}", err),
    };

    std::process::exit(summary_to_exit_code(&summary));
}

pub fn suite(cmd: SubcommandSuite) -> ! {
    // RUSTFIX proper error handling, instead of just calling `unwrap()`.
    let success = suite::run_suite(
        &cmd.suite_name,
        &cmd.opts
            .suite_root_dir
            .unwrap_or_else(assets::default_suite_dir),
        cmd.opts.only.as_ref(),
        cmd.opts.max_clocks.unwrap_or_default().into_option(),
    )
    .unwrap();

    std::process::exit(if success { 0 } else { 1 });
}

// RUSTFIX remove entirely once we move to proper error handling, so
// we don't even manage exit codes in this module.
fn summary_to_exit_code(summary: &Summary) -> i32 {
    match summary.state {
        crate::vm::State::Halted => 0,
        _ => 1,
    }
}

fn execute_prog_with_opts(
    bios_bin: Option<&[u8]>,
    prog_bin: &[u8],
    vm_opts: VmOpts,
) -> Result<Summary, disasm::Error> {
    execute::execute_with_hook(
        Config {
            headless: vm_opts.headless,
            max_clocks: vm_opts.max_clocks.unwrap_or_default().into_option(),
            abort_action: if vm_opts.debugger {
                AbortAction::Prompt
            } else {
                AbortAction::Stop
            },

            verbosity: if vm_opts.verbose {
                Verbosity::Disassemble
            } else {
                Verbosity::Silent
            },
            print_marginals: true,
        },
        bios_bin,
        Some(prog_bin),
        debug::hook(
            vm_opts.verbose,
            if vm_opts.debugger {
                Some(BreakOn::Inst)
            } else {
                None
            },
        ),
    )
}
