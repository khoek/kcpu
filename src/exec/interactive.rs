use super::types::{Commander, Frontend, FrontendError};

pub trait Interactor {
    type State;
    type Action;

    /// Called on startup.
    fn startup(&mut self);

    /// Returns `None` if the run should be stopped.
    fn handle(&mut self, state: &Self::State) -> Option<Self::Action>;

    fn teardown(self);
}

pub trait Engine {
    type Action;
    type Command: 'static; // RUSTFIX `'static`...
    type Response;
    type Monitor;

    fn startup(&mut self) -> Self::Command;

    /// Returns `None` if the run should be stopped.
    fn act(&mut self, resp: &Self::Response, action: Self::Action) -> Option<Self::Command>;

    fn report(&mut self, resp: Self::Response) -> Self::Monitor;
}

pub struct InteractiveFrontend<E: Engine, I> {
    cmds: Option<Box<dyn Commander<E::Command>>>,
    engine: E,
    interactor: I,
}

impl<E: Engine, I: Interactor<State = E::Response, Action = E::Action>> InteractiveFrontend<E, I> {
    pub fn new(engine: E, interactor: I) -> Self {
        InteractiveFrontend {
            cmds: None,
            engine,
            interactor,
        }
    }
}

impl<E: Engine, I: Interactor<State = E::Response, Action = E::Action>> Frontend<E::Monitor>
    for InteractiveFrontend<E, I>
{
    type Command = E::Command;
    type Response = E::Response;

    fn startup(&mut self, cmds: Box<dyn Commander<E::Command>>) -> Result<(), FrontendError> {
        self.interactor.startup();

        cmds.send(self.engine.startup())?;
        self.cmds = Some(cmds);

        Ok(())
    }

    fn process(&mut self, resp: E::Response) -> Result<E::Monitor, FrontendError> {
        let cmd = self
            .interactor
            .handle(&resp)
            .map(|action| self.engine.act(&resp, action))
            .flatten();

        match &self.cmds {
            None => return Err(FrontendError::Shutdown),
            Some(cmds) => {
                if let Some(cmd) = cmd {
                    cmds.send(cmd)?;
                } else {
                    self.cmds = None;
                }
            }
        };

        Ok(self.engine.report(resp))
    }

    fn teardown(self) {
        self.interactor.teardown();
    }
}
