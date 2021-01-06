use crate::{FsmBackend, FsmBackendImpl, FsmEvent, FsmRegionId, lib::*};

pub trait Inspect<F> where F: FsmBackend {
    type ContextDispatchEvent;

    fn on_dispatch_event(&self, fsm: &FsmBackendImpl<F>, event: &FsmEvent<<F as FsmBackend>::Events>) -> Self::ContextDispatchEvent;
    fn on_state_enter<State>(&self, fsm: &FsmBackendImpl<F>, ctx: &mut Self::ContextDispatchEvent) where <F as FsmBackend>::States: AsRef<State>;
    fn on_state_exit<State>(&self, fsm: &FsmBackendImpl<F>, ctx: &mut Self::ContextDispatchEvent) where <F as FsmBackend>::States: AsRef<State>;
    fn on_matched_transition<T>(&self, fsm: &FsmBackendImpl<F>, region: FsmRegionId, ctx: &mut Self::ContextDispatchEvent);
    fn on_action(&self);
    fn on_guard<T, Guard>(&self);
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
    type ContextDispatchEvent = ();

    fn on_dispatch_event(&self, _fsm: &FsmBackendImpl<F>, _event: &FsmEvent<<F as FsmBackend>::Events>) -> Self::ContextDispatchEvent {
        ()
    }

    fn on_state_enter<State>(&self, _fsm: &FsmBackendImpl<F>, _ctx: &mut Self::ContextDispatchEvent) where <F as FsmBackend>::States: AsRef<State> {
        
    }

    fn on_state_exit<State>(&self, _fsm: &FsmBackendImpl<F>, _ctx: &mut Self::ContextDispatchEvent) where <F as FsmBackend>::States: AsRef<State> {
        
    }

    fn on_action(&self) {
        
    }

    fn on_guard<T, Guard>(&self) {
        
    }

    fn on_matched_transition<T>(&self, fsm: &FsmBackendImpl<F>, region: FsmRegionId, ctx: &mut Self::ContextDispatchEvent) {
        
    }
}