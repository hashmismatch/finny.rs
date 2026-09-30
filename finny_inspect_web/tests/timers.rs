//! The timers' state in the snapshots.

use std::{any::type_name, time::Duration};

use finny::{FsmBackend, FsmBackendImpl, FsmEventQueueVec, FsmFactory, FsmResult, FsmTimers, FsmTimersNull, TimerSettings,
    decl::{BuiltFsm, FsmBuilder}, finny_fsm};
use finny_inspect_web::{Inspector, InspectorConfig, snapshot::{Snapshot, TimerState, TimerStatus}};
use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize)]
pub struct Idle;
#[derive(Default, Serialize, Deserialize)]
pub struct Blinking { blinks: u32 }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Go;
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Stop;
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Work;
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Blink;

#[finny_fsm]
fn build_machine(mut fsm: FsmBuilder<Machine, ()>) -> BuiltFsm {
    fsm.serde();
    fsm.initial_state::<Idle>();

    fsm.state::<Idle>()
        .on_event::<Go>()
        .transition_to::<Blinking>();
    fsm.state::<Idle>()
        .on_event::<Work>()
        .transition_to::<Worker>();

    fsm.state::<Blinking>()
        .on_event::<Stop>()
        .transition_to::<Idle>();
    fsm.state::<Blinking>()
        .on_event::<Blink>()
        .internal_transition()
        .action(|_, _, state| { state.blinks += 1; });
    fsm.state::<Blinking>()
        .on_entry_start_timer(|_, timer| {
            timer.timeout = Duration::from_millis(400);
            timer.renew = true;
        }, |_, _| Some(Blink.into()))
        .with_timer_ty::<BlinkTimer>();
    fsm.state::<Blinking>()
        .on_entry_start_timer(|_, timer| {
            timer.timeout = Duration::from_secs(2);
        }, |_, _| Some(Blink.into()))
        .with_timer_ty::<Deadline>();

    fsm.sub_machine::<Worker>()
        .on_event::<Stop>()
        .transition_to::<Idle>();

    fsm.build()
}

#[derive(Default, Serialize, Deserialize)]
pub struct Warmup;
#[derive(Default, Serialize, Deserialize)]
pub struct Working;
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Warm;

#[finny_fsm]
fn build_worker(mut fsm: FsmBuilder<Worker, ()>) -> BuiltFsm {
    fsm.serde();
    fsm.initial_state::<Warmup>();
    fsm.state::<Warmup>()
        .on_event::<Warm>()
        .transition_to::<Working>();
    fsm.state::<Warmup>()
        .on_entry_start_timer(|_, timer| {
            timer.timeout = Duration::from_millis(1500);
        }, |_, _| Some(Warm.into()))
        .with_timer_ty::<WarmupTimeout>();
    fsm.state::<Working>();
    fsm.build()
}

/// Triggers the timers on demand.
struct ManualTimers<F: FsmBackend> {
    triggered: Vec<F::Timers>
}

impl<F: FsmBackend> FsmTimers<F> for ManualTimers<F> {
    fn create(&mut self, _id: F::Timers, _settings: &TimerSettings) -> FsmResult<()> { Ok(()) }
    fn cancel(&mut self, _id: F::Timers) -> FsmResult<()> { Ok(()) }
    fn get_triggered_timer(&mut self) -> Option<F::Timers> { self.triggered.pop() }
}

fn last(inspector: &Inspector, name: &str) -> Snapshot {
    inspector.snapshots(name).unwrap().pop().unwrap()
}

fn timer<'a>(snapshot: &'a Snapshot, path: &[&str], timer: &str) -> &'a TimerStatus {
    snapshot.timers.iter()
        .find(|t| t.path.iter().map(|p| p.as_str()).eq(path.iter().copied()) && t.timer == timer)
        .unwrap_or_else(|| panic!("no timer {} in {:?}", timer, snapshot.timers))
}

