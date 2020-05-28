use super::types::{AdaptBackendFn, Adaptor, Backend, Frontend, Vram};

// RUSTFIX define and use the notion of an `Adaptor`?

pub struct VramAccess;

impl<Monitor, F: Frontend<Monitor>, B: Backend + vram_access::Provider + 'static>
    Adaptor<
        Monitor,
        (Monitor, Vram),
        F,
        vram_access::Frontend<Monitor, F>,
        B,
        vram_access::Backend<B>,
    > for VramAccess
{
    fn adapt(
        self,
        frontend: F,
    ) -> (
        vram_access::Frontend<Monitor, F>,
        AdaptBackendFn<B, vram_access::Backend<B>>,
    ) {
        (
            vram_access::Frontend::new(frontend),
            Box::new(vram_access::Backend::new),
        )
    }
}

pub mod vram_access {
    use super::super::types::{
        Backend as TraitBackend, Commander as TraitCommander, Frontend as TraitFrontend,
        FrontendError, Vram,
    };
    use std::{marker::PhantomData, sync::mpsc::SendError};

    pub trait Provider {
        fn vram(&self) -> Vram;
    }

    pub struct Backend<Back: TraitBackend + Provider>(Back);

    impl<Back: TraitBackend + Provider> Backend<Back> {
        pub fn new(back: Back) -> Self {
            Self(back)
        }
    }

    pub struct Frontend<Monitor, Front: TraitFrontend<Monitor>>(
        Front,
        PhantomData<fn() -> Monitor>,
    );

    impl<Monitor, Front: TraitFrontend<Monitor>> Frontend<Monitor, Front> {
        pub fn new(front: Front) -> Self {
            Self(front, PhantomData)
        }
    }

    pub struct Commander<'a, Command>(bool, Box<dyn TraitCommander<CommandWrapper<Command>> + 'a>);

    impl<'a, Command> TraitCommander<Command> for Commander<'a, Command> {
        fn send(&self, cmd: Command) -> Result<(), SendError<Command>> {
            self.1
                .send(CommandWrapper {
                    cmd,
                    want_vram: self.0,
                })
                .map_err(|cmd| SendError(cmd.0.cmd))
        }
    }

    pub struct CommandWrapper<Command> {
        cmd: Command,
        want_vram: bool,
    }

    pub struct ResponseWrapper<Response> {
        resp: Response,
        vram: Option<Vram>,
    }

    impl<Back: TraitBackend + Provider> TraitBackend for Backend<Back> {
        type Command = CommandWrapper<Back::Command>;
        type Response = ResponseWrapper<Back::Response>;
        type Error = Back::Error;

        fn process(&mut self, cmd: Self::Command) -> Result<Option<Self::Response>, Self::Error> {
            let want_vram = cmd.want_vram;
            Ok(self.0.process(cmd.cmd)?.map(|resp| ResponseWrapper {
                resp,
                vram: if want_vram { Some(self.0.vram()) } else { None },
            }))
        }
    }

    /// More nuanced frontends for a `vram_access::Provider`-equipped backed are of course available,
    /// where the `Vram` is only requested some fraction of the time, but we do not yet
    /// require such a facility.
    ///
    /// Of course we could also implement `Frontend<Monitor>` (by never requesting vram,
    /// and unwrapping the lack of vram at the other endpoint), but you'll likely never
    /// want to do this: just use the original frontend/backend directly! But who knows if
    /// trait resolution woes will eventually call for it...
    impl<Monitor, Front: TraitFrontend<Monitor>> TraitFrontend<(Monitor, Vram)>
        for Frontend<Monitor, Front>
    {
        type Command = CommandWrapper<Front::Command>;
        type Response = ResponseWrapper<Front::Response>;

        fn startup(
            &mut self,
            cmds: Box<dyn TraitCommander<Self::Command>>,
        ) -> Result<(), FrontendError> {
            self.0.startup(Box::new(Commander(true, cmds)))
        }

        fn process(
            &mut self,
            resp: ResponseWrapper<Front::Response>,
        ) -> Result<(Monitor, Vram), FrontendError> {
            let ResponseWrapper { resp, vram } = resp;
            let mon = self.0.process(resp)?;
            Ok((mon, vram.expect("No vram!")))
        }

        fn teardown(self) {
            self.0.teardown()
        }
    }
}
