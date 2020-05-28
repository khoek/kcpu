use crate::exec::types::{
    Backend, EventLoop as TraitEventLoop, Frontend, FrontendError, Poller, PollerError,
};

pub struct EventLoop;

impl<Monitor> TraitEventLoop<Monitor> for EventLoop {
    type Monitor = Monitor;

    fn run<B: Backend, F: Frontend<Monitor, Response = B::Response>, P: Poller<B>>(
        self,
        mut poller: P,
        mut frontend: F,
    ) -> Result<Monitor, PollerError<B::Error>> {
        let mut last_monitor: Option<Monitor> = None;

        loop {
            match poller.recv() {
                Err(PollerError::Shutdown) => break,
                Err(err) => Err(err)?,
                Ok(rsp) => match frontend.process(rsp) {
                    Err(FrontendError::Shutdown) => break,
                    Err(FrontendError::Nothing) => (),
                    Ok(monitor) => last_monitor = Some(monitor),
                },
            }
        }

        frontend.teardown();
        return last_monitor.ok_or(PollerError::Shutdown);
    }
}
