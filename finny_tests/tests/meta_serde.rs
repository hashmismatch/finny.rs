//! The runtime description of the FSMs and the serialization of the FSMs that opt in.

use std::{any::type_name, sync::{Mutex, atomic::{AtomicUsize, Ordering}}, time::Duration};

use finny::{FsmAsyncFactory, FsmBackend, FsmFactory, FsmResult, decl::{BuiltFsm, FsmAsyncBuilder, FsmBuilder}, finny_fsm,
    meta::{EventInfo, FsmMeta, TransitionKindInfo, plantuml::to_plantuml}};
use serde::Serialize;
use serde_json::json;

#[derive(Default, Serialize)]
pub struct Idle { visits: usize }
#[derive(Default, Serialize)]
pub struct Running;
#[derive(Default, Serialize)]
pub struct LedOff;
#[derive(Default, Serialize)]
pub struct LedOn;

#[derive(Clone, Debug, Serialize)]
pub struct Go { speed: u32 }
#[derive(Clone, Debug, Serialize)]
pub struct Tick;
#[derive(Clone, Debug, Serialize)]
pub struct Toggle;

#[derive(Default, Serialize)]
pub struct MachineContext { gos: usize }

#[finny_fsm]
fn build_machine(mut fsm: FsmBuilder<Machine, MachineContext>) -> BuiltFsm {
    fsm.serde();
    fsm.initial_states::<(Idle, LedOff)>();

    fsm.state::<Idle>()
        .on_entry(|state, _| { state.visits += 1; })
        .on_event::<Go>()
        .transition_to::<Worker>()
        .guard(|ev, _, _| ev.speed > 0)
        .action(|_, ctx, _, _| { ctx.gos += 1; });

    fsm.sub_machine::<Worker>()
        .on_event::<Tick>()
        .self_transition();

    fsm.state::<LedOff>()
        .on_event::<Toggle>()
        .transition_to::<LedOn>();

    fsm.state::<LedOn>()
        .on_event::<Toggle>()
        .internal_transition();

    fsm.state::<LedOn>()
        .on_entry_start_timer(|_ctx, timer| {
            timer.timeout = Duration::from_millis(100);
        }, |_ctx, _state| {
            Some(Toggle.into())
        })
        .with_timer_ty::<Blink>();

    fsm.build()
}

#[derive(Default, Serialize)]
pub struct Warmup;
#[derive(Default, Serialize)]
pub struct Working { units: u32 }
#[derive(Clone, Debug, Serialize)]
pub struct Warm;

#[finny_fsm]
fn build_worker(mut fsm: FsmBuilder<Worker, ()>) -> BuiltFsm {
    fsm.serde();
    fsm.initial_state::<Warmup>();
    fsm.state::<Warmup>()
        .on_event::<Warm>()
        .transition_to::<Working>()
        .action(|_, _, _, working| { working.units += 1; });
    fsm.state::<Working>();
    fsm.build()
}

#[test]
fn test_meta_describes_the_fsm() {
    let info = Machine::fsm_info();

    assert_eq!("Machine", info.id);
    assert_eq!(type_name::<Machine>(), info.type_name);
    assert_eq!(type_name::<MachineContext>(), info.context_type_name);
    assert_eq!(2, info.regions.len());

    let r0 = &info.regions[0];
    assert_eq!("Idle", r0.initial_state);
    let idle = r0.states.iter().find(|s| s.id == "Idle").unwrap();
    assert_eq!(type_name::<Idle>(), idle.type_name);
    assert_eq!("idle", idle.storage_field);
    assert!(idle.sub_machine.is_none());

    let worker = r0.states.iter().find(|s| s.id == "Worker").unwrap();
    let sub = worker.sub_machine.as_ref().expect("a sub-machine");
    assert_eq!(type_name::<Worker>(), sub.type_name);
    assert_eq!(vec!["Warmup", "Working"], sub.regions[0].states.iter().map(|s| s.id.as_str()).collect::<Vec<_>>());

    let go = r0.transitions.iter().find(|t| matches!(&t.event, EventInfo::Event { id, .. } if id == "Go")).unwrap();
    assert!(go.has_guard);
    assert!(go.has_action);
    assert_eq!(TransitionKindInfo::Normal { from: Some("Idle".into()), to: Some("Worker".into()) }, go.kind);
    assert_eq!(EventInfo::Event { id: "Go".into(), type_name: type_name::<Go>().into() }, go.event);

    let start = r0.transitions.iter().find(|t| t.event == EventInfo::Start).unwrap();
    assert_eq!(TransitionKindInfo::Normal { from: None, to: Some("Idle".into()) }, start.kind);

    let r1 = &info.regions[1];
    let led_on = r1.states.iter().find(|s| s.id == "LedOn").unwrap();
    assert_eq!(1, led_on.timers.len());
    assert_eq!("Blink", led_on.timers[0].id);
    assert!(r1.transitions.iter().any(|t| t.kind == TransitionKindInfo::Internal { state: "LedOn".into() }));
}

