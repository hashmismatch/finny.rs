use crate::{FsmBackend, FsmBackendImpl, FsmDispatchResult, FsmEvent, FsmRegionId, lib::*};

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

#[derive(Default)]
pub struct InspectNull<F> {
    _fsm: PhantomData<F>
}

impl<F> InspectNull<F> {
    pub fn new() -> Self {
        Self {
            _fsm: PhantomData::default()
        }
    }
}

impl<F> Inspect<F> for InspectNull<F> where F: FsmBackend {
    type CtxEvent = ();
    type CtxTransition = ();

    fn on_dispatch_event(&self, _fsm: &FsmBackendImpl<F>, _event: &FsmEvent<<F as FsmBackend>::Events>) -> Self::CtxEvent {
        ()
    }

    fn on_dispatched_event(&self, fsm: &FsmBackendImpl<F>, ctx: Self::CtxEvent, result: &FsmDispatchResult) {

    }

    fn on_state_enter<State>(&self, _fsm: &FsmBackendImpl<F>, _ctx: &mut Self::CtxEvent) where <F as FsmBackend>::States: AsRef<State> {
        
    }

    fn on_state_exit<State>(&self, _fsm: &FsmBackendImpl<F>, _ctx: &mut Self::CtxEvent) where <F as FsmBackend>::States: AsRef<State> {
        
    }

    fn on_action<T>(&self, _fsm: &FsmBackendImpl<F>, _ctx: &mut Self::CtxEvent) {
        
    }

    fn on_guard<T>(&self, fsm: &FsmBackendImpl<F>, ctx: &mut Self::CtxEvent, guard_result: bool) {
        
    }

    fn on_matched_transition<T>(&self, fsm: &FsmBackendImpl<F>, region: FsmRegionId, ctx: &mut Self::CtxEvent) -> Self::CtxTransition {
        ()
    }
}