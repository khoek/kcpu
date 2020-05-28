mod instance;
mod interface;
mod types;

mod alu;
mod ctl;
mod io;
mod mem;
mod reg;

pub use instance::{Instance, State};
pub use interface::{VIDEO_HEIGHT, VIDEO_WIDTH};
pub use types::LogLevel;

pub mod debug {
    pub use super::instance::debug::ExecPhase;
}
