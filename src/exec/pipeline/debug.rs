use super::super::{
    adaptor::vram_access,
    interactive::{Engine as FrontendEngine, Interactor},
    types::{
        Backend as TraitBackend, Pipeline, PipelineBuilder as TraitPipelineBuilder, Snapshot, Vram,
    },
};
use crate::{
    assembler::disasm::{self, SteppingDisassembler},
    exec::interactive::InteractiveFrontend,
    spec::types::hw::Word,
    vm::{self, debug},
};

pub struct Builder<I: Interactor<State = DebugReport, Action = Command>> {
    interactor: I,
}

impl<I: Interactor<State = DebugReport, Action = Command>> Builder<I> {
    pub fn new(interactor: I) -> Self {
        Self { interactor }
    }
}

impl<I: Interactor<State = DebugReport, Action = Command> + 'static> TraitPipelineBuilder<Snapshot>
    for Builder<I>
{
    type Backend = Backend;
    type Frontend = InteractiveFrontend<FrontendCore, I>;

    fn build(self) -> Pipeline<Snapshot, Self::Frontend, Self::Backend> {
        Pipeline::new(
            InteractiveFrontend::new(FrontendCore, self.interactor),
            move |vm| Ok(Backend::new(vm)?),
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub enum BreakOn {
    Inst,
    UCReset,
    UInst,
    // RUSTFIX implement these, etc.
    Count(u64),
    Ip(Word),
    Return,
}

impl BreakOn {
    pub fn should_break(self, phase: debug::ExecPhase) -> bool {
        match self {
            BreakOn::Inst => phase == debug::ExecPhase::TrueInst(0),
            BreakOn::UCReset => phase.is_first_uop(),
            BreakOn::UInst => true,
            // RUSTFIX implement!
            _ => unimplemented!(),
        }
    }
}

#[derive(Debug)]
pub struct DebugReport {
    pub snap: Snapshot,
    pub phase: debug::ExecPhase,
    pub ctx: disasm::Context<'static>,
    pub vm_dump: String,
}

impl DebugReport {
    pub fn new(
        snap: Snapshot,
        phase: debug::ExecPhase,
        ctx: disasm::Context<'static>,
        vm_dump: String,
    ) -> Self {
        Self {
            snap,
            phase,
            ctx,
            vm_dump,
        }
    }
}

#[derive(Debug)]
pub enum Command {
    Report,
    Step(BreakOn),
    Resume,
}

// RUSTFIX 'static everywhere!
pub struct Backend {
    disasm: SteppingDisassembler<'static>,
    vm: vm::Instance<'static>,
}

// RUSTFIX 'static everywhere!
impl Backend {
    pub fn new(vm: vm::Instance<'static>) -> Result<Self, disasm::Error> {
        let disasm = SteppingDisassembler::new(&mut vm.iter_at_ip())?;
        Ok(Self { disasm, vm })
    }
}

impl vram_access::Provider for Backend {
    fn vram(&self) -> Vram {
        Vram::new(&*self.vm.video().vram())
    }
}

impl TraitBackend for Backend {
    type Command = Command;
    type Response = DebugReport;
    type Error = disasm::Error;

    fn process(&mut self, cmd: Command) -> Result<Option<DebugReport>, disasm::Error> {
        let timeout = match cmd {
            Command::Step(break_on) => loop {
                let timeout = self.vm.run(Some(1));
                let phase = self.vm.debug_exec_phase();

                if let debug::ExecPhase::Load(0) = phase {
                    self.disasm.step(self.vm.iter_at_ip())?;
                }

                if break_on.should_break(phase) {
                    break timeout;
                }
            },
            Command::Report => false,
            Command::Resume => {
                self.vm.resume();
                false
            }
        };

        Ok(Some(DebugReport::new(
            Snapshot::of(&self.vm, timeout),
            self.vm.debug_exec_phase(),
            self.disasm.context().clone(),
            self.vm.to_string(),
        )))
    }
}

pub struct FrontendCore;

impl FrontendEngine for FrontendCore {
    type Action = Command;
    type Command = Command;
    type Response = DebugReport;
    type Monitor = Snapshot;

    fn startup(&mut self) -> Command {
        Command::Report
    }

    fn act(&mut self, _: &DebugReport, cmd: Command) -> Option<Command> {
        Some(cmd)
    }

    fn report(&mut self, dbg: DebugReport) -> Snapshot {
        dbg.snap
    }
}
