use crate::exec::types::{Backend, EventLoop as TraitEventLoop, Frontend, Poller, Snapshot, Vram};

// RUSTFIX implement
pub struct EventLoop;

impl TraitEventLoop<Snapshot> for EventLoop {
    type Monitor = (Snapshot, Vram);

    fn run<B: Backend, F: Frontend<(Snapshot, Vram), Response = B::Response>, P: Poller<B>>(
        &mut self,
        _: P,
        _: F,
    ) -> Result<Snapshot, B::Error> {
        todo!();
    }
}
