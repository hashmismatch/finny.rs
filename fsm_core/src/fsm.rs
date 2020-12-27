//! The public Finite State Machine traits. The derive macros will implement these for your particular
//! state machines.

pub trait Fsm where Self: Sized {
    type Context;
}

pub struct StateContext<'a, TFsm: Fsm> where TFsm::Context: 'a {
    pub context: &'a mut TFsm::Context
}

pub struct EventContext<'a, TFsm: Fsm> {
    pub context: &'a mut TFsm::Context
}