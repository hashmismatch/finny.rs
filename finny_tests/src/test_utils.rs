//! Helpers shared by the tests.

use std::{any::Any, cell::RefCell, fmt::Debug, rc::Rc};

use finny::{FsmBackend, FsmBackendImpl, FsmError, FsmEvent, Inspect, InspectEvent, InspectFsmEvent};

/// An inspector that records the errors of the queued events.
#[derive(Clone, Default)]
pub struct QueuedErrors {
    pub errors: Rc<RefCell<Vec<(String, FsmError)>>>
}

impl Inspect for QueuedErrors {
    fn new_event<F: FsmBackend>(&self, _event: &FsmEvent<<F as FsmBackend>::Events, <F as FsmBackend>::Timers>, _fsm: &FsmBackendImpl<F>) -> Self { self.clone() }
    fn event_done<F: FsmBackend>(self, _fsm: &FsmBackendImpl<F>) { }
    fn for_transition<T>(&self) -> Self { self.clone() }
    fn for_sub_machine<FSub: FsmBackend>(&self) -> Self { self.clone() }
    fn for_timer<F>(&self, _timer_id: <F as FsmBackend>::Timers) -> Self where F: FsmBackend { self.clone() }
    fn on_guard<T>(&self, _guard_result: bool) { }
    fn on_state_enter<S>(&self) { }
    fn on_state_exit<S>(&self) { }
    fn on_action<S>(&self) { }
    fn on_error<E>(&self, _msg: &str, _error: &E) where E: Debug { }
    fn info(&self, _msg: &str) { }

    fn on_queued_event_error<F: FsmBackend>(&self, event: &<F as FsmBackend>::Events, error: &FsmError) {
        self.errors.borrow_mut().push((event.as_ref().to_string(), *error));
    }
}

impl InspectEvent for QueuedErrors {
    fn on_event<S: Any + Debug + Clone>(&self, _event: &InspectFsmEvent<S>) { }
}

/// An inspector that records the names of the executed transitions.
#[derive(Clone, Default)]
pub struct TransitionNames {
    pub names: Rc<RefCell<Vec<String>>>
}

impl Inspect for TransitionNames {
    fn new_event<F: FsmBackend>(&self, _event: &FsmEvent<<F as FsmBackend>::Events, <F as FsmBackend>::Timers>, _fsm: &FsmBackendImpl<F>) -> Self { self.clone() }
    fn event_done<F: FsmBackend>(self, _fsm: &FsmBackendImpl<F>) { }
    fn for_transition<T>(&self) -> Self {
        let name = std::any::type_name::<T>();
        self.names.borrow_mut().push(name.rsplit("::").next().unwrap_or(name).to_string());
        self.clone()
    }
    fn for_sub_machine<FSub: FsmBackend>(&self) -> Self { self.clone() }
    fn for_timer<F>(&self, _timer_id: <F as FsmBackend>::Timers) -> Self where F: FsmBackend { self.clone() }
    fn on_guard<T>(&self, _guard_result: bool) { }
    fn on_state_enter<S>(&self) { }
    fn on_state_exit<S>(&self) { }
    fn on_action<S>(&self) { }
    fn on_error<E>(&self, _msg: &str, _error: &E) where E: Debug { }
    fn info(&self, _msg: &str) { }
}

impl InspectEvent for TransitionNames {
    fn on_event<S: Any + Debug + Clone>(&self, _event: &InspectFsmEvent<S>) { }
}

/// An inspector that counts the dispatched events and the entered states.
#[derive(Clone, Default)]
pub struct EventCounts {
    pub new_events: Rc<RefCell<usize>>,
    pub events_done: Rc<RefCell<usize>>,
    pub state_enters: Rc<RefCell<usize>>
}

impl Inspect for EventCounts {
    fn new_event<F: FsmBackend>(&self, _event: &FsmEvent<<F as FsmBackend>::Events, <F as FsmBackend>::Timers>, _fsm: &FsmBackendImpl<F>) -> Self {
        *self.new_events.borrow_mut() += 1;
        self.clone()
    }
    fn event_done<F: FsmBackend>(self, _fsm: &FsmBackendImpl<F>) {
        *self.events_done.borrow_mut() += 1;
    }
    fn for_transition<T>(&self) -> Self { self.clone() }
    fn for_sub_machine<FSub: FsmBackend>(&self) -> Self { self.clone() }
    fn for_timer<F>(&self, _timer_id: <F as FsmBackend>::Timers) -> Self where F: FsmBackend { self.clone() }
    fn on_guard<T>(&self, _guard_result: bool) { }
    fn on_state_enter<S>(&self) {
        *self.state_enters.borrow_mut() += 1;
    }
    fn on_state_exit<S>(&self) { }
    fn on_action<S>(&self) { }
    fn on_error<E>(&self, _msg: &str, _error: &E) where E: Debug { }
    fn info(&self, _msg: &str) { }
}

impl InspectEvent for EventCounts {
    fn on_event<S: Any + Debug + Clone>(&self, _event: &InspectFsmEvent<S>) { }
}
