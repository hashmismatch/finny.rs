use crate::{FsmBackendImpl, lib::*};
use crate::{FsmBackend, FsmEvent};
pub trait Inspect {
    
    fn new_event<F: FsmBackend>(&self, event: &FsmEvent<<F as FsmBackend>::Events, <F as FsmBackend>::Timers>, fsm: &FsmBackendImpl<F>) -> Self;
    fn event_done<F: FsmBackend>(self, fsm: &FsmBackendImpl<F>);

    fn for_transition<T>(&self) -> Self;
    fn for_sub_machine<FSub: FsmBackend>(&self) -> Self;
    fn for_timer<F>(&self, timer_id: <F as FsmBackend>::Timers) -> Self where F: FsmBackend;

    fn on_guard<T>(&self, guard_result: bool);
    fn on_state_enter<S>(&self);
    fn on_state_exit<S>(&self);
    fn on_action<S>(&self);

    fn on_error<E>(&self, msg: &str, error: &E) where E: Debug;
    fn info(&self, msg: &str);
}

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

    fn on_error<E>(&self, msg: &str, error: &E) where E: Debug {
        
    }

    fn info(&self, msg: &str) {
        
    }
}

pub struct InspectChain<A, B>
    where A: Inspect, B: Inspect
{
    pub a: A,
    pub b: B
}

impl<A, B> InspectChain<A, B>
    where A: Inspect, B: Inspect
{
    pub fn new_pair(inspect_a: A, inspect_b: B) -> Self {
        InspectChain {
            a: inspect_a,
            b: inspect_b
        }
    }
}

impl<A, B> Inspect for InspectChain<A, B>
    where A: Inspect, B: Inspect
{
    fn new_event<F: FsmBackend>(&self, event: &FsmEvent<<F as FsmBackend>::Events, <F as FsmBackend>::Timers>, fsm: &FsmBackendImpl<F>) -> Self {
        Self {
            a: self.a.new_event(event, fsm),
            b: self.b.new_event(event, fsm)
        }
    }

    fn event_done<F: FsmBackend>(self, fsm: &FsmBackendImpl<F>) {
        self.a.event_done(fsm);
        self.b.event_done(fsm);
    }

    fn for_transition<T>(&self) -> Self {
        Self {
            a: self.a.for_transition::<T>(),
            b: self.b.for_transition::<T>()
        }
    }

    fn for_sub_machine<FSub: FsmBackend>(&self) -> Self {
        Self {
            a: self.a.for_sub_machine::<FSub>(),
            b: self.b.for_sub_machine::<FSub>()
        }
    }

    fn for_timer<F>(&self, timer_id: <F as FsmBackend>::Timers) -> Self where F: FsmBackend {
        Self {
            a: self.a.for_timer::<F>(timer_id.clone()),
            b: self.b.for_timer::<F>(timer_id)
        }
    }

    fn on_guard<T>(&self, guard_result: bool) {
        self.a.on_guard::<T>(guard_result);
        self.b.on_guard::<T>(guard_result);
    }

    fn on_state_enter<S>(&self) {
        self.a.on_state_enter::<S>();
        self.b.on_state_enter::<S>();
    }

    fn on_state_exit<S>(&self) {
        self.a.on_state_exit::<S>();
        self.b.on_state_exit::<S>();
    }

    fn on_action<S>(&self) {
        self.a.on_action::<S>();
        self.b.on_action::<S>();
    }

    fn on_error<E>(&self, msg: &str, error: &E) where E: Debug {
        self.a.on_error(msg, error);
        self.b.on_error(msg, error);
    }

    fn info(&self, msg: &str) {
        self.a.info(msg);
        self.b.info(msg);
    }
}