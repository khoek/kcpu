use super::execute::{ExecutionHook, ExecutionMode};
use crate::{
    assembler::disasm::{self, SteppingDisassembler},
    vm::{DebugExecInfo, Instance},
};
use io::{BufRead, Write};
use std::io;

#[derive(Debug, Clone, Copy)]
pub enum BreakOn {
    Inst,
    UCReset,
    UInst,
}

impl BreakOn {
    // RUSTFIX remove??
    fn should_pause(&self, dbg: DebugExecInfo) -> bool {
        match self {
            BreakOn::Inst => dbg.is_true_inst_beginning(),
            BreakOn::UCReset => dbg.uc_reset,
            BreakOn::UInst => true,
        }
    }
}

fn debug_hook(
    vm: &Instance,
    break_on: BreakOn,
    disasm: &mut Option<SteppingDisassembler>,
) -> Result<ExecutionMode, disasm::Error> {
    if let None = disasm {
        *disasm = Some(SteppingDisassembler::new(&mut vm.iter_at_ip())?);
    }
    let disasm = disasm.as_mut().unwrap();

    let dbg = vm.get_debug_exec_info();

    if dbg.is_true_inst_beginning() {
        disasm.step(vm.iter_at_ip())?;
    }

    println!("----------------------------------");
    println!("\t{}", format!("{}", disasm.context()).replace("\n", "\t"));
    println!("----------------------------------");
    println!("{}", vm);
    println!("----------------------------------");

    if break_on.should_pause(dbg) {
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

pub fn hook(break_on: Option<BreakOn>) -> impl ExecutionHook<disasm::Error> {
    move |vm: &Instance| match break_on {
        None => Ok(ExecutionMode::Continue),
        Some(break_on) => debug_hook(vm, break_on, &mut None),
    }
}
