//! Saving the FSMs that opt in with `fsm.serde()` and restoring them.

use std::{sync::atomic::{AtomicUsize, Ordering}, time::Duration};

use finny::{FsmAsyncFactory, FsmBackend, FsmBackendImpl, FsmCurrentState, FsmEventQueueVec, FsmFactory, FsmResult,
    FsmTimers, FsmTimersNull, TimerSettings, decl::{BuiltFsm, FsmAsyncBuilder, FsmBuilder}, finny_fsm, inspect::null::InspectNull};
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct Idle { visits: usize }
#[derive(Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct Busy { jobs: Vec<String> }
#[derive(Default, Serialize, Deserialize)]
pub struct LedOff;
#[derive(Default, Serialize, Deserialize)]
pub struct LedOn { blinks: usize }

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Job { name: String }
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Done;
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Toggle;
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Blinked;

#[derive(Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct MachineContext { jobs_done: usize }

#[finny_fsm]
fn build_machine(mut fsm: FsmBuilder<Machine, MachineContext>) -> BuiltFsm {
    fsm.serde();
    fsm.events_debug();
    fsm.initial_states::<(Idle, LedOff)>();

    fsm.state::<Idle>()
        .on_entry(|state, _| { state.visits += 1; })
        .on_event::<Job>()
        .transition_to::<Worker>();

    fsm.sub_machine::<Worker>()
        .on_event::<Done>()
        .transition_to::<Idle>()
        .action(|_, ctx, _, _| { ctx.jobs_done += 1; });

    fsm.state::<LedOff>()
        .on_event::<Toggle>()
        .transition_to::<LedOn>();

    fsm.state::<LedOn>()
        .on_event::<Toggle>()
        .transition_to::<LedOff>();

    fsm.state::<LedOn>()
        .on_event::<Blinked>()
        .internal_transition()
        .action(|_, _, state| { state.blinks += 1; });

    fsm.state::<LedOn>()
        .on_entry_start_timer(|_ctx, timer| {
            timer.timeout = Duration::from_millis(250);
            timer.renew = true;
        }, |_ctx, _state| {
            Some(Blinked.into())
        })
        .with_timer_ty::<Blink>();

    fsm.build()
}

#[derive(Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct Warmup;
#[derive(Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct Working { units: u32 }
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Warm;
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Cooled;

#[finny_fsm]
fn build_worker(mut fsm: FsmBuilder<Worker, ()>) -> BuiltFsm {
    fsm.serde();
    fsm.events_debug();
    fsm.initial_state::<Warmup>();

    fsm.state::<Warmup>()
        .on_event::<Warm>()
        .transition_to::<Working>()
        .action(|_, _, _, working| { working.units += 1; });

    fsm.state::<Warmup>()
        .on_entry_start_timer(|_ctx, timer| {
            timer.timeout = Duration::from_secs(5);
        }, |_ctx, _state| {
            Some(Warm.into())
        })
        .with_timer_ty::<WarmupTimeout>();

    fsm.state::<Working>()
        .on_event::<Cooled>()
        .transition_to::<Warmup>();

    fsm.build()
}

/// Records the created timers and triggers them on demand.
#[derive(Default)]
struct ManualTimers<F: FsmBackend> {
    created: Vec<(F::Timers, Duration, bool)>,
    triggered: Vec<F::Timers>
}

impl<F: FsmBackend> FsmTimers<F> for ManualTimers<F> {
    fn create(&mut self, id: F::Timers, settings: &TimerSettings) -> FsmResult<()> {
        self.created.retain(|(i, _, _)| *i != id);
        self.created.push((id, settings.timeout, settings.renew));
        Ok(())
    }

    fn cancel(&mut self, id: F::Timers) -> FsmResult<()> {
        self.created.retain(|(i, _, _)| *i != id);
        Ok(())
    }

    fn get_triggered_timer(&mut self) -> Option<F::Timers> {
        self.triggered.pop()
    }
}

fn new_timers() -> ManualTimers<Machine> {
    ManualTimers { created: vec![], triggered: vec![] }
}

fn roundtrip(backend: &FsmBackendImpl<Machine>) -> FsmBackendImpl<Machine> {
    let json = serde_json::to_string(backend).unwrap();
    serde_json::from_str(&json).unwrap()
}

