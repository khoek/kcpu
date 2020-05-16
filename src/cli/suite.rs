use crate::assembler;
use crate::{
    exec::{
        event_loop,
        interactor::noninteractive,
        pipeline, poller,
        types::{PipelineBuilder, Snapshot},
    },
    vm,
};
use ansi_term::Color::{Green, Red};
use derive_more::Constructor;
use event_loop::headless;
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
            .map(super::command::assemble_path)
            .transpose()?;
        let prog_bin = self
            .prog_src
            .as_deref()
            .map(super::command::assemble_path)
            .transpose()?;

        Ok(UnitBin::new(bios_bin, prog_bin))
    }
}

impl UnitBin {
    fn execute(self, max_clocks: Option<u64>) -> Snapshot {
        // RUSTFIX proper error handling!
        pipeline::Run::new(None, max_clocks, noninteractive::Interactor)
            .build()
            .runner(poller::BlockingFactory, headless::EventLoop)
            .run_with_binaries(self.bios_bin.as_deref(), self.prog_bin.as_deref())
            .unwrap()
    }
}

// RUSTFIX proper error handling
pub fn run_suite(
    suite_name: &OsString,
    suite_root_dir: &PathBuf,
    only_this: Option<&OsString>,
    max_clocks: Option<u64>,
) -> Result<bool, assembler::Error> {
    let mut suite_dir = suite_root_dir.clone();
    suite_dir.push(suite_name);
    let all_units = find_units(&suite_dir);

    let mut selected_units = match only_this {
        None => all_units,
        Some(only_this) => {
            // RUSTFIX report an error!
            vec![all_units
                .into_iter()
                .find(|unit| &unit.name == only_this)
                .unwrap()]
        }
    };

    selected_units.sort_unstable_by(|unit1, unit2| unit1.name.cmp(&unit2.name));

    Ok(run_units(
        &suite_name.to_string_lossy(),
        max_clocks,
        &selected_units,
    ))
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
}

// RUSTFIX run these in parallel using green threads, using `rayon`.
fn run_units(name: &str, max_clocks: Option<u64>, units: &[UnitSrc]) -> bool {
    let name_pad = units.iter().map(|unit| unit.name.len()).max().unwrap_or(0);

    println!("Running suite: '{}' ({} units)", name, units.len());
    println!("{:-<line_len$}", "", line_len = name_pad + 45);

    // RUSTFIX have a notion of "SynchronousRunner" and "ParallelRunner", the former printing
    // each test unit's name while its in progress, the latter printing whole lines as soon as
    // results come in.
    let passes = units
        .iter()
        .enumerate()
        .filter(|(num, unit)| run_unit(unit, num + 1, name_pad, max_clocks))
        .count();
    let success = passes == units.len();

    println!("{:-<line_len$}", "", line_len = name_pad + 45);
    println!(
        "Suite Result: {}, {}/{} passes",
        if success {
            Green.bold().paint("SUCCESS")
        } else {
            Red.bold().paint("FAILED")
        },
        passes,
        units.len()
    );

    success
}

fn run_unit(src: &UnitSrc, num: usize, name_pad: usize, max_clocks: Option<u64>) -> bool {
    let summary = src.assemble().map(|bin| bin.execute(max_clocks));

    let (success, msg) = match summary {
        Err(err) => (
            false,
            format!(
                "{}:\n\t{}",
                Red.bold().paint("FAIL: ASSEMBLY ERROR"),
                err.to_string().replace("\n", "\n\t")
            ),
        ),
        Ok(summary) => {
            let msg = match (summary.timeout, &summary.state) {
                (true, _) => format!(
                    "{} after {}μops ({}ms)",
                    Red.bold().paint("FAIL: DETERMINISTIC TIMEOUT"),
                    summary.total_clocks,
                    summary.real_ns_elapsed / 1000 / 1000
                ),
                (false, vm::State::Halted) => format!(
                    "{} {:7 }μops {: >4}ms  ({: >5.2}MHz)",
                    Green.bold().paint("PASS"),
                    summary.total_clocks,
                    summary.real_ns_elapsed / 1000 / 1000,
                    summary.to_effective_freq_megahertz(),
                ),
                (false, vm::State::Aborted) => format!("{}", Red.bold().paint("FAIL: ABORTED")),
                (false, vm::State::Running) => {
                    panic!("internal unit runner error: VM still running!")
                }
            };

            (summary.state == vm::State::Halted, msg)
        }
    };

    println!(
        "Unit {:2 }: {} {}{}",
        num,
        src.name
            .to_str()
            .unwrap_or(&format!("<invalid UTF-8>: {:?}", src.name)),
        " ".repeat(name_pad - src.name.len()),
        msg
    );

    success
}
