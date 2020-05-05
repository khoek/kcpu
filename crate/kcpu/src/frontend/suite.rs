use super::{
    assemble,
    execute::{self, AbortAction, BreakMode, Config, Summary, Verbosity},
};
use crate::assembler;
use crate::vm::State;
use colored::Colorize;
use derive_more::Constructor;
use std::ffi::OsString;
use std::path::PathBuf;

pub enum SuiteKind {
    Test,
    Bench,
}

#[derive(Constructor)]
struct CaseSrc {
    name: OsString,
    bios_src: Option<PathBuf>,
    prog_src: Option<PathBuf>,
}

#[derive(Constructor)]
struct CaseBin {
    bios_bin: Option<Vec<u8>>,
    prog_bin: Option<Vec<u8>>,
}

impl CaseSrc {
    fn assemble(&self) -> Result<CaseBin, assembler::Error> {
        let bios_bin = self
            .bios_src
            .as_deref()
            .map(assemble::assemble_path)
            .transpose()?;
        let prog_bin = self
            .prog_src
            .as_deref()
            .map(assemble::assemble_path)
            .transpose()?;

        Ok(CaseBin::new(bios_bin, prog_bin))
    }
}

impl CaseBin {
    fn execute(&self, max_clocks: Option<u64>) -> Summary {
        execute::execute(
            Config {
                headless: true,
                max_clocks,
                verbosity: Verbosity::Silent,
                mode: BreakMode::Noninteractive,
                abort_action: AbortAction::Stop,
                print_marginals: false,
            },
            self.bios_bin.as_deref(),
            self.prog_bin.as_deref(),
        )
    }
}

// RUSTFIX proper error handling
pub fn suite(
    kind: SuiteKind,
    suite_dir: &PathBuf,
    only_this: &Option<OsString>,
    max_clocks: Option<u64>,
) -> Result<bool, assembler::Error> {
    let all_cases = find_cases(suite_dir);

    let selected_tests = match only_this {
        None => all_cases,
        Some(only_this) => {
            // RUSTFIX report an error!
            vec![all_cases
                .into_iter()
                .find(|test| &test.name == only_this)
                .unwrap()]
        }
    };

    match kind {
        SuiteKind::Test => Ok(run_tests(max_clocks, &selected_tests)),
        SuiteKind::Bench => todo!(),
    }
}

