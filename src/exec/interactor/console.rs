use super::super::{
    interactive::Interactor as TraitInteractor,
    pipeline::debug::{BreakOn, Command, DebugReport},
};
use crate::{
    exec::types::Snapshot,
    vm::{self, debug::ExecPhase},
};
use ansi_term::{Color, Style};
use io::{BufRead, Write};
use std::io;

fn print_start_marginal() {
    println!("CPU Start");
}

fn print_end_marginal(snap: &Snapshot) {
    println!(
        "CPU Stop (in state {}{}), {}μops executed taking {}ms, @{: >5.2}MHz",
        if snap.timeout { "Timeout/" } else { "" },
        snap.state,
        snap.total_clocks,
        (snap.real_ns_elapsed / 1000 / 1000),
        snap.to_effective_freq_megahertz()
    );
}

pub struct RunInteractor;

impl TraitInteractor for RunInteractor {
    type State = Snapshot;
    type Action = ();

    fn startup(&mut self) {
        print_start_marginal();
    }

    fn handle(&mut self, snap: &Snapshot) -> Option<()> {
        if snap.state != vm::State::Running {
            print_end_marginal(snap);
        }

        Some(())
    }
}

pub struct DebugInteractor {
    break_on: BreakOn,
    verbose: bool,
}

impl DebugInteractor {
    pub fn new(break_on: BreakOn, verbose: bool) -> Self {
        Self { break_on, verbose }
    }
}

impl TraitInteractor for DebugInteractor {
    type State = DebugReport;
    type Action = Command;

    fn startup(&mut self) {
        print_start_marginal();
    }

    fn handle(&mut self, report: &DebugReport) -> Option<Command> {
        let DebugReport {
            snap,
            phase,
            ctx,
            vm_dump,
        } = report;

        match snap.state {
            vm::State::Running => (),
            vm::State::Halted => {
                print_end_marginal(snap);
                return None;
            }
            vm::State::Aborted => {
                print!("CPU Aborted, continue(y)? ");
                io::stdout().flush().unwrap();

                let c = std::io::stdin().lock().lines().next().unwrap().unwrap();
                // RUSTFIX does this actually work?
                if c == "n" || c == "N" {
                    println!("Stopping...");
                    println!("{}", vm_dump);

                    return None;
                }

                println!("Continuing...");
                return Some(Command::Resume);
            }
        }

        let (style, prefix) = match phase {
            ExecPhase::DispatchInterrupt(_) => (
                Color::White.bold().on(Color::Fixed(55)),
                String::from("DINT"),
            ),
            ExecPhase::IoWait(_) => (
                Color::White.bold().on(Color::Fixed(125)),
                String::from("IO"),
            ),
            ExecPhase::Load(uc) => (
                Color::White.bold().on(Color::Fixed(88)),
                format!(
                    "L {}/{}",
                    (*uc as usize) + 1,
                    ctx.current_blob()
                        .map(|blob| if blob.blob.inst.load_data { 4 } else { 2 })
                        .map(|i| i.to_string())
                        .unwrap_or_else(|| Style::new().blink().paint("???").to_string())
                ),
            ),
            ExecPhase::TrueInst(uc) => (
                if *uc == 0 {
                    Color::White.bold().on(Color::Fixed(22))
                } else {
                    Color::White.bold().on(Color::Fixed(130))
                },
                String::from("TRUE"),
            ),
        };

        let inst = ctx.to_string().replace("\n", "\t");

        let col_space = 16;
        println!("{:-<50}", "");
        println!(
            "{}",
            style.paint(format!(
                " {}{: <padding1$}{: <padding2$}",
                prefix,
                "",
                inst,
                padding1 = col_space - 1 - prefix.len(),
                padding2 = 50 - col_space,
            ))
        );
        if self.verbose {
            println!("{:-<50}", "");
            println!("{}", vm_dump);
        }
        println!("{:-<50}", "");

        if self.break_on.should_break(*phase) {
            let prompt_msg = "[ENTER to step]";
            println!("{}", prompt_msg);
            io::stdout().flush().unwrap();

            std::io::stdin().lock().lines().next();

            println!("\r{}\r", " ".repeat(prompt_msg.len()));
            io::stdout().flush().unwrap();
        }

        // Note that this is the condition *being passed to the backend*, not neccesarily
        // the mode in which we are in in the debugger frontend.
        Some(Command::Step(BreakOn::UInst))
    }
}
