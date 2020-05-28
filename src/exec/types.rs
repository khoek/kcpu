use crate::{assets, spec::types::hw::Word, vm};
use std::fmt::Display;
use std::marker::PhantomData;
use std::sync::mpsc::SendError;

// RUSTFIX remove all of the `Vec` stuff from the pollers

// RUSTFIX relocate?
#[derive(Debug, Clone)]
pub struct Snapshot {
    pub state: vm::State,
    pub timeout: bool,
    pub total_clocks: u64,
    pub real_ns_elapsed: u128,
}

impl Snapshot {
    pub fn of(vm: &vm::Instance, did_timeout: bool) -> Self {
        Self {
            state: vm.state(),
            timeout: did_timeout,
            total_clocks: vm.total_clocks(),
            real_ns_elapsed: vm.real_ns_elapsed(),
        }
    }

    pub fn to_effective_freq_megahertz(&self) -> f64 {
        ((self.total_clocks as f64) * 1000.0) / (self.real_ns_elapsed as f64)
    }
}

// RUSTFIX relocate?
#[derive(Debug)]
pub struct Vram(pub Box<[Word]>);

impl Vram {
    pub fn new<Ref: AsRef<[Word]>>(vram: Ref) -> Self {
        assert!(vram.as_ref().len() == 2 * vm::VIDEO_WIDTH * vm::VIDEO_HEIGHT);
        Vram(Box::from(vram.as_ref()))
    }
}

pub trait PipelineBuilder<Monitor> {
    type Backend: Backend;
    type Frontend: Frontend<
        Monitor,
        Command = <Self::Backend as Backend>::Command,
        Response = <Self::Backend as Backend>::Response,
    >;

    fn build(self) -> Pipeline<Monitor, Self::Frontend, Self::Backend>;
}

pub type AdaptBackendFn<B, NewB> = Box<dyn FnOnce(B) -> NewB + Send + 'static>;

pub trait Adaptor<
    Monitor,
    NewMonitor,
    F: Frontend<Monitor>,
    NewF: Frontend<NewMonitor>,
    B: Backend,
    NewB: Backend,
>
{
    // RUSTFIX remove this box once we can do `impl` in `trait`s.
    fn adapt(self, f: F) -> (NewF, AdaptBackendFn<B, NewB>);
}

pub struct Pipeline<Monitor, F, B>
where
    F: Frontend<Monitor, Command = B::Command, Response = B::Response>,
    B: Backend + 'static,
{
    // RUSTFIX why does this have to stick around?
    _marker: PhantomData<fn() -> Monitor>,

    frontend: F,
    // RUSTFIX remove this box once we can do `impl` in `trait`s.
    backend_new: Box<dyn FnOnce(vm::Instance<'static>) -> Result<B, B::Error> + 'static + Send>,
}

impl<Monitor, F, B> Pipeline<Monitor, F, B>
where
    F: Frontend<Monitor, Command = B::Command, Response = B::Response>,
    B: Backend + 'static,
{
    pub fn new(
        frontend: F,
        backend_new: impl FnOnce(vm::Instance<'static>) -> Result<B, B::Error> + 'static + Send,
    ) -> Self {
        Self {
            _marker: PhantomData,

            frontend,
            backend_new: Box::new(backend_new),
        }
    }

    pub fn adapt<NewMonitor, NewF, NewB, Ad: Adaptor<Monitor, NewMonitor, F, NewF, B, NewB>>(
        self,
        adaptor: Ad,
    ) -> Pipeline<NewMonitor, NewF, NewB>
    where
        NewF: Frontend<NewMonitor, Command = NewB::Command, Response = NewB::Response>,
        NewB: Backend<Error = B::Error>,
    {
        let (frontend, backend_adaptor) = adaptor.adapt(self.frontend);
        let backend_new = self.backend_new;
        Pipeline::new(frontend, move |vm| Ok(backend_adaptor(backend_new(vm)?)))
    }

    pub fn runner<'a, Output, E: EventLoop<Output, Monitor = Monitor>, PF: PollerFactory<B>>(
        self,
        poller_factory: PF,
        event_loop: E,
    ) -> Runner<'a, Output, PollerError<B::Error>>
    where
        F: 'a,
        E: 'a,
        PF: 'a,
        PF::Commander: 'static,
    {
        let backend_new = self.backend_new;
        let mut frontend = self.frontend;
        Runner::new(move |vm_new: Box<dyn VmNewFn>| {
            let (commander, poller) = poller_factory
                .poller(|| backend_new(vm_new()))
                .map_err(PollerError::Backend)?;
            // RUSTFIX make this impossible to call twice!!
            // RUSTFIX proper error handling!!!
            frontend
                .startup(Box::new(commander))
                .expect("Frontend startup error!");
            event_loop.run(poller, frontend)
        })
    }
}

pub trait VmNewFn: FnOnce() -> vm::Instance<'static> + Send + 'static {}

impl<T: FnOnce() -> vm::Instance<'static> + Send + 'static> VmNewFn for T {}

pub trait EventLoopRunnerFn<'a, Output, Error>:
    FnOnce(Box<dyn VmNewFn>) -> Result<Output, Error> + 'a
{
}

impl<'a, Output, Error, T: FnOnce(Box<dyn VmNewFn>) -> Result<Output, Error> + 'a>
    EventLoopRunnerFn<'a, Output, Error> for T
{
}

pub struct Runner<'a, Output: 'a, Error: 'a> {
    event_loop_run: Box<dyn EventLoopRunnerFn<'a, Output, Error>>,
}

