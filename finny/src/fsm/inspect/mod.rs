use core::fmt::Debug;
use core::any::Any;

use crate::{FsmBackend, FsmBackendImpl, FsmEvent, FsmStates, TimerFsmSettings};

#[derive(Debug, Clone)]
pub enum InspectFsmEvent<S> where S: Debug + Clone {
    StateEnter(S),
    StateExit(S)
}

/// A change in the lifecycle of a state's timer, see [`Inspect::on_timer`].
#[derive(Debug, Clone, Copy)]
pub enum InspectTimerEvent {
    /// The timer was created in the timers service: its state was entered, or the machine was
    /// `restored` with `FsmFactory::restore`.
    Started { settings: TimerFsmSettings, restored: bool },
    /// The timer wasn't started, its setup disabled it.
    Disabled,
    /// The timers service failed to create the timer.
    Failed,
    /// The timer was cancelled, as its state was exited.
    Cancelled,
    /// The timer triggered, its event, if any, was enqueued.
    Triggered
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

    /// The lifecycle of a state's timer of the machine `F`. Called on the inspector returned by
    /// `for_timer`, or on the machine's own inspector when the timers of a restored machine are
    /// re-created.
    fn on_timer<F: FsmBackend>(&self, _timer: &<F as FsmBackend>::Timers, _event: &InspectTimerEvent) { }
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