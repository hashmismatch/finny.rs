use crate::lib::*;
use crate::{FsmBackend, FsmError, FsmEvent};
pub trait Inspect {
    
    fn new_event<F: FsmBackend>(&self, event: &FsmEvent<<F as FsmBackend>::Events>) -> Self;
    fn event_done(self);

    fn for_transition<T>(&self) -> Self;
    fn for_sub_machine<FSub: FsmBackend>(&self) -> Self;

    fn on_guard<T>(&self, guard_result: bool);
    fn on_state_enter<S>(&self);
    fn on_state_exit<S>(&self);
    fn on_action<S>(&self);

    fn on_error<E>(&self, msg: &str, error: &E) where E: Debug;
}

#[derive(Default)]
pub struct InspectNull;

impl InspectNull {
    pub fn new() -> Self {
        InspectNull { }
    }
}

impl Inspect for InspectNull {
    fn new_event<F: FsmBackend>(&self, _event: &FsmEvent<<F as FsmBackend>::Events>) -> Self {
        Self::default()
    }

    fn for_transition<T>(&self) -> Self {
        Self::default()
    }

    fn for_sub_machine<FSub: FsmBackend>(&self) -> Self {
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

    fn event_done(self) {
        
    }

    fn on_error<E>(&self, msg: &str, error: &E) where E: Debug {
        
    }
}