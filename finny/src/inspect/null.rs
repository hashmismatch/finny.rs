use crate::{FsmBackend, FsmBackendImpl, FsmEvent, Inspect, InspectEvent, InspectFsmEvent};
use core::fmt::Debug;
use core::any::Any;

#[derive(Default)]
pub struct InspectNull;

impl InspectNull {
    pub fn new() -> Self {
        InspectNull { }
    }
}

impl Inspect for InspectNull {
    fn new_event<F: FsmBackend>(&self, _event: &FsmEvent<<F as FsmBackend>::Events, <F as FsmBackend>::Timers>, _fsm: &FsmBackendImpl<F>) -> Self {
        Self::default()
    }

    fn for_transition<T>(&self) -> Self {
        Self::default()
    }

    fn for_sub_machine<FSub: FsmBackend>(&self) -> Self {
        Self::default()
    }

    fn for_timer<F>(&self, _timer_id: <F as FsmBackend>::Timers) -> Self where F: FsmBackend {
        Self::default()
    }    

    fn on_guard<T>(&self, _guard_result: bool) {
        
    }

    fn on_state_enter<S>(&self) {
        
    }

    fn on_state_exit<S>(&self) {
        
    }

    fn on_action<S>(&self) {
        
    }

    fn event_done<F: FsmBackend>(self, fsm: &FsmBackendImpl<F>) {
        
    }

    fn on_error<E>(&self, msg: &str, error: &E) where E: core::fmt::Debug {
        
    }

    fn info(&self, msg: &str) {
        
    }
}

impl InspectEvent for InspectNull {
    fn on_event<S: Any + Debug + Clone>(&self, event: &InspectFsmEvent<S>) {
        
    }
}