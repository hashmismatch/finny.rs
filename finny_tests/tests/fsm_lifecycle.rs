//! Entering, leaving, restarting and stopping machines and sub-machines.

use std::{thread::sleep, time::Duration};

use finny::{FsmCurrentState, FsmEventQueueVec, FsmFactory, FsmResult, decl::{BuiltFsm, FsmBuilder}, finny_fsm, timers::std::TimersStd};
use finny_tests::test_utils::QueuedErrors;

#[derive(Default)]
pub struct StateA;
#[derive(Clone, Debug)]
pub struct Enter;
#[derive(Clone, Debug)]
pub struct Leave;
#[derive(Clone, Debug)]
pub struct Restart;

#[finny_fsm]
fn build_parent(mut fsm: FsmBuilder<Parent, ()>) -> BuiltFsm {
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
fn build_sub(mut fsm: FsmBuilder<SubMachine, ()>) -> BuiltFsm {
    fsm.initial_state::<SubA>();

    fsm.state::<SubA>()
        .on_entry(|state, _| { state.counters.entries += 1; })
        .on_exit(|state, _| { state.counters.exits += 1; })
        .on_event::<Next>()
        .transition_to::<SubB>();

    fsm.state::<SubA>()
        .on_event::<Tick>()
        .internal_transition()
        .action(|_, _, state| { state.counters.ticks += 1; });

    fsm.state::<SubA>()
        .on_entry_start_timer(|_, timer| {
            timer.timeout = Duration::from_millis(20);
            timer.renew = true;
        }, |_, _| Some(Tick.into()))
        .with_timer_ty::<SubTimer>();

    fsm.state::<SubB>()
        .on_entry(|state, _| { state.counters.entries += 1; })
        .on_exit(|state, _| { state.counters.exits += 1; });

    fsm.build()
}

fn sub_event<E: Into<SubMachineEvents>>(ev: E) -> SubMachineEvents {
    ev.into()
}

#[test]
fn test_leaving_a_sub_machine_exits_its_state() -> FsmResult<()> {
    let mut fsm = Parent::new(())?;
    fsm.start()?;
    fsm.dispatch(Enter)?;
    fsm.dispatch(sub_event(Next))?;

    let sub: &SubMachine = fsm.get_state();
    assert_eq!(FsmCurrentState::State(SubMachineCurrentState::SubB), sub.get_current_states()[0]);

    fsm.dispatch(Leave)?;

    let sub: &SubMachine = fsm.get_state();
    assert_eq!(FsmCurrentState::Stopped, sub.get_current_states()[0]);
    let sub_b: &SubB = sub.get_state();
    assert_eq!(1, sub_b.counters.entries);
    assert_eq!(1, sub_b.counters.exits);

    Ok(())
}

#[test]
fn test_leaving_a_sub_machine_cancels_its_timers() -> FsmResult<()> {
    let inspect = QueuedErrors::default();
    let mut fsm = Parent::new_with((), FsmEventQueueVec::new(), inspect.clone(), TimersStd::new())?;
    fsm.start()?;
    fsm.dispatch(Enter)?;
    fsm.dispatch(Leave)?;

    // the sub machine's timer would have triggered a couple of times by now
    sleep(Duration::from_millis(70));
    fsm.dispatch_timer_events()?;

    let sub: &SubMachine = fsm.get_state();
    let sub_a: &SubA = sub.get_state();
    assert_eq!(1, sub_a.counters.exits);
    assert_eq!(0, sub_a.counters.ticks);
    assert!(inspect.errors.borrow().is_empty(), "{:?}", inspect.errors.borrow());

    Ok(())
}

#[test]
fn test_self_transition_restarts_a_sub_machine() -> FsmResult<()> {
    let mut fsm = Parent::new(())?;
    fsm.start()?;
    fsm.dispatch(Enter)?;
    fsm.dispatch(sub_event(Next))?;
    fsm.dispatch(Restart)?;

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

/// Two regions, one of them a sub machine.
#[finny_fsm]
fn build_two_regions(mut fsm: FsmBuilder<TwoRegions, ()>) -> BuiltFsm {
    fsm.initial_states::<(StateX, SubMachine)>();

    fsm.state::<StateX>()
        .on_entry(|state, _| { state.counters.entries += 1; })
        .on_exit(|state, _| { state.counters.exits += 1; });

    fsm.sub_machine::<SubMachine>();

    fsm.build()
}

#[test]
fn test_stop_exits_all_regions() -> FsmResult<()> {
    let inspect = QueuedErrors::default();
    let mut fsm = TwoRegions::new_with((), FsmEventQueueVec::new(), inspect.clone(), TimersStd::new())?;
    fsm.start()?;
    fsm.dispatch(sub_event(Next))?;

    fsm.stop()?;
    assert_eq!([FsmCurrentState::Stopped, FsmCurrentState::Stopped], fsm.get_current_states());
    let state_x: &StateX = fsm.get_state();
    assert_eq!(1, state_x.counters.exits);
    let sub: &SubMachine = fsm.get_state();
    assert_eq!(FsmCurrentState::Stopped, sub.get_current_states()[0]);
    let sub_b: &SubB = sub.get_state();
    assert_eq!(1, sub_b.counters.exits);

    // stopping a stopped machine
    assert_eq!(Err(finny::FsmError::NoTransition), fsm.stop());

    // and it can be started again
    fsm.start()?;
    assert_eq!([FsmCurrentState::State(TwoRegionsCurrentState::StateX), FsmCurrentState::State(TwoRegionsCurrentState::SubMachine)], fsm.get_current_states());
    let state_x: &StateX = fsm.get_state();
    assert_eq!(2, state_x.counters.entries);
    let sub: &SubMachine = fsm.get_state();
    assert_eq!(FsmCurrentState::State(SubMachineCurrentState::SubA), sub.get_current_states()[0]);

    assert!(inspect.errors.borrow().is_empty(), "{:?}", inspect.errors.borrow());

    Ok(())
}

#[test]
fn test_stopped_machine_ignores_its_timers() -> FsmResult<()> {
    let inspect = QueuedErrors::default();
    let mut fsm = TwoRegions::new_with((), FsmEventQueueVec::new(), inspect.clone(), TimersStd::new())?;
    fsm.start()?;
    fsm.stop()?;

    sleep(Duration::from_millis(70));
    fsm.dispatch_timer_events()?;

    let sub: &SubMachine = fsm.get_state();
    let sub_a: &SubA = sub.get_state();
    assert_eq!(0, sub_a.counters.ticks);
    assert!(inspect.errors.borrow().is_empty(), "{:?}", inspect.errors.borrow());

    Ok(())
}
