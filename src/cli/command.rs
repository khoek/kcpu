use super::suite;
use crate::exec::{
    adaptor::{self, vram_access},
    event_loop::{headless, webgpu},
    interactor::console,
    pipeline::{self, debug::BreakOn},
    poller,
    types::{Backend, PipelineBuilder, Runner, Snapshot},
};
use crate::{assembler, assets};
use std::ffi::OsString;
use std::{
    fmt::{Debug, Display},
    path::{Path, PathBuf},
    str::FromStr,
};
use structopt::StructOpt;

#[cfg(windows)]
pub fn terminal_init() {
    ansi_term::enable_ansi_support().expect("Could enable terminal ANSI support");
}

#[cfg(not(windows))]
pub fn terminal_init() {}

pub fn assemble_path(path: &Path) -> Result<Vec<u8>, assembler::Error> {
    // RUSTFIX proper IO error handling
    let prog_src = std::fs::read_to_string(path).unwrap();
    assembler::assemble_bytes(&prog_src)
}

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
    headless: bool,

    #[structopt(short, long)]
    debugger: bool,

    // RUSTFIX what should this flag do when there is no debugger?
    #[structopt(short, long)]
    verbose: bool,
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

#[derive(Debug, Clone, Copy)]
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
    let out_bin = assemble_path(&cmd.in_src).unwrap();

    let out_name = match cmd.out_bin {
        Some(outfile) => outfile,
        None => PathBuf::from(cmd.in_src.file_stem().unwrap())
            .with_extension(assets::DEFAULT_BINARY_EXT),
    };

    std::fs::write(out_name, out_bin).unwrap();

    std::process::exit(0);
}

pub fn vm(cmd: SubcommandVm) -> ! {
    let bios_bin = cmd
        .in_bios_bin
        .map(|bios_bin| std::fs::read(bios_bin).unwrap());
    let prog_bin = std::fs::read(cmd.in_prog_bin).unwrap();

    // RUSTFIX proper error handling in all of these, instead of just calling `unwrap()`.
    let snap = run_prog_with_opts(bios_bin.as_deref(), &prog_bin, cmd.vm_opts).unwrap();

    std::process::exit(state_to_exit_code(snap.state));
}

pub fn run(cmd: SubcommandRun) -> ! {
    // RUSTFIX proper error handling, instead of just calling `unwrap()`.
    let bios_bin = cmd
        .in_bios_src
        .as_ref()
        .map(|path| assemble_path(path))
        .transpose()
        .unwrap();
    let prog_bin = assemble_path(&cmd.in_prog_src).unwrap();

    // RUSTFIX proper error handling in all of these, instead of just calling `unwrap()`.
    let snap = run_prog_with_opts(bios_bin.as_deref(), &prog_bin, cmd.vm_opts).unwrap();

    std::process::exit(state_to_exit_code(snap.state));
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
fn state_to_exit_code(state: crate::vm::State) -> i32 {
    match state {
        crate::vm::State::Halted => 0,
        _ => 1,
    }
}

fn run_prog_with_opts(
    bios_bin: Option<&[u8]>,
    prog_bin: &[u8],
    opts: VmOpts,
) -> Result<Snapshot, anyhow::Error> {
    let runner = if opts.debugger {
        build_runner(
            opts.headless,
            pipeline::Debug::new(console::DebugInteractor::new(BreakOn::Inst, opts.verbose)),
        )
    } else {
        build_runner(
            opts.headless,
            pipeline::Run::new(
                None,
                opts.max_clocks.unwrap_or_default().into_option(),
                console::RunInteractor,
            ),
        )
        .map_err(|_| unreachable!())
    };

    Ok(runner.run_with_binaries(bios_bin, Some(&prog_bin))?)
}

fn build_runner<'a, B, PB>(run_headless: bool, builder: PB) -> Runner<'a, Snapshot, B::Error>
where
    // RUSTFIX a `'static` on `B` has crept in here, where it doesn't need to be.
    B: Backend + vram_access::Provider + 'static,
    PB: PipelineBuilder<Snapshot, Backend = B> + 'static,
{
    if run_headless {
        PipelineBuilder::build(builder).runner(poller::BlockingFactory, headless::EventLoop)
    } else {
        PipelineBuilder::build(builder)
            .adapt(adaptor::VramAccess)
            .runner(
                poller::AsyncFactory::new(poller::ThreadSpawner),
                webgpu::EventLoop,
            )
    }
}
