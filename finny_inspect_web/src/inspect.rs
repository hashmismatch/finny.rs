//! The inspector that records the dispatching of the events into snapshots.

use std::{any::{Any, type_name}, fmt::Debug, sync::{Arc, Mutex}, time::Instant};

use finny::{FsmBackend, FsmBackendImpl, FsmCurrentState, FsmDispatchResult, FsmError, FsmEvent, FsmStates, Inspect, InspectEvent, InspectFsmEvent};
use serde_json::Value;

use crate::{registry::{FsmInstance, lock, now_ms}, snapshot::{EventRecord, Snapshot, TraceEntry, TraceKind}};

/// The inspector of an FSM instance, created by [`Inspector::attach`](crate::Inspector::attach).
/// Records a snapshot after each event that the FSM handled.
pub struct InspectWeb {
    instance: Arc<FsmInstance>,
    /// `None` for the FSM's own inspector, set within the handling of an event.
    ctx: Option<EventCtx>
}

#[derive(Clone)]
struct EventCtx {
    trace: Arc<Mutex<Trace>>,
    depth: u32,
    /// The type names of the sub-machines, from the root machine.
    path: Arc<[String]>,
    /// Created for the root machine's event, finishing it records the snapshot.
    root_event: bool
}

struct Trace {
    started: Instant,
    timestamp_ms: u64,
    event: EventRecord,
    error: Option<String>,
    entries: Vec<TraceEntry>
}

impl InspectWeb {
    pub(crate) fn new_root(instance: Arc<FsmInstance>) -> Self {
        InspectWeb { instance, ctx: None }
    }

    /// The id of the FSM instance in the frontend.
    pub fn instance_id(&self) -> &str {
        &self.instance.id
    }

    fn push(&self, kind: TraceKind) {
        if let Some(ref ctx) = self.ctx {
            lock(&ctx.trace).entries.push(TraceEntry { depth: ctx.depth, kind });
        }
    }

    /// Records the entry and returns the context of the nested entries.
    fn nested(&self, kind: TraceKind, sub_machine: Option<&str>) -> Self {
        let ctx = self.ctx.as_ref().map(|ctx| {
            lock(&ctx.trace).entries.push(TraceEntry { depth: ctx.depth, kind });
            let path = match sub_machine {
                Some(sub) => ctx.path.iter().cloned().chain(std::iter::once(sub.to_string())).collect(),
                None => ctx.path.clone()
            };
            EventCtx { trace: ctx.trace.clone(), depth: ctx.depth + 1, path, root_event: false }
        });

        InspectWeb { instance: self.instance.clone(), ctx }
    }
}

impl Drop for InspectWeb {
    fn drop(&mut self) {
        if self.ctx.is_none() {
            self.instance.detach();
        }
    }
}

fn to_json(value: Option<&dyn finny::bundled::erased_serde::Serialize>) -> Result<Option<Value>, String> {
    match value {
        Some(v) => serde_json::to_value(v).map(Some).map_err(|e| e.to_string()),
        None => Ok(None)
    }
}

fn event_record<F: FsmBackend>(event: &FsmEvent<<F as FsmBackend>::Events, <F as FsmBackend>::Timers>) -> EventRecord {
    match event {
        FsmEvent::Start => EventRecord::Start,
        FsmEvent::Stop => EventRecord::Stop,
        FsmEvent::Timer(t) => EventRecord::Timer { timer: format!("{:?}", t) },
        FsmEvent::Event(ev) => EventRecord::Event {
            name: ev.as_ref().to_string(),
            value: to_json(F::serialize_event(ev)).unwrap_or_else(|e| Some(Value::String(format!("Failed to serialize: {}", e))))
        }
    }
}

/// The ids of the current states, one for each region.
fn current_states<F: FsmBackend>(fsm: &FsmBackendImpl<F>) -> Vec<Option<String>> {
    let current = fsm.get_current_states();
    let states: &[FsmCurrentState<<<F as FsmBackend>::States as FsmStates<F>>::StateKind>] = current.as_ref();
    states.iter().map(|s| match s {
        FsmCurrentState::Stopped => None,
        FsmCurrentState::State(kind) => Some(format!("{:?}", kind))
    }).collect()
}

fn error_string(e: &FsmError) -> String {
    format!("{:?}", e)
}