fn find_cases(test_dir: &PathBuf) -> Vec<CaseSrc> {
    // RUSTFIX IMPLEMENT!!
    // todo!();

    // RUSTFIX TODO look for both "test_name.ks" and ("test_name.prog.ks", "test_name.bios.ks")
    //              Freak out if all three names are present?

    let fn_cook_fake_test = |name: &str| -> CaseSrc {
        let mut srcpath = (*test_dir).clone();
        srcpath.push(format!("{}{}", name, ".ks"));
        CaseSrc::new(OsString::from(name), None, Some(srcpath))
    };

    let fn_cook_fake_test2 = |name: &str| -> CaseSrc {
        let mut srcpath = (*test_dir).clone();
        srcpath.push(format!("{}{}", name, ".prog.ks"));
        let mut srcpath2 = (*test_dir).clone();
        srcpath2.push(format!("{}{}", name, ".bios.ks"));
        CaseSrc::new(OsString::from(name), Some(srcpath2), Some(srcpath))
    };

    // FIXMEFIXMEFIXME
    vec![
        fn_cook_fake_test("add3_fam"),
        // fn_cook_fake_test("auto"), ASM
        fn_cook_fake_test("call_ret_2"),
        fn_cook_fake_test("fibb"),
        // fn_cook_fake_test("int_disable_2"), VM
        // fn_cook_fake_test("int_multiple"), ASM
        // fn_cook_fake_test("int_recursive"), ASM
        fn_cook_fake_test("io_probe"), // FAILS BECAUSE io::dev::slow_regs is unimplemented
        fn_cook_fake_test("ldwo_fam"),
        fn_cook_fake_test("mov_self"),
        fn_cook_fake_test("primes"),
        fn_cook_fake_test("pushpop"),
        fn_cook_fake_test("stwo"),
        fn_cook_fake_test("add3"),
        fn_cook_fake_test("byte_ld"),
        fn_cook_fake_test("enter_fr"),
        // fn_cook_fake_test("flag_tui2nmi"), ASM
        // fn_cook_fake_test("int_during_io"), ASM
        fn_cook_fake_test("int_nmi"),
        fn_cook_fake_test("int_simple"),
        fn_cook_fake_test("io_uid"),
        fn_cook_fake_test("ldwo"),
        fn_cook_fake_test("nop"),
        // fn_cook_fake_test("primes_nmispam"), ASM
        fn_cook_fake_test("pushpop_rsp"),
        fn_cook_fake_test("alu"),
        fn_cook_fake_test("byte_st"),
        fn_cook_fake_test("enter_leave"),
        // fn_cook_fake_test("int_async"), ASM
        fn_cook_fake_test("int_fastdeliv"),
        // fn_cook_fake_test("int_nmi_no_eoi"), ASM
        fn_cook_fake_test("int_stackcheck"),
        // fn_cook_fake_test("io_video"), VM(unimplemented)
        fn_cook_fake_test2("ljmp"),
        fn_cook_fake_test("old_test"),
        fn_cook_fake_test("pushpop_a"),
        fn_cook_fake_test("simple"),
        fn_cook_fake_test("alu_noflags"),
        fn_cook_fake_test("call_ret_1"),
        // fn_cook_fake_test("family"), ASM
        fn_cook_fake_test("int_disable_1"),
        // fn_cook_fake_test("int_ie_pushpop"), ASM
        fn_cook_fake_test("int_nmi_no_rec"),
        fn_cook_fake_test("io_latency"),
        fn_cook_fake_test("jmp"),
        fn_cook_fake_test("pushpop_fg"),
        fn_cook_fake_test("stwo_fam"),
    ]
}

fn run_tests(max_clocks: Option<u64>, tests: &[CaseSrc]) -> bool {
    let name_pad = tests.iter().map(|test| test.name.len()).max().unwrap_or(0);

    println!("--------------------------------------------------------------");

    let passes: usize = tests
        .iter()
        .enumerate()
        .map(|(num, test)| run_test(test, num + 1, name_pad, max_clocks) as usize)
        .sum();
    let success = passes == tests.len();

    println!("--------------------------------------------------------------");
    println!(
        "Test Suite Result: {}, {}/{} passes",
        if success {
            "SUCCESS".green()
        } else {
            "FAILED".red()
        },
        passes,
        tests.len()
    );

    success
}

fn run_test(src: &CaseSrc, num: usize, name_pad: usize, max_clocks: Option<u64>) -> bool {
    print!(
        "Test {:2 }: {} {}",
        num,
        src.name
            .to_str()
            .unwrap_or(&format!("<invalid UTF-8>: {:?}", src.name)),
        " ".repeat(name_pad - src.name.len())
    );

    let summary = src.assemble().map(|bin| bin.execute(max_clocks));

    match summary {
        Err(err) => {
            // RUSTFIX don't use debug print here
            println!("{} ({:?})", "FAIL: ASSEMBLY ERROR".red(), err);

            false
        }
        Ok(summary) => {
            match summary.state {
                State::Halted => println!(
                    "{} {:7 }μops {: >4}ms  ({: >5.2}MHz)",
                    "PASS".green(),
                    summary.total_clocks,
                    summary.real_ns_elapsed / 1000 / 1000,
                    summary.to_effective_freq_megahertz(),
                ),
                State::Aborted => println!("{}", "FAIL: ABORTED".red()),
                State::Timeout => println!("{}", "FAIL: DETERMINISTIC TIMEOUT".red()),
                _ => panic!("internal testrunner error: VM still running!"),
            }

            summary.state == State::Halted
        }
    }
}