// RUSTFIX remove this
static LOGLEVEL: crate::vm::LogLevel = crate::vm::LogLevel { internals: false };

impl<'a, Output: 'a, Error: 'a> Runner<'a, Output, Error> {
    pub fn new(evt_loop_run: impl EventLoopRunnerFn<'a, Output, Error>) -> Self {
        Self {
            event_loop_run: Box::new(evt_loop_run),
        }
    }

    pub fn run<VmN: VmNewFn>(self, vm_new: VmN) -> Result<Output, Error> {
        // RUSTFIX avoid this `Box`.
        (self.event_loop_run)(Box::new(vm_new))
    }

    pub fn run_with_binaries(
        self,
        bios_bin: Option<&[u8]>,
        prog_bin: Option<&[u8]>,
    ) -> Result<Output, Error> {
        let bios_bin = bios_bin.unwrap_or_else(|| assets::default_bios()).to_vec();
        let prog_bin = prog_bin.unwrap_or_else(|| assets::default_prog()).to_vec();

        self.run(move || vm::Instance::new(&LOGLEVEL, &bios_bin, &prog_bin))
    }

    pub fn map<NewOutput, F: FnOnce(Output) -> NewOutput + 'static>(
        self,
        f: F,
    ) -> Runner<'a, NewOutput, Error> {
        let evt_loop_run = self.event_loop_run;
        Runner::new(move |vm_new| evt_loop_run(vm_new).map(f))
    }

    pub fn map_err<NewError, F: FnOnce(Error) -> NewError + 'static>(
        self,
        f: F,
    ) -> Runner<'a, Output, NewError> {
        let evt_loop_run = self.event_loop_run;
        Runner::new(move |vm_new| evt_loop_run(vm_new).map_err(f))
    }
}

pub trait EventLoop<Output> {
    type Monitor;

    fn run<B: Backend, F: Frontend<Self::Monitor, Response = B::Response>, P: Poller<B>>(
        self,
        poller: P,
        frontend: F,
    ) -> Result<Output, PollerError<B::Error>>;
}

pub trait Frontend<Monitor> {
    // RUSTFIX Think about getting rid of this `'static`.
    type Command: 'static;
    type Response;

    fn startup(&mut self, cmds: Box<dyn Commander<Self::Command>>) -> Result<(), FrontendError>;

    fn process(&mut self, resp: Self::Response) -> Result<Monitor, FrontendError>;

    // RUSTFIX this is silly---startup isn't called by the event loop, but process+teardown are...
    // I think we also shouldn't have `teardown`---remove it!
    fn teardown(self);
}

#[derive(Debug)]
pub enum FrontendError {
    Nothing,
    Shutdown,
}

pub trait Backend {
    // RUSTFIX These are only neccesary if you want to use an asyncrhonous poller.
    // We could use an `AsyncBackend` trait to automatically detect when this is possible,
    // but eager checking of trait conditions in `where` blocks makes the syntax for this
    // absolutely horrible; I eagerly await implementation of the `chalk` solver!
    type Command: Send + 'static;
    type Response: Send + 'static;
    type Error: Send + 'static;

    fn process(&mut self, cmd: Self::Command) -> Result<Option<Self::Response>, Self::Error>;
}

pub trait Commander<Command> {
    fn send(&self, cmd: Command) -> Result<(), SendError<Command>>;
}

pub trait PollerFactory<B: Backend> {
    type Commander: Commander<B::Command>;
    type Poller: Poller<B>;

    fn poller<BN: FnOnce() -> Result<B, B::Error> + Send + 'static>(
        &self,
        b: BN,
    ) -> Result<(Self::Commander, Self::Poller), B::Error>;
}

pub trait Poller<B: Backend> {
    fn recv(&mut self) -> Result<B::Response, PollerError<B::Error>>;

    fn try_recv(&mut self) -> Result<Option<B::Response>, PollerError<B::Error>>;
}

#[derive(Debug)]
pub enum PollerError<BackendError> {
    Shutdown,
    Backend(BackendError),
}

impl<BackendError: std::error::Error> Display for PollerError<BackendError> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            PollerError::Shutdown => write!(f, "Shutdown"),
            PollerError::Backend(err) => write!(f, "Backend({})", err),
        }
    }
}

impl<BackendError: std::error::Error> std::error::Error for PollerError<BackendError> {}

impl<T> From<SendError<T>> for FrontendError {
    fn from(_: SendError<T>) -> Self {
        FrontendError::Shutdown
    }
}
