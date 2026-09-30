//! The recorded history: a snapshot of the machine after each dispatched event.

use serde::Serialize;
use serde_json::Value;

/// The state of the machine after an event was fully handled, including the event's trace.
#[derive(Debug, Clone, Serialize)]
pub struct Snapshot {
    /// Increases with each snapshot of the FSM instance, starting at 1.
    pub seq: u64,
    /// Milliseconds since the UNIX epoch, when the event was dispatched.
    pub timestamp_ms: u64,
    /// How long it took to handle the event.
    pub duration_us: u64,
    pub event: EventRecord,
    /// The dispatch error, `None` when the event was handled.
    pub error: Option<String>,
    /// Everything that happened while handling the event, in order.
    pub trace: Vec<TraceEntry>,
    /// The current states of the machine and of all its sub-machines.
    pub active: Vec<ActiveStates>,
    /// The timers of the machine and of its sub-machines that were started at least once.
    pub timers: Vec<TimerStatus>,
    /// The serialized `{ context, states, current_states }`, for the FSMs that opted in with
    /// `fsm.serde()`.
    pub values: Option<Value>,
    /// Set when the FSM's values failed to serialize.
    pub values_error: Option<String>
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind")]
pub enum EventRecord {
    Start,
    Stop,
    Timer {
        timer: String
    },
    Event {
        name: String,
        /// The serialized event, for the FSMs that opted in with `fsm.serde()`.
        value: Option<Value>
    }
}

/// The current states of a machine.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ActiveStates {
    /// The type names of the sub-machines leading to this machine, empty for the root machine.
    pub path: Vec<String>,
    /// The id of the current state of each region, `None` when the region is stopped.
    pub states: Vec<Option<String>>
}

/// The last known state of a timer.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TimerStatus {
    /// The type names of the sub-machines leading to the timer's machine, empty for the root machine.
    pub path: Vec<String>,
    /// The id of the timer, as in the FSM's description.
    pub timer: String,
    pub status: TimerState,
    pub timeout_ms: f64,
    pub renew: bool,
    pub cancel_on_state_exit: bool,
    /// Milliseconds since the UNIX epoch, when the timer was last started.
    pub started_ms: u64,
    /// Milliseconds since the UNIX epoch, when the timer last triggered since it was started.
    pub last_triggered_ms: Option<u64>,
    /// How many times the timer triggered since it was started.
    pub triggers: u64,
    /// Started by restoring a saved machine.
    pub restored: bool
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TimerState {
    Running,
    /// A timer that doesn't renew and already triggered.
    Expired,
    Cancelled,
    /// The timers service failed to create the timer.
    Failed,
    /// The timer's setup disabled it.
    Disabled
}

#[derive(Debug, Clone, Serialize)]
pub struct TraceEntry {
    /// The nesting within the transitions, sub-machines and timers.
    pub depth: u32,
    #[serde(flatten)]
    pub kind: TraceKind
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TraceKind {
    /// An event dispatched within the handling of the root event: into a sub-machine, or the
    /// event that a timer enqueued.
    Event { event: EventRecord },
    /// A transition was selected. The nested entries belong to it.
    Transition { type_name: String },
    /// The event is dispatched into a sub-machine.
    SubMachine { type_name: String },
    Timer { timer: String },
    Guard { type_name: String, result: bool },
    StateExit { state: String },
    StateEnter { state: String },
    Action { type_name: String },
    /// A nested dispatch, into a sub-machine, failed.
    DispatchError { error: String },
    Error { message: String, error: String },
    Info { message: String }
}
