use crate::lib::*;

use crate::FsmResult;

/// The implementation should hold all of the FSM's states as fields.
pub trait FsmStates: FsmStateFactory {
    /// The enum type for all states that's used as the "current state" field in the FSM's backend.
    type StateKind: Clone + Copy + Debug + PartialEq;
    /// An array of current states for the machine, one for each region.
    type CurrentState: Clone + Copy + Debug + Default + AsMut<[FsmCurrentState<Self::StateKind>]>;
}

/// The current state of the FSM.
#[derive(Copy, Clone, PartialEq)]
pub enum FsmCurrentState<S> where S: Clone + Copy {
    /// The FSM is halted and has to be started using the `start()` method.
    Stopped,
    /// The FSM is in this state.
    State(S)
}

impl<S> Debug for FsmCurrentState<S> where S: Debug + Copy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FsmCurrentState::Stopped => f.write_str("Fsm::Stopped"),
            FsmCurrentState::State(s) => s.fmt(f)
        }
    }
}

impl<S> Default for FsmCurrentState<S> where S: Clone + Copy {
    fn default() -> Self {
        Self::Stopped
    }
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

/// Retrieve a pair of states as immutable references. Used in state transitions.
pub trait FsmStateTransitionAsRef<T1, T2> {
    fn as_state_transition_ref(&self) -> (&T1, &T2);
}

/// Retrieve a pair of states as mutable references. Used in state transitions.
pub trait FsmStateTransitionAsMut<T1, T2> {
    fn as_state_transition_mut(&mut self) -> (&mut T1, &mut T2);
}