#[test]
fn test_restore_continues_where_it_left_off() -> FsmResult<()> {
    let mut fsm = Machine::new_with(MachineContext::default(), FsmEventQueueVec::new(), InspectNull::new(), new_timers())?;
    fsm.start()?;
    fsm.dispatch(Job { name: "a".into() })?;
    fsm.dispatch(WorkerEvents::from(Warm))?;
    fsm.dispatch(Done)?;
    fsm.dispatch(Job { name: "b".into() })?;

    let backend = roundtrip(&fsm.backend);
    let mut restored = Machine::restore_with(backend, FsmEventQueueVec::new(), InspectNull::new(), new_timers())?;

    assert_eq!(fsm.get_current_states(), restored.get_current_states());
    assert_eq!(MachineContext { jobs_done: 1 }, restored.context);
    assert_eq!(&Idle { visits: 2 }, restored.get_state::<Idle>());
    let worker: &Worker = restored.get_state();
    assert_eq!([FsmCurrentState::State(WorkerCurrentState::Warmup)], worker.get_current_states());
    assert_eq!(&Working { units: 1 }, worker.get_state::<Working>());

    // both machines continue in the same way
    for fsm in [&mut fsm, &mut restored] {
        fsm.dispatch(WorkerEvents::from(Warm))?;
        fsm.dispatch(Done)?;
        assert_eq!(2, fsm.context.jobs_done);
        assert_eq!(&Idle { visits: 3 }, fsm.get_state::<Idle>());
        let worker: &Worker = fsm.get_state();
        assert_eq!(&Working { units: 2 }, worker.get_state::<Working>());
    }

    Ok(())
}

#[test]
fn test_restore_a_stopped_machine() -> FsmResult<()> {
    let fsm = Machine::new(MachineContext::default())?;
    let mut restored = Machine::restore(roundtrip(&fsm.backend))?;
    assert!(FsmCurrentState::all_stopped(&restored.get_current_states()));

    restored.start()?;
    assert_eq!([FsmCurrentState::State(MachineCurrentState::Idle), FsmCurrentState::State(MachineCurrentState::LedOff)],
        restored.get_current_states());
    Ok(())
}

#[test]
fn test_restore_the_running_timers() -> FsmResult<()> {
    let mut fsm = Machine::new_with(MachineContext::default(), FsmEventQueueVec::new(), InspectNull::new(), new_timers())?;
    fsm.start()?;
    fsm.dispatch(Toggle)?;
    fsm.dispatch(Job { name: "a".into() })?;
    // the LED's blinking and the worker's warmup timeout
    assert_eq!(2, fsm.timers.created.len());

    let json = serde_json::to_value(&fsm.backend).unwrap();
    assert_eq!(serde_json::json!({
        "enabled": true,
        "timeout": { "secs": 0, "nanos": 250_000_000 },
        "renew": true,
        "cancel_on_state_exit": true
    }), json["states"]["blink"]);
    assert!(json["states"]["worker"]["states"]["warmup_timeout"].is_object());

    let backend: FsmBackendImpl<Machine> = serde_json::from_value(json).unwrap();
    let mut restored = Machine::restore_with(backend, FsmEventQueueVec::new(), InspectNull::new(), new_timers())?;

    let mut created = restored.timers.created.clone();
    created.sort_by_key(|(id, _, _)| *id);
    assert_eq!(vec![
        (MachineTimers::Worker(WorkerTimers::WarmupTimeout), Duration::from_secs(5), false),
        (MachineTimers::Blink, Duration::from_millis(250), true)
    ], created);

    // the restored timers trigger their events
    restored.timers.triggered.push(MachineTimers::Blink);
    restored.timers.triggered.push(MachineTimers::Worker(WorkerTimers::WarmupTimeout));
    restored.dispatch_timer_events()?;
    let led_on: &LedOn = restored.get_state();
    assert_eq!(1, led_on.blinks);
    let worker: &Worker = restored.get_state();
    assert_eq!([FsmCurrentState::State(WorkerCurrentState::Working)], worker.get_current_states());

    // and are cancelled on exit, as usual: the warmup's when the worker started working
    assert_eq!(vec![(MachineTimers::Blink, Duration::from_millis(250), true)], restored.timers.created);
    restored.dispatch(Toggle)?;
    assert!(restored.timers.created.is_empty());
    Ok(())
}

#[test]
fn test_restore_without_timers_drops_them() -> FsmResult<()> {
    let mut fsm = Machine::new_with(MachineContext::default(), FsmEventQueueVec::new(), InspectNull::new(), new_timers())?;
    fsm.start()?;
    fsm.dispatch(Toggle)?;

    let restored = Machine::restore_with(roundtrip(&fsm.backend), FsmEventQueueVec::new(), InspectNull::new(), FsmTimersNull)?;
    let json = serde_json::to_value(&restored.backend).unwrap();
    assert!(json["states"]["blink"].is_null());
    Ok(())
}

