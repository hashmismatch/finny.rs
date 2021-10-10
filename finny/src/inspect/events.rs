use crate::{FsmBackend, FsmBackendImpl, FsmEvent, Inspect, InspectEvent, InspectFsmEvent};
use core::any::Any;
use core::fmt::Debug;

#[derive(Clone)]
pub struct EventInspector<T>
    where T: InspectEvent + Clone
{
    event_handler: T
}

impl<T> EventInspector<T>
    where T: InspectEvent + Clone
{
    pub fn new(inspect: T) -> Self {
        Self {
            event_handler: inspect
        }
    }
}

impl<TI> Inspect for EventInspector<TI>
    where TI: InspectEvent + Clone
{
    fn new_event<F: FsmBackend>(&self, _event: &FsmEvent<<F as FsmBackend>::Events, <F as FsmBackend>::Timers>, _fsm: &FsmBackendImpl<F>) -> Self {
        self.clone()
    }

    fn for_transition<T>(&self) -> Self {
        self.clone()
    }

    fn for_sub_machine<FSub: FsmBackend>(&self) -> Self {
        self.clone()
    }

    fn for_timer<F>(&self, _timer_id: <F as FsmBackend>::Timers) -> Self where F: FsmBackend {
        self.clone()
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

impl<TI> InspectEvent for EventInspector<TI>
    where TI: InspectEvent + Clone
{
    fn on_event<S: Any + Debug + Clone>(&self, event: &InspectFsmEvent<S>) {
        self.event_handler.on_event(event)
    }
}