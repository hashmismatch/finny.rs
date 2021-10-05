pub mod null;
pub mod chain;
pub mod events;

use crate::{FsmBackend, FsmBackendImpl, FsmEvent, FsmStates};

#[derive(Debug)]
pub enum InspectFsmEvent<F> where F: FsmBackend {
    StateEnter(<<F as FsmBackend>::States as FsmStates<F>>::StateKind),
    StateExit(<<F as FsmBackend>::States as FsmStates<F>>::StateKind)
}

impl<F> Clone for InspectFsmEvent<F> where F: FsmBackend {
    fn clone(&self) -> Self {
        match self {
            Self::StateEnter(arg0) => Self::StateEnter(arg0.clone()),
            Self::StateExit(arg0) => Self::StateExit(arg0.clone()),
        }
    }
}

pub trait Inspect: InspectEvent {
    
    fn new_event<F: FsmBackend>(&self, event: &FsmEvent<<F as FsmBackend>::Events, <F as FsmBackend>::Timers>, fsm: &FsmBackendImpl<F>) -> Self;
    fn event_done<F: FsmBackend>(self, fsm: &FsmBackendImpl<F>);

    fn for_transition<T>(&self) -> Self;
    fn for_sub_machine<FSub: FsmBackend>(&self) -> Self;
    fn for_timer<F>(&self, timer_id: <F as FsmBackend>::Timers) -> Self where F: FsmBackend;

    fn on_guard<T>(&self, guard_result: bool);
    fn on_state_enter<S>(&self);
    fn on_state_exit<S>(&self);
    fn on_action<S>(&self);

    fn on_error<E>(&self, msg: &str, error: &E) where E: core::fmt::Debug;
    fn info(&self, msg: &str);    
}

pub trait InspectEvent {
    fn on_event<F: FsmBackend>(&self, event: InspectFsmEvent<F>);
}