#[test]
fn test_the_lifecycle_of_the_timers() -> FsmResult<()> {
    let inspector = Inspector::new(InspectorConfig::default());
    let mut fsm = Machine::new_with((), FsmEventQueueVec::new(), inspector.attach::<Machine>("main"), ManualTimers { triggered: vec![] })?;

    fsm.start()?;
    assert!(last(&inspector, "main").timers.is_empty());

    fsm.dispatch(Go)?;
    let s = last(&inspector, "main");
    let blink = timer(&s, &[], "BlinkTimer");
    assert_eq!((TimerState::Running, 400.0, true, 0, None, false),
        (blink.status, blink.timeout_ms, blink.renew, blink.triggers, blink.last_triggered_ms, blink.restored));
    let deadline = timer(&s, &[], "Deadline");
    assert_eq!((TimerState::Running, 2000.0, false), (deadline.status, deadline.timeout_ms, deadline.renew));

    // an interval keeps running
    fsm.timers.triggered.push(MachineTimers::BlinkTimer);
    fsm.dispatch_timer_events()?;
    fsm.timers.triggered.push(MachineTimers::BlinkTimer);
    fsm.dispatch_timer_events()?;
    let s = last(&inspector, "main");
    let blink = timer(&s, &[], "BlinkTimer");
    assert_eq!((TimerState::Running, 2), (blink.status, blink.triggers));
    assert!(blink.last_triggered_ms.is_some());

    // a timeout expires
    fsm.timers.triggered.push(MachineTimers::Deadline);
    fsm.dispatch_timer_events()?;
    let deadline = timer(&last(&inspector, "main"), &[], "Deadline").clone();
    assert_eq!((TimerState::Expired, 1), (deadline.status, deadline.triggers));

    // the running one is cancelled when the state is exited, the expired one stays expired
    fsm.dispatch(Stop)?;
    let s = last(&inspector, "main");
    assert_eq!(TimerState::Cancelled, timer(&s, &[], "BlinkTimer").status);
    assert_eq!(TimerState::Expired, timer(&s, &[], "Deadline").status);

    // a sub-machine's timer
    fsm.dispatch(Work)?;
    let s = last(&inspector, "main");
    let warmup = timer(&s, &[type_name::<Worker>()], "WarmupTimeout");
    assert_eq!((TimerState::Running, 1500.0), (warmup.status, warmup.timeout_ms));
    // the parents' timers first
    assert!(s.timers.last().unwrap().path.len() == 1);

    // started again: the counts are reset
    fsm.dispatch(Stop)?;
    fsm.dispatch(Go)?;
    let blink = timer(&last(&inspector, "main"), &[], "BlinkTimer").clone();
    assert_eq!((TimerState::Running, 0, None), (blink.status, blink.triggers, blink.last_triggered_ms));

    Ok(())
}

#[test]
fn test_timers_that_failed_to_start() -> FsmResult<()> {
    let inspector = Inspector::new(InspectorConfig::default());
    let mut fsm = Machine::new_with((), FsmEventQueueVec::new(), inspector.attach::<Machine>("main"), FsmTimersNull)?;
    fsm.start()?;
    fsm.dispatch(Go)?;
    assert_eq!(TimerState::Failed, timer(&last(&inspector, "main"), &[], "BlinkTimer").status);
    Ok(())
}

#[test]
fn test_restored_timers() -> FsmResult<()> {
    let inspector = Inspector::new(InspectorConfig::default());
    let mut fsm = Machine::new_with((), FsmEventQueueVec::new(), inspector.attach::<Machine>("main"), ManualTimers { triggered: vec![] })?;
    fsm.start()?;
    fsm.dispatch(Work)?;

    let json = serde_json::to_string(&fsm.backend).unwrap();
    let backend: FsmBackendImpl<Machine> = serde_json::from_str(&json).unwrap();
    let mut restored = Machine::restore_with(backend, FsmEventQueueVec::new(), inspector.attach::<Machine>("restored"), ManualTimers { triggered: vec![] })?;

    // restoring isn't an event, the timers are in the next snapshot
    assert!(inspector.snapshots("restored").unwrap().is_empty());
    restored.timers.triggered.push(MachineTimers::Worker(WorkerTimers::WarmupTimeout));
    restored.dispatch_timer_events()?;

    let s = last(&inspector, "restored");
    let warmup = timer(&s, &[type_name::<Worker>()], "WarmupTimeout");
    assert_eq!((TimerState::Expired, 1, true), (warmup.status, warmup.triggers, warmup.restored));

    // the inspectors created while restoring don't detach the instance
    assert!(inspector.instances().iter().find(|i| i.id == "restored").unwrap().attached);
    Ok(())
}
