use crate::{FsmBackend, FsmBackendImpl, FsmDispatchResult, FsmEvent, FsmRegionId, lib::*};

/*
// todo: pull the F into the methods.
pub trait Inspect<F> where F: FsmBackend {
    type CtxEvent;
    type CtxTransition;
    
    fn on_dispatch_event(&self, fsm: &FsmBackendImpl<F>, event: &FsmEvent<<F as FsmBackend>::Events>) -> Self::CtxEvent;
    fn on_dispatched_event(&self, fsm: &FsmBackendImpl<F>, ctx: Self::CtxEvent, result: &FsmDispatchResult);
    
    fn on_guard<T>(&self, fsm: &FsmBackendImpl<F>, ctx: &mut Self::CtxEvent, guard_result: bool);

    fn on_matched_transition<T>(&self, fsm: &FsmBackendImpl<F>, region: FsmRegionId, ctx: &mut Self::CtxEvent) -> Self::CtxTransition;
    
    fn on_state_enter<State>(&self, fsm: &FsmBackendImpl<F>, ctx: &mut Self::CtxTransition) where <F as FsmBackend>::States: AsRef<State>;
    fn on_state_exit<State>(&self, fsm: &FsmBackendImpl<F>, ctx: &mut Self::CtxTransition) where <F as FsmBackend>::States: AsRef<State>;
    fn on_action<T>(&self, fsm: &FsmBackendImpl<F>, ctx: &mut Self::CtxTransition);
}
*/

pub trait Inspect {
    
    fn new_event<F: FsmBackend>(&self, event: &FsmEvent<<F as FsmBackend>::Events>) -> Self;
    fn event_done(self);

    fn for_transition<T>(&self) -> Self;
    fn for_sub_machine<FSub: FsmBackend>(&self) -> Self;

    fn on_guard<T>(&self, guard_result: bool);
    fn on_state_enter<S>(&self);
    fn on_state_exit<S>(&self);
    fn on_action<S>(&self);
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
}