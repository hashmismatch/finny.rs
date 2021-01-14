//! The public Finite State Machine traits. The derive macros will implement these for your particular
//! state machines.

mod events;
mod fsm_impl;
mod fsm_factory;
mod queue;
mod states;
mod transitions;
mod tests_fsm;
mod inspect;
mod dispatch;

pub use self::events::*;
pub use self::fsm_factory::*;
pub use self::fsm_impl::*;
pub use self::queue::*;
pub use self::states::*;
pub use self::transitions::*;
pub use self::inspect::*;
pub use self::dispatch::*;

use crate::{bundled, lib::*};

pub type FsmResult<T> = Result<T, FsmError>;

/// The lib-level error type.
#[derive(Debug, PartialEq)]
pub enum FsmError {
    NoTransition,
    QueueOverCapacity
}

pub type FsmDispatchResult = FsmResult<()>;

/// Finite State Machine backend. Handles the dispatching, the types are
/// defined by the code generator.
pub trait FsmBackend where Self: Sized {
    /// The machine's context that is shared between its constructors and actions.
    type Context;
    /// The type that holds the states of the machine.
    type States: FsmStates;
    /// A tagged union type with all the supported events. This type has to support cloning to facilitate
    /// the dispatch into sub-machines and into multiple regions.
    type Events: AsRef<str> + Clone;

    fn dispatch_event<Q, I>(ctx: DispatchContext<Self, Q, I>, event: FsmEvent<Self::Events>) -> FsmDispatchResult
        where Q: FsmEventQueue<Self>, I: Inspect;
}