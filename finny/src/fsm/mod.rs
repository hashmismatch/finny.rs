//! The public Finite State Machine traits. The derive macros will implement these for your particular
//! state machines.

mod events;
mod fsm_impl;
mod fsm_factory;
mod queue;
mod states;
mod transitions;
mod tests_fsm;
mod dispatch;
mod timers;
mod inspect;
#[cfg(feature = "async")]
mod asynch;

pub use self::events::*;
pub use self::fsm_factory::*;
pub use self::fsm_impl::*;
pub use self::queue::*;
pub use self::states::*;
pub use self::transitions::*;
pub use self::inspect::*;
pub use self::dispatch::*;
pub use self::timers::*;
#[cfg(feature = "async")]
pub use self::asynch::*;

use crate::lib::*;

pub type FsmResult<T = ()> = Result<T, FsmError>;

/// The lib-level error type.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum FsmError {
    NoTransition,
    QueueOverCapacity,
    NotSupported,
    TimerNotStarted,
    /// A renewing timer needs a non-zero timeout.
    InvalidTimerSettings
}

pub type FsmDispatchResult = FsmResult<()>;

/// Finite State Machine backend. The types are defined by the code generator. The
/// dispatching is implemented by either [`FsmDispatch`] or `FsmAsyncDispatch`.
pub trait FsmBackend where Self: Sized + Debug {
    /// The machine's context that is shared between its constructors and actions.
    type Context;
    /// The type that holds the states of the machine.
    type States: FsmStates<Self>;
    /// A tagged union type with all the supported events. This type has to support cloning to facilitate
    /// the dispatch into sub-machines and into multiple regions.
    type Events: AsRef<str> + Clone;
    /// An enum with variants for all the possible timer instances, with support for submachines.
    type Timers: Debug + Clone + PartialEq + AllVariants;

    /// The machine's context, states and current states, for the FSMs that opted into the
    /// serialization with `fsm.serde()`.
    #[cfg(feature = "serde")]
    fn serialize_backend(_backend: &FsmBackendImpl<Self>) -> Option<&dyn erased_serde::Serialize> {
        None
    }

    /// The event, for the FSMs that opted into the serialization with `fsm.serde()`.
    #[cfg(feature = "serde")]
    fn serialize_event(_event: &Self::Events) -> Option<&dyn erased_serde::Serialize> {
        None
    }

    /// Can this state be the current state of the region? Checked while deserializing the
    /// machine's current states.
    #[cfg(feature = "serde")]
    fn is_valid_current_state(_region: FsmRegionId, _state: &<Self::States as FsmStates<Self>>::StateKind) -> bool {
        true
    }

    /// Re-creates the timers of a deserialized machine, including the ones of its sub-machines,
    /// with their original settings. The timers that fail to be created are reported to the
    /// inspector and dropped.
    #[cfg(feature = "serde")]
    fn restore_timers<I: Inspect, T: FsmTimers<Self>>(_backend: &mut FsmBackendImpl<Self>, _inspect: &I, _timers: &mut T) {

    }
}

/// Synchronous event dispatching, implemented by the code generator.
pub trait FsmDispatch: FsmBackend {
    fn dispatch_event<Q, I, T>(ctx: DispatchContext<Self, Q, I, T>, event: FsmEvent<Self::Events, Self::Timers>) -> FsmDispatchResult
        where Q: FsmEventQueue<Self>, I: Inspect, T: FsmTimers<Self>;
}

/// Enumerates all the possible variants of a simple enum.
pub trait AllVariants where Self: Sized
{
    type Iter: Iterator<Item=Self>;

    fn iter() -> Self::Iter;
}