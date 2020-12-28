//! The public Finite State Machine traits. The derive macros will implement these for your particular
//! state machines.

pub trait Fsm where Self: Sized {
    type Context;
    type States;

    fn get_states(&self) -> &Self::States;
    fn get_states_mut(&mut self) -> &mut Self::States;
}

pub trait FsmStateFactory {
    fn new_state<C>(context: &C) -> Self;
}

impl<TState> FsmStateFactory for TState where TState: Default {
    fn new_state<C>(_context: &C) -> Self {
        Default::default()
    }
}

pub struct StateContext<'a, TFsm: Fsm> {
    pub context: &'a mut TFsm::Context
}

pub struct EventContext<'a, TFsm: Fsm> {
    pub context: &'a mut TFsm::Context
}