#[test]
fn test_meta_plantuml() {
    let uml = to_plantuml(&Machine::fsm_info());
    assert!(uml.starts_with("@startuml Machine\n"));
    assert!(uml.trim_end().ends_with("@enduml"));
    assert!(uml.contains("[*] --> Idle : Start\n"));
    assert!(uml.contains("Idle --> Worker : Go [guard]\n"));
    assert!(uml.contains("Worker --> Worker : Tick (Self)\n"));
    assert!(uml.contains("state LedOn : Toggle (Internal)\n"));
    assert!(uml.contains("state LedOn : Timer Blink\n"));
    // the regions are separated
    assert!(uml.contains("\n--\n"));
    // the nested sub-machine
    assert!(uml.contains("state Worker {\n"));
    assert!(uml.contains("  Warmup --> Working : Warm\n"));
}

#[test]
fn test_serialize_the_backend() -> FsmResult<()> {
    let mut fsm = Machine::new(MachineContext::default())?;
    fsm.start()?;
    fsm.dispatch(Go { speed: 3 })?;
    let ev: WorkerEvents = Warm.into();
    fsm.dispatch(ev)?;

    let value = serde_json::to_value(Machine::serialize_backend(&fsm).unwrap()).unwrap();
    assert_eq!(json!({
        "context": { "gos": 1 },
        "states": {
            "idle": { "visits": 1 },
            "worker": {
                "context": null,
                "states": { "warmup": null, "working": { "units": 1 } },
                "current_states": ["Working"]
            },
            "led_off": null,
            "led_on": null,
            // the settings of the timer, while it runs
            "blink": null
        },
        "current_states": ["Worker", "LedOff"]
    }), value);

    let event: MachineEvents = Go { speed: 7 }.into();
    let value = serde_json::to_value(Machine::serialize_event(&event).unwrap()).unwrap();
    assert_eq!(json!({ "Go": { "speed": 7 } }), value);

    let event: MachineEvents = WorkerEvents::from(Warm).into();
    let value = serde_json::to_value(Machine::serialize_event(&event).unwrap()).unwrap();
    assert_eq!(json!({ "Worker": { "Warm": null } }), value);

    Ok(())
}

#[derive(Default, Serialize)]
pub struct Plain;

#[finny_fsm]
fn build_not_serialized(mut fsm: FsmBuilder<NotSerialized, ()>) -> BuiltFsm {
    fsm.initial_state::<Plain>();
    fsm.state::<Plain>();
    fsm.build()
}

#[test]
fn test_without_opt_in_nothing_is_serialized() -> FsmResult<()> {
    let fsm = NotSerialized::new(())?;
    assert!(NotSerialized::serialize_backend(&fsm).is_none());
    Ok(())
}

#[derive(Default, Serialize)]
pub struct AsyncContext {
    counter: AtomicUsize,
    log: Mutex<Vec<String>>
}

#[finny_fsm]
fn build_async_machine(mut fsm: FsmAsyncBuilder<AsyncMachine, AsyncContext>) -> BuiltFsm {
    fsm.serde();
    fsm.initial_state::<Idle>();
    fsm.state::<Idle>()
        .on_event::<Go>()
        .transition_to::<Running>()
        .action(async |ev, ctx, _, _| {
            ctx.counter.fetch_add(ev.speed as usize, Ordering::SeqCst);
            ctx.log.lock().unwrap().push("go".into());
        });
    fsm.state::<Running>();
    fsm.build()
}

#[tokio::test]
async fn test_serialize_an_async_backend() -> FsmResult<()> {
    let mut fsm = AsyncMachine::new(AsyncContext::default())?;
    fsm.start().await?;
    fsm.dispatch(Go { speed: 2 }).await?;

    let value = serde_json::to_value(AsyncMachine::serialize_backend(&fsm).unwrap()).unwrap();
    assert_eq!(json!({
        "context": { "counter": 2, "log": ["go"] },
        "states": { "idle": { "visits": 0 }, "running": null },
        "current_states": ["Running"]
    }), value);
    Ok(())
}

#[derive(Serialize)]
pub struct GenericContext<T> { val: T }

#[finny_fsm]
fn build_generic<TVal>(mut fsm: FsmBuilder<GenericMachine<TVal>, GenericContext<TVal>>) -> BuiltFsm
    where TVal: Serialize
{
    fsm.serde();
    fsm.initial_state::<Idle>();
    fsm.state::<Idle>();
    fsm.build()
}

#[test]
fn test_serialize_a_generic_context() -> FsmResult<()> {
    let mut fsm = GenericMachine::new(GenericContext { val: "hello" })?;
    fsm.start()?;
    let value = serde_json::to_value(GenericMachine::<&str>::serialize_backend(&fsm).unwrap()).unwrap();
    assert_eq!(json!({ "val": "hello" }), value["context"]);
    assert_eq!(type_name::<GenericContext<&str>>(), GenericMachine::<&str>::fsm_info().context_type_name);
    Ok(())
}