impl Inspect for InspectWeb {
    fn new_event<F: FsmBackend>(&self, event: &FsmEvent<<F as FsmBackend>::Events, <F as FsmBackend>::Timers>, _fsm: &FsmBackendImpl<F>) -> Self {
        let record = event_record::<F>(event);
        match self.ctx {
            // an event of the root machine
            None => {
                let trace = Trace {
                    started: Instant::now(),
                    timestamp_ms: now_ms(),
                    event: record,
                    error: None,
                    entries: Vec::new()
                };
                InspectWeb {
                    instance: self.instance.clone(),
                    ctx: Some(EventCtx { trace: Arc::new(Mutex::new(trace)), depth: 0, path: Arc::from([]), root_event: true })
                }
            },
            // dispatched into a sub-machine, or enqueued by a timer
            Some(_) => self.nested(TraceKind::Event { event: record }, None)
        }
    }

    fn event_done<F: FsmBackend>(self, fsm: &FsmBackendImpl<F>) {
        let Some(ref ctx) = self.ctx else { return };

        self.instance.set_active(&ctx.path, current_states(fsm));
        if !ctx.root_event {
            return;
        }

        let (values, values_error) = match to_json(F::serialize_backend(fsm)) {
            Ok(values) => (values, None),
            Err(e) => (None, Some(e))
        };

        let trace = {
            let mut trace = lock(&ctx.trace);
            Trace {
                started: trace.started,
                timestamp_ms: trace.timestamp_ms,
                event: trace.event.clone(),
                error: trace.error.take(),
                entries: std::mem::take(&mut trace.entries)
            }
        };

        self.instance.publish(Snapshot {
            seq: 0,
            timestamp_ms: trace.timestamp_ms,
            duration_us: trace.started.elapsed().as_micros() as u64,
            event: trace.event,
            error: trace.error,
            trace: trace.entries,
            active: Vec::new(),
            values,
            values_error
        });
    }

    fn for_transition<T>(&self) -> Self {
        self.nested(TraceKind::Transition { type_name: type_name::<T>().into() }, None)
    }

    fn for_sub_machine<FSub: FsmBackend>(&self) -> Self {
        let sub = type_name::<FSub>();
        self.nested(TraceKind::SubMachine { type_name: sub.into() }, Some(sub))
    }

    fn for_timer<F>(&self, timer_id: <F as FsmBackend>::Timers) -> Self where F: FsmBackend {
        self.nested(TraceKind::Timer { timer: format!("{:?}", timer_id) }, None)
    }

    fn on_guard<T>(&self, guard_result: bool) {
        self.push(TraceKind::Guard { type_name: type_name::<T>().into(), result: guard_result });
    }

    // The states are recorded by `on_event`, with the ids of the states.
    fn on_state_enter<S>(&self) { }

    fn on_state_exit<S>(&self) { }

    fn on_action<S>(&self) {
        self.push(TraceKind::Action { type_name: type_name::<S>().into() });
    }

    fn on_dispatch_result(&self, result: &FsmDispatchResult) {
        let (Some(ctx), Err(e)) = (&self.ctx, result) else { return };
        if ctx.root_event {
            lock(&ctx.trace).error = Some(error_string(e));
        } else {
            self.push(TraceKind::DispatchError { error: error_string(e) });
        }
    }

    fn on_error<E>(&self, msg: &str, error: &E) where E: Debug {
        self.push(TraceKind::Error { message: msg.into(), error: format!("{:?}", error) });
    }

    fn info(&self, msg: &str) {
        self.push(TraceKind::Info { message: msg.into() });
    }

    // Already recorded, the snapshot of the failed event has the error.
    fn on_queued_event_error<F: FsmBackend>(&self, _event: &<F as FsmBackend>::Events, _error: &FsmError) { }
}

impl InspectEvent for InspectWeb {
    fn on_event<S: Any + Debug + Clone>(&self, event: &InspectFsmEvent<S>) {
        match event {
            InspectFsmEvent::StateEnter(s) => self.push(TraceKind::StateEnter { state: format!("{:?}", s) }),
            InspectFsmEvent::StateExit(s) => self.push(TraceKind::StateExit { state: format!("{:?}", s) })
        }
    }
}
