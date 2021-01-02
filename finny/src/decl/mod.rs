//! The builder-style API structures for defining your Finny FSM. The procedural macro parses
//! these method calls and generated the optimized implementation.

mod fsm;
mod state;
mod event;

pub use self::fsm::*;
pub use self::state::*;
pub use self::event::*;