#[test]
fn test_invalid_current_states_are_rejected() {
    let fsm = Machine::new(MachineContext::default()).unwrap();
    let mut json = serde_json::to_value(&fsm.backend).unwrap();

    let mut with_states = |states: serde_json::Value| {
        json["current_states"] = states;
        serde_json::from_value::<FsmBackendImpl<Machine>>(json.clone()).map(|_| ()).map_err(|e| e.to_string())
    };

    assert!(with_states(serde_json::json!(["Worker", "LedOn"])).is_ok());
    assert!(with_states(serde_json::json!([null, null])).is_ok());
    // a state of the other region
    let err = with_states(serde_json::json!(["LedOn", "LedOff"])).unwrap_err();
    assert!(err.contains("isn't in the region 0"), "{}", err);
    // the wrong number of regions
    assert!(with_states(serde_json::json!(["Idle"])).is_err());
    assert!(with_states(serde_json::json!(["Idle", "LedOff", null])).is_err());
    // an unknown state
    assert!(with_states(serde_json::json!(["Nope", "LedOff"])).is_err());
}

#[test]
fn test_events_roundtrip() {
    let events: Vec<MachineEvents> = vec![Job { name: "x".into() }.into(), WorkerEvents::from(Cooled).into(), Toggle.into()];
    let json = serde_json::to_string(&events).unwrap();
    let restored: Vec<MachineEvents> = serde_json::from_str(&json).unwrap();
    assert_eq!(format!("{:?}", events), format!("{:?}", restored));
}

#[derive(Default, Serialize, Deserialize)]
pub struct AsyncContext { counter: AtomicUsize }

#[finny_fsm]
fn build_async_machine(mut fsm: FsmAsyncBuilder<AsyncMachine, AsyncContext>) -> BuiltFsm {
    fsm.serde();
    fsm.initial_state::<Idle>();
    fsm.state::<Idle>()
        .on_event::<Job>()
        .transition_to::<Busy>()
        .action(async |ev, ctx, _, busy| {
            ctx.counter.fetch_add(1, Ordering::SeqCst);
            busy.jobs.push(ev.name.clone());
        });
    fsm.state::<Busy>()
        .on_event::<Done>()
        .transition_to::<Idle>();
    fsm.build()
}

#[tokio::test]
async fn test_restore_an_async_machine() -> FsmResult<()> {
    let mut fsm = AsyncMachine::new(AsyncContext::default())?;
    fsm.start().await?;
    fsm.dispatch(Job { name: "a".into() }).await?;

    let json = serde_json::to_string(&fsm.backend).unwrap();
    let backend: FsmBackendImpl<AsyncMachine> = serde_json::from_str(&json).unwrap();
    let mut restored = AsyncMachine::restore(backend)?;

    assert_eq!(1, restored.context.counter.load(Ordering::SeqCst));
    assert_eq!(&Busy { jobs: vec!["a".into()] }, restored.get_state::<Busy>());

    restored.dispatch(Done).await?;
    restored.dispatch(Job { name: "b".into() }).await?;
    assert_eq!(2, restored.context.counter.load(Ordering::SeqCst));
    assert_eq!(&Busy { jobs: vec!["a".into(), "b".into()] }, restored.get_state::<Busy>());
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct GenericContext<T> { val: T }

#[finny_fsm]
fn build_generic<TVal>(mut fsm: FsmBuilder<GenericMachine<TVal>, GenericContext<TVal>>) -> BuiltFsm
    where TVal: Serialize
{
    fsm.serde();
    fsm.initial_state::<Idle>();
    fsm.state::<Idle>()
        .on_entry(|state, _| { state.visits += 1; });
    fsm.build()
}

#[test]
fn test_restore_a_generic_machine() -> FsmResult<()> {
    let mut fsm = GenericMachine::new(GenericContext { val: 42u64 })?;
    fsm.start()?;

    let json = serde_json::to_string(&fsm.backend).unwrap();
    let backend: FsmBackendImpl<GenericMachine<u64>> = serde_json::from_str(&json).unwrap();
    let restored = GenericMachine::restore(backend)?;
    assert_eq!(42, restored.context.val);
    assert_eq!(&Idle { visits: 1 }, restored.get_state::<Idle>());
    Ok(())
}
