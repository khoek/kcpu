use crate::exec::types::{
    Backend, EventLoop as TraitEventLoop, Frontend, FrontendError, Poller, PollerError,
};

pub struct EventLoop;

impl<T> TraitEventLoop<T> for EventLoop {
    type Monitor = T;

    fn run<B: Backend, F: Frontend<T, Response = B::Response>, P: Poller<B>>(
        &mut self,
        mut poller: P,
        mut frontend: F,
    ) -> Result<T, B::Error> {
        let mut last_snap: Option<T> = None;
        loop {
            match poller.recv() {
                Err(PollerError::Shutdown) => return Ok(last_snap.unwrap()),
                Err(PollerError::Backend(b)) => return Err(b),
                Ok(rsp) => match frontend.process(rsp) {
                    Err(FrontendError::Shutdown) => return Ok(last_snap.unwrap()),
                    Err(FrontendError::Nothing) => (),
                    Ok(snap) => last_snap = Some(snap),
                },
            }
        }
    }
}
