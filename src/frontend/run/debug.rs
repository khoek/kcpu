use super::execute::{ExecutionHook, ExecutionMode};
use crate::{
    assembler::disasm::{self, SteppingDisassembler},
    vm::{debug::ExecPhase, Instance},
};
use ansi_term::{Color, Style};
use io::{BufRead, Write};
use std::io;

#[derive(Debug, Clone, Copy)]
pub enum BreakOn {
    Inst,
    UCReset,
    UInst,
}

impl BreakOn {
    fn should_pause(&self, phase: ExecPhase) -> bool {
        match self {
            BreakOn::Inst => phase == ExecPhase::TrueInst(0),
            BreakOn::UCReset => phase.is_first_uop(),
            BreakOn::UInst => true,
        }
    }
}

fn debug_hook(
    vm: &Instance,
    verbose: bool,
    break_on: BreakOn,
    disasm: &mut Option<SteppingDisassembler>,
) -> Result<ExecutionMode, disasm::Error> {
    let phase = vm.debug_exec_phase();

    if let None = disasm {
        *disasm = Some(SteppingDisassembler::new(&mut vm.iter_at_ip())?);
    }

    let disasm = disasm.as_mut().unwrap();
    if let ExecPhase::Load(0) = phase {
        disasm.step(vm.iter_at_ip())?;
    }
    let ctx = disasm.context();

    let (style, prefix) = match phase {
        ExecPhase::DispatchInterrupt(_) => (
            Color::White.bold().on(Color::Fixed(55)),
            String::from("DINT"),
        ),
        ExecPhase::IoWait(_) => (Color::White.bold().on(Color::Fixed(125)), String::from("IO")),
        ExecPhase::Load(uc) => (
            Color::White.bold().on(Color::Fixed(88)),
            format!(
                "L {}/{}",
                (uc as usize) + 1,
                ctx.current_blob()
                    .map(|blob| if blob.blob.inst.load_data { 4 } else { 2 })
                    .map(|i| i.to_string())
                    .unwrap_or(Style::new().blink().paint("???").to_string())
            ),
        ),
        ExecPhase::TrueInst(uc) => (
            if uc == 0 {
                Color::White.bold().on(Color::Fixed(22))
            } else {
                Color::White.bold().on(Color::Fixed(130))
            },
            String::from("TRUE"),
        ),
    };

    let inst = format!("{}", ctx.to_string().replace("\n", "\t"));

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
    if verbose {
        println!("{:-<50}", "");
        println!("{}", vm);
    }
    println!("{:-<50}", "");

    if break_on.should_pause(phase) {
        let prompt_msg = "[ENTER to step]";
        println!("{}", prompt_msg);
        io::stdout().flush().unwrap();

        std::io::stdin().lock().lines().next();

        println!("\r{}\r", " ".repeat(prompt_msg.len()));
        io::stdout().flush().unwrap();
    }

    // At the moment there is no real debugger, so you can't "resume" execution.
    Ok(ExecutionMode::Stepping)
}

pub fn hook(verbose: bool, break_on: Option<BreakOn>) -> impl ExecutionHook<disasm::Error> {
    let mut disasm = None;
    move |vm: &Instance| match break_on {
        None => Ok(ExecutionMode::Continue),
        Some(break_on) => debug_hook(vm, verbose, break_on, &mut disasm),
    }
}
