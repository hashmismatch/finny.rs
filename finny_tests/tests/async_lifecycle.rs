//! Entering, leaving, restarting and stopping async machines and sub-machines.

use std::time::Duration;

use finny::{FsmAsyncFactory, FsmCurrentState, FsmEventQueueVec, FsmResult, decl::{BuiltFsm, FsmAsyncBuilder}, finny_fsm, timers::tokio::TimersTokio};
use finny_tests::test_utils::QueuedErrors;
use tokio::sync::mpsc;

#[derive(Default)]
pub struct StateA;
#[derive(Clone, Debug)]
pub struct Enter;
#[derive(Clone, Debug)]
pub struct Leave;
#[derive(Clone, Debug)]
pub struct Restart;

#[finny_fsm]
fn build_parent(mut fsm: FsmAsyncBuilder<Parent, ()>) -> BuiltFsm {
    fsm.initial_state::<StateA>();

    fsm.state::<StateA>()
        .on_event::<Enter>()
        .transition_to::<SubMachine>();

    fsm.sub_machine::<SubMachine>()
        .on_event::<Leave>()
        .transition_to::<StateA>();

    fsm.sub_machine::<SubMachine>()
        .on_event::<Restart>()
        .self_transition();

    fsm.build()
}

#[derive(Default)]
pub struct Counters {
    entries: usize,
    exits: usize,
    ticks: usize
}

#[derive(Default)]
pub struct SubA { counters: Counters }
#[derive(Default)]
pub struct SubB { counters: Counters }
#[derive(Clone, Debug)]
pub struct Next;
#[derive(Clone, Debug)]
pub struct Tick;

#[finny_fsm]
fn build_sub(mut fsm: FsmAsyncBuilder<SubMachine, ()>) -> BuiltFsm {
    fsm.initial_state::<SubA>();

    fsm.state::<SubA>()
        .on_entry(async |state, _| { state.counters.entries += 1; })
        .on_exit(async |state, _| {
            tokio::task::yield_now().await;
            state.counters.exits += 1;
        })
        .on_event::<Next>()
        .transition_to::<SubB>();

    fsm.state::<SubA>()
        .on_event::<Tick>()
        .internal_transition()
        .action(async |_, _, state| { state.counters.ticks += 1; });

    fsm.state::<SubA>()
        .on_entry_start_timer(|_, timer| {
            timer.timeout = Duration::from_millis(20);
            timer.renew = true;
        }, |_, _| Some(Tick.into()))
        .with_timer_ty::<SubTimer>();

    fsm.state::<SubB>()
        .on_entry(async |state, _| { state.counters.entries += 1; })
        .on_exit(async |state, _| { state.counters.exits += 1; });

    fsm.build()
}

fn sub_event<E: Into<SubMachineEvents>>(ev: E) -> SubMachineEvents {
    ev.into()
}

#[tokio::test(start_paused = true)]
async fn test_async_leaving_a_sub_machine_exits_it_and_cancels_its_timers() -> FsmResult<()> {
    let inspect = QueuedErrors::default();
    let mut fsm = Parent::new_with((), FsmEventQueueVec::new(), inspect.clone(), TimersTokio::new())?;
    let (_tx, mut rx) = mpsc::channel::<ParentEvents>(1);

    fsm.start().await?;
    fsm.dispatch(Enter).await?;

    // the sub machine's timer is running
    let _ = tokio::time::timeout(Duration::from_millis(50), fsm.run(&mut rx)).await;
    let sub: &SubMachine = fsm.get_state();
    let sub_a: &SubA = sub.get_state();
    assert_eq!(2, sub_a.counters.ticks);

    fsm.dispatch(Leave).await?;
    let sub: &SubMachine = fsm.get_state();
    assert_eq!(FsmCurrentState::Stopped, sub.get_current_states()[0]);
    let sub_a: &SubA = sub.get_state();
    assert_eq!(1, sub_a.counters.exits);

    // and it isn't anymore
    let _ = tokio::time::timeout(Duration::from_millis(100), fsm.run(&mut rx)).await;
    let sub: &SubMachine = fsm.get_state();
    let sub_a: &SubA = sub.get_state();
    assert_eq!(2, sub_a.counters.ticks);
    assert!(inspect.errors.borrow().is_empty(), "{:?}", inspect.errors.borrow());

    Ok(())
}

#[tokio::test]
async fn test_async_self_transition_restarts_a_sub_machine() -> FsmResult<()> {
    let mut fsm = Parent::new(())?;
    fsm.start().await?;
    fsm.dispatch(Enter).await?;
    fsm.dispatch(sub_event(Next)).await?;
    fsm.dispatch(Restart).await?;

    let sub: &SubMachine = fsm.get_state();
    assert_eq!(FsmCurrentState::State(SubMachineCurrentState::SubA), sub.get_current_states()[0]);
    let sub_a: &SubA = sub.get_state();
    assert_eq!(2, sub_a.counters.entries);
    let sub_b: &SubB = sub.get_state();
    assert_eq!(1, sub_b.counters.exits);

    Ok(())
}

#[derive(Default)]
pub struct StateX { counters: Counters }

/// Two concurrent regions, one of them a sub machine.
#[finny_fsm]
fn build_two_regions(mut fsm: FsmAsyncBuilder<TwoRegions, ()>) -> BuiltFsm {
    fsm.concurrent_regions();
    fsm.initial_states::<(StateX, SubMachine)>();

    fsm.state::<StateX>()
        .on_entry(async |state, _| { state.counters.entries += 1; })
        .on_exit(async |state, _| { state.counters.exits += 1; });

    fsm.sub_machine::<SubMachine>();

    fsm.build()
}

#[tokio::test(start_paused = true)]
async fn test_async_stop_exits_all_concurrent_regions() -> FsmResult<()> {
    let inspect = QueuedErrors::default();
    let mut fsm = TwoRegions::new_with((), FsmEventQueueVec::new(), inspect.clone(), TimersTokio::new())?;
    let (_tx, mut rx) = mpsc::channel::<TwoRegionsEvents>(1);

    fsm.start().await?;
    fsm.dispatch(sub_event(Next)).await?;

    fsm.stop().await?;
    assert_eq!([FsmCurrentState::Stopped, FsmCurrentState::Stopped], fsm.get_current_states());
    let state_x: &StateX = fsm.get_state();
    assert_eq!(1, state_x.counters.exits);
    let sub: &SubMachine = fsm.get_state();
    assert_eq!(FsmCurrentState::Stopped, sub.get_current_states()[0]);
    let sub_b: &SubB = sub.get_state();
    assert_eq!(1, sub_b.counters.exits);

    fsm.start().await?;
    let state_x: &StateX = fsm.get_state();
    assert_eq!(2, state_x.counters.entries);

    // the sub machine's timer was restarted with it, and is stopped with it
    let _ = tokio::time::timeout(Duration::from_millis(50), fsm.run(&mut rx)).await;
    fsm.stop().await?;
    let _ = tokio::time::timeout(Duration::from_millis(100), fsm.run(&mut rx)).await;
    let sub: &SubMachine = fsm.get_state();
    let sub_a: &SubA = sub.get_state();
    assert_eq!(2, sub_a.counters.ticks);

    assert!(inspect.errors.borrow().is_empty(), "{:?}", inspect.errors.borrow());

    Ok(())
}
