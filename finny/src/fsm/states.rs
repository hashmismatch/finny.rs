use lib::*;

use crate::FsmResult;

/// The implementation should hold all of the FSM's states as fields.
pub trait FsmStates: FsmStateFactory {
    /// The enum type for all states that's used as the "current state" field in the FSM's backend.
    /// In case the FSM has multiple regions, then this type is a tuple!
    type StateKind: Clone + Copy + Debug + PartialEq;
}

/// The current state of the FSM.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum FsmCurrentState<S: Clone + Copy> {
    /// The FSM is halted and has to be started using the `start()` method.
    Stopped,
    /// The FSM is in this state.
    State(S)
}

/// Create a new state from the shared global context.
pub trait FsmStateFactory where Self: Sized {
    /// Constructor for building this state from the shared global context.
    fn new_state<C>(context: &C) -> FsmResult<Self>;
}

impl<TState> FsmStateFactory for TState where TState: Default {
    fn new_state<C>(_context: &C) -> FsmResult<Self> {
        Ok(Default::default())
    }
}
