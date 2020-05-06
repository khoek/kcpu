use super::{
    assemble,
    execute::{self, AbortAction, BreakMode, Config, Summary, Verbosity},
};
use crate::assembler;
use crate::vm::State;
use colored::Colorize;
use derive_more::Constructor;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

#[derive(Constructor)]
struct UnitSrc {
    name: OsString,
    bios_src: Option<PathBuf>,
    prog_src: Option<PathBuf>,
}

#[derive(Constructor)]
struct UnitBin {
    bios_bin: Option<Vec<u8>>,
    prog_bin: Option<Vec<u8>>,
}

impl UnitSrc {
    fn assemble(&self) -> Result<UnitBin, assembler::Error> {
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

        Ok(UnitBin::new(bios_bin, prog_bin))
    }
}

impl UnitBin {
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
    suite_name: &OsString,
    suite_root_dir: &PathBuf,
    only_this: &Option<OsString>,
    max_clocks: Option<u64>,
) -> Result<bool, assembler::Error> {
    let mut suite_dir = suite_root_dir.clone();
    suite_dir.push(suite_name);
    let all_units = find_units(&suite_dir);

    let selected_units = match only_this {
        None => all_units,
        Some(only_this) => {
            // RUSTFIX report an error!
            vec![all_units
                .into_iter()
                .find(|unit| &unit.name == only_this)
                .unwrap()]
        }
    };

    Ok(run_units(max_clocks, &selected_units))
}

fn find_file_unit(path: &Path) -> Option<UnitSrc> {
    if !path.extension().map_or(false, |ext| ext == "ks") {
        return None;
    }

    Some(UnitSrc {
        name: path.file_stem().unwrap().to_owned(),
        prog_src: Some(PathBuf::from(path)),
        bios_src: None,
    })
}

fn path_exists_to_option<T: AsRef<Path>>(path: T) -> Option<T> {
    if path.as_ref().exists() {
        Some(path)
    } else {
        None
    }
}

fn find_dir_unit(path: &Path) -> Option<UnitSrc> {
    let bios_src = path_exists_to_option(path.to_owned().join("bios.ks"));
    let prog_src = path_exists_to_option(path.to_owned().join("prog.ks"));

    match (bios_src, prog_src) {
        (None, None) => None,
        (bios_src, prog_src) => Some(UnitSrc {
            name: path.file_stem().unwrap().to_owned(),
            bios_src,
            prog_src,
        }),
    }
}

fn find_units(suite_dir: &PathBuf) -> Vec<UnitSrc> {
    // RUSTFIX proper error handling
    suite_dir
        .read_dir()
        .unwrap()
        .filter_map(|d| {
            // RUSTFIX proper error handling
            let f = d.unwrap();
            let path = f.path();

            let typ = f.file_type().unwrap();
            if typ.is_file() {
                find_file_unit(&path)
            } else if typ.is_dir() {
                find_dir_unit(&path)
            } else {
                None
            }
        })
        .collect()

    // // FIXMEFIXMEFIXME
    // vec![
    //     fn_cook_fake_unit("add3_fam"),
    //     // fn_cook_fake_unit("auto"), ASM
    //     fn_cook_fake_unit("call_ret_2"),
    //     fn_cook_fake_unit("fibb"),
    //     // fn_cook_fake_unit("int_disable_2"), VM
    //     // fn_cook_fake_unit("int_multiple"), ASM
    //     // fn_cook_fake_unit("int_recursive"), ASM
    //     fn_cook_fake_unit("io_probe"),
    //     fn_cook_fake_unit("ldwo_fam"),
    //     fn_cook_fake_unit("mov_self"),
    //     fn_cook_fake_unit("primes"),
    //     fn_cook_fake_unit("pushpop"),
    //     fn_cook_fake_unit("stwo"),
    //     fn_cook_fake_unit("add3"),
    //     fn_cook_fake_unit("byte_ld"),
    //     fn_cook_fake_unit("enter_fr"),
    //     // fn_cook_fake_unit("flag_tui2nmi"), ASM
    //     // fn_cook_fake_unit("int_during_io"), ASM
    //     fn_cook_fake_unit("int_nmi"),
    //     fn_cook_fake_unit("int_simple"),
    //     fn_cook_fake_unit("io_uid"),
    //     fn_cook_fake_unit("ldwo"),
    //     fn_cook_fake_unit("nop"),
    //     // fn_cook_fake_unit("primes_nmispam"), ASM
    //     fn_cook_fake_unit("pushpop_rsp"),
    //     fn_cook_fake_unit("alu"),
    //     fn_cook_fake_unit("byte_st"),
    //     fn_cook_fake_unit("enter_leave"),
    //     // fn_cook_fake_unit("int_async"), ASM
    //     fn_cook_fake_unit("int_fastdeliv"),
    //     // fn_cook_fake_unit("int_nmi_no_eoi"), ASM
    //     fn_cook_fake_unit("int_stackcheck"),
    //     // fn_cook_fake_unit("io_video"), VM(unimplemented)
    //     fn_cook_fake_unit2("ljmp"),
    //     fn_cook_fake_unit("old_test"),
    //     fn_cook_fake_unit("pushpop_a"),
    //     fn_cook_fake_unit("simple"),
    //     fn_cook_fake_unit("alu_noflags"),
    //     fn_cook_fake_unit("call_ret_1"),
    //     // fn_cook_fake_unit("family"), ASM
    //     fn_cook_fake_unit("int_disable_1"),
    //     // fn_cook_fake_unit("int_ie_pushpop"), ASM
    //     fn_cook_fake_unit("int_nmi_no_rec"),
    //     fn_cook_fake_unit("io_latency"),
    //     fn_cook_fake_unit("jmp"),
    //     fn_cook_fake_unit("pushpop_fg"),
    //     fn_cook_fake_unit("stwo_fam"),
    // ]
}

fn run_units(max_clocks: Option<u64>, units: &[UnitSrc]) -> bool {
    let name_pad = units.iter().map(|unit| unit.name.len()).max().unwrap_or(0);

    println!("--------------------------------------------------------------");

    let passes: usize = units
        .iter()
        .enumerate()
        .map(|(num, unit)| run_unit(unit, num + 1, name_pad, max_clocks) as usize)
        .sum();
    let success = passes == units.len();

    println!("--------------------------------------------------------------");
    println!(
        "Suite Result: {}, {}/{} passes",
        if success {
            "SUCCESS".green()
        } else {
            "FAILED".red()
        },
        passes,
        units.len()
    );

    success
}

fn run_unit(src: &UnitSrc, num: usize, name_pad: usize, max_clocks: Option<u64>) -> bool {
    print!(
        "Unit {:2 }: {} {}",
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
                _ => panic!("internal unitrunner error: VM still running!"),
            }

            summary.state == State::Halted
        }
    }
}
