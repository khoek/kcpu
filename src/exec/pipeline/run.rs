use super::super::{
    adaptor::vram_access,
    interactive::{Engine as FrontendEngine, Interactor},
    types::{
        Backend as TraitBackend, Pipeline, PipelineBuilder as TraitPipelineBuilder, Snapshot, Vram,
    },
};
use crate::{exec::interactive::InteractiveFrontend, vm};
use std::convert::Infallible;

pub struct Builder<I: Interactor<State = Snapshot, Action = ()>> {
    quantum: Option<u64>,
    max_clocks: Option<u64>,
    interactor: I,
}

impl<I: Interactor<State = Snapshot, Action = ()>> Builder<I> {
    pub fn new(quantum: Option<u64>, max_clocks: Option<u64>, interactor: I) -> Self {
        Self {
            quantum,
            max_clocks,
            interactor,
        }
    }
}

impl<I: Interactor<State = Snapshot, Action = ()> + 'static> TraitPipelineBuilder<Snapshot>
    for Builder<I>
{
    type Backend = Backend;
    type Frontend = InteractiveFrontend<FrontendCore, I>;

    fn build(self) -> Pipeline<Snapshot, Self::Frontend, Self::Backend> {
        Pipeline::new(
            InteractiveFrontend::new(
                FrontendCore::new(self.quantum, self.max_clocks),
                self.interactor,
            ),
            move |vm| Ok(Backend::new(vm)),
        )
    }
}

#[derive(Debug)]
pub enum Command {
    RunQuantum(Option<u64>),
}

pub struct Backend {
    // RUSTFIX remove 'static
    vm: vm::Instance<'static>,
}

impl Backend {
    pub fn new(vm: vm::Instance<'static>) -> Self {
        Self { vm }
    }
}

impl vram_access::Provider for Backend {
    fn vram(&self) -> Vram {
        Vram::new(&*self.vm.video().vram())
    }
}

impl<'a> TraitBackend for Backend {
    type Command = Command;
    type Response = Snapshot;
    type Error = Infallible;

    fn process(&mut self, cmd: Command) -> Result<Option<Snapshot>, Infallible> {
        Ok(Some(match cmd {
            Command::RunQuantum(quantum) => {
                let timeout = self.vm.run(quantum);
                Snapshot::of(&self.vm, timeout)
            }
        }))
    }
}

pub struct FrontendCore {
    quantum: Option<u64>,
    max_clocks: Option<u64>,
}

impl FrontendCore {
    pub fn new(mut quantum: Option<u64>, max_clocks: Option<u64>) -> Self {
        assert!(quantum.map(|quantum| quantum != 0).unwrap_or(true));

        if quantum.is_none() {
            quantum = max_clocks;
        }

        Self {
            quantum,
            max_clocks,
        }
    }
}

impl FrontendEngine for FrontendCore {
    type Action = ();
    type Command = Command;
    type Response = Snapshot;
    type Monitor = Snapshot;

    fn startup(&mut self) -> Command {
        Command::RunQuantum(self.quantum)
    }

    fn act(&mut self, snap: &Snapshot, _: ()) -> Option<Command> {
        let timeout = self
            .max_clocks
            .map(|max_clocks| max_clocks <= snap.total_clocks)
            .unwrap_or(false);
        if timeout || snap.state != vm::State::Running {
            None
        } else {
            Some(Command::RunQuantum(self.quantum))
        }
    }

    fn report(&mut self, resp: Snapshot) -> Snapshot {
        resp
    }
}
