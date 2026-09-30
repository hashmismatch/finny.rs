use core::fmt::Debug;
use core::any::Any;

use crate::{FsmBackend, FsmBackendImpl, FsmEvent, FsmStates};

#[derive(Debug, Clone)]
pub enum InspectFsmEvent<S> where S: Debug + Clone {
    StateEnter(S),
    StateExit(S)
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

    /// The result of dispatching the event, reported just before `event_done`.
    fn on_dispatch_result(&self, _result: &crate::FsmDispatchResult) { }

    fn on_error<E>(&self, msg: &str, error: &E) where E: core::fmt::Debug;
    fn info(&self, msg: &str);

    /// An event from the queue, dispatched as part of running the machine to completion, failed.
    /// Most commonly with `FsmError::NoTransition`, when the current state doesn't handle it.
    fn on_queued_event_error<F: FsmBackend>(&self, event: &<F as FsmBackend>::Events, error: &crate::FsmError) {
        self.on_error(event.as_ref(), error);
    }
}

pub trait InspectEvent {
    fn on_event<S: Any + Debug + Clone>(&self, event: &InspectFsmEvent<S>);
}