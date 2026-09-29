extern crate alloc;

use ::tracing::{Span, error, info, info_span};
use crate::{FsmBackend, FsmBackendImpl, FsmEvent, Inspect, InspectEvent, InspectFsmEvent};
use crate::lib::*;
use core::fmt::Debug;
use core::any::Any;
use alloc::string::ToString;
use alloc::format;

/// Reports the FSM's dispatching through [`tracing`](::tracing). Each dispatched event opens
/// a span, with nested spans for the matched transition, submachines and timers.
/// Install a `tracing` subscriber to collect the output.
pub struct InspectTracing {
    pub span: Span
}

impl InspectTracing {
    /// The dispatch spans will be children of the span that is current at the time of the dispatch.
    pub fn new() -> Self {
        Self::with_parent(Span::none())
    }

    /// The dispatch spans will be children of the provided span.
    pub fn with_parent(span: Span) -> Self {
        InspectTracing {
            span
        }
    }

    /// Runs `f` within our span. For the root inspector (`Span::none()`) this is a no-op, so the
    /// spans and events inherit whatever span is current at the call site.
    fn scope<T>(&self, f: impl FnOnce() -> T) -> T {
        self.span.in_scope(f)
    }
}

impl Default for InspectTracing {
    fn default() -> Self {
        Self::new()
    }
}

impl Inspect for InspectTracing
{
    fn new_event<F: FsmBackend>(&self, event: &FsmEvent<<F as FsmBackend>::Events, <F as FsmBackend>::Timers>, fsm: &FsmBackendImpl<F>) -> Self {
        let event_display = match event {
            FsmEvent::Timer(t) => format!("Fsm::Timer({:?})", t),
            _ => event.as_ref().to_string()
        };

        let span = self.scope(|| info_span!("dispatch", event = %event_display, start_state = ?fsm.get_current_states()));
        span.in_scope(|| info!("Dispatching"));
        Self::with_parent(span)
    }

    fn for_transition<T>(&self) -> Self {
        let span = self.scope(|| info_span!("transition", transition = %type_name::<T>()));
        span.in_scope(|| info!("Matched transition"));
        Self::with_parent(span)
    }

    fn for_sub_machine<FSub: FsmBackend>(&self) -> Self {
        let span = self.scope(|| info_span!("sub_machine", sub_fsm = %type_name::<FSub>()));
        span.in_scope(|| info!("Dispatching to a submachine"));
        Self::with_parent(span)
    }

    fn for_timer<F>(&self, timer_id: <F as FsmBackend>::Timers) -> Self where F: FsmBackend {
        Self::with_parent(self.scope(|| info_span!("timer", timer_id = ?timer_id)))
    }

    fn on_guard<T>(&self, guard_result: bool) {
        self.scope(|| info!(guard = %type_name::<T>(), guard_result, "Guard evaluated"));
    }

    fn on_state_enter<S>(&self) {
        self.scope(|| info!(state = %type_name::<S>(), "Entering state"));
    }

    fn on_state_exit<S>(&self) {
        self.scope(|| info!(state = %type_name::<S>(), "Exiting state"));
    }

    fn on_action<S>(&self) {
        self.scope(|| info!(action = %type_name::<S>(), "Executing action"));
    }

    fn event_done<F: FsmBackend>(self, fsm: &FsmBackendImpl<F>) {
        self.scope(|| info!(stop_state = ?fsm.get_current_states(), "Dispatch done"));
    }

    fn on_error<E>(&self, msg: &str, error: &E) where E: Debug {
        self.scope(|| error!(error = ?error, "{}", msg));
    }

    fn info(&self, msg: &str) {
        self.scope(|| info!("{}", msg));
    }
}

impl InspectEvent for InspectTracing
{
    fn on_event<S: Any + Debug + Clone>(&self, event: &InspectFsmEvent<S>) {
        self.scope(|| info!(event = ?event, "Inspection event"));
    }
}
