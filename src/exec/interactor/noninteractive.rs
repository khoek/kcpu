use super::super::interactive::Interactor as TraitInteractor;
use crate::exec::types::Snapshot;

pub struct Interactor;

impl TraitInteractor for Interactor {
    type State = Snapshot;
    type Action = ();

    fn startup(&mut self) {}

    fn handle(&mut self, _: &Snapshot) -> Option<()> {
        Some(())
    }
}
