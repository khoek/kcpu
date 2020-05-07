use super::{
    assemble, assets,
    execute::{AbortAction, BreakMode, Config, ExecFlags, Summary, Verbosity},
    suite,
};
use crate::assembler::disasm;
use std::ffi::OsString;
use std::path::PathBuf;
use structopt::StructOpt;

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
    #[structopt(name = "SRC", parse(from_os_str))]
    in_src: PathBuf,

    #[structopt(name = "OUTBIN", parse(from_os_str))]
    out_bin: Option<PathBuf>,
}

#[derive(StructOpt, Debug)]
struct VmOpts {
    #[structopt(short = "mc", long)]
    max_clocks: Option<u64>,

    #[structopt(short, long)]
    disassemble: bool,

    #[structopt(short, long)]
    step: bool,

    #[structopt(short, long)]
    headless: bool,
}

#[derive(StructOpt, Debug)]
#[structopt(name = "kcpu-vm")]
pub struct SubcommandVm {
    #[structopt(flatten)]
    vm_opts: VmOpts,

    #[structopt(name = "PROGBIN", parse(from_os_str))]
    in_prog_bin: PathBuf,

    #[structopt(name = "BIOSBIN", parse(from_os_str))]
    in_bios_bin: Option<PathBuf>,
}

#[derive(StructOpt, Debug)]
pub struct SubcommandRun {
    #[structopt(flatten)]
    vm_opts: VmOpts,

    #[structopt(name = "PROGSRC", parse(from_os_str))]
    in_prog_src: PathBuf,

    #[structopt(name = "BIOSSRC", parse(from_os_str))]
    in_bios_src: Option<PathBuf>,
}

#[derive(StructOpt, Debug)]
pub struct SuiteOpts {
    #[structopt(name = "SUITEROOTDIR", parse(from_os_str))]
    suite_root_dir: Option<PathBuf>,

    #[structopt(short = "only", long, parse(from_os_str))]
    only: Option<OsString>,

    // RUSTFIX at the moment there is no way to specify "unlimited"
    // for suite runs.
    #[structopt(short = "mc", long, default_value = "50000000")]
    max_clocks: u64,
}

#[derive(StructOpt, Debug)]
pub struct SubcommandSuite {
    #[structopt(name = "SUITENAME", parse(from_os_str))]
    suite_name: OsString,

    #[structopt(flatten)]
    opts: SuiteOpts,
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

    // let summary = execute_prog_with_opts(bios_bin.as_deref(), &prog_bin, cmd.vm_opts).unwrap();
    let summary = match execute_prog_with_opts(bios_bin.as_deref(), &prog_bin, cmd.vm_opts) {
        Ok(summary) => summary,
        Err(disasm::Error::NoSuitableAlias(msg)) => {
            panic!("NoSuitableAlias: {} {}", msg.len(), msg.join(", "))
        }
        Err(err) => panic!("{:#?}", err),
    };

    std::process::exit(summary_to_exit_code(&summary));
}

pub fn suite(cmd: SubcommandSuite) -> ! {
    // RUSTFIX proper error handling, instead of just calling `unwrap()`.
    let success = suite::suite(
        &cmd.suite_name,
        &cmd.opts
            .suite_root_dir
            .unwrap_or_else(assets::get_default_suite_dir),
        &cmd.opts.only,
        Some(cmd.opts.max_clocks),
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
    super::execute::execute(
        Config {
            flags: ExecFlags {
                headless: vm_opts.headless,
                max_clocks: vm_opts.max_clocks,
                mode: if vm_opts.step {
                    BreakMode::OnInst
                } else {
                    BreakMode::Noninteractive
                },
                abort_action: if vm_opts.step {
                    AbortAction::Prompt
                } else {
                    AbortAction::Stop
                },
            },

            verbosity: if vm_opts.disassemble || vm_opts.step {
                Verbosity::Disassemble
            } else {
                Verbosity::Silent
            },
            print_marginals: true,
        },
        bios_bin,
        Some(prog_bin),
    )
}
