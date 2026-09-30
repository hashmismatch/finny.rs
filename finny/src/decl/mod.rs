//! The builder-style API structures for defining your Finny FSM. The procedural macro parses
//! these method calls and generated the optimized implementation.

mod fsm;
#[cfg(feature = "async")]
mod fsm_async;
mod state;
mod event;
mod sub;

pub use self::fsm::*;
#[cfg(feature = "async")]
pub use self::fsm_async::*;
pub use self::state::*;
pub use self::event::*;
pub use self::sub::*;

/// Marks the builders of a synchronous FSM, [`FsmBuilder`].
pub struct SyncMode;
/// Marks the builders of an async FSM, `FsmAsyncBuilder`.
pub struct AsyncMode;

#[cfg(feature = "std")]
pub type FsmQueueMock<F> = crate::FsmEventQueueVec<F>;

#[cfg(not(feature = "std"))]
pub type FsmQueueMock<F> = crate::FsmEventQueueNull<F>;