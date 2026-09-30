//! The async port of `fsm_sub.rs`: async sub-machines.

use std::sync::atomic::{AtomicUsize, Ordering};

use finny::{FsmAsyncFactory, FsmCurrentState, FsmError, FsmEventQueueVec, FsmResult, FsmTimersNull, decl::{BuiltFsm, FsmAsyncBuilder}, finny_fsm, inspect::tracing::InspectTracing};

#[derive(Default)]
pub struct MainContext {
    sub_enter: AtomicUsize,
    sub_exit: AtomicUsize,
    sub_action: AtomicUsize
}

#[derive(Default)]
pub struct StateA {
    value: usize
}

#[derive(Debug, Clone)]
pub struct Event;
#[derive(Debug, Clone)]
pub struct EventSub { n: usize }
#[derive(Debug, Clone)]
pub struct EventSubSecond;

#[finny_fsm]
fn build_fsm(mut fsm: FsmAsyncBuilder<StateMachine, MainContext>) -> BuiltFsm {
    fsm.initial_state::<StateA>();
    fsm.state::<StateA>()
        .on_event::<Event>().transition_to::<SubStateMachine>()
    ;

    fsm.sub_machine::<SubStateMachine>()
        .with_context(|ctx| SubContext { value: ctx.sub_enter.load(Ordering::SeqCst) })
        .on_entry(async |_sub, ctx| {
            tokio::task::yield_now().await;
            ctx.sub_enter.fetch_add(1, Ordering::SeqCst);
        })
        .on_exit(async |_sub, ctx| {
            ctx.sub_exit.fetch_add(1, Ordering::SeqCst);
        })
        .on_event::<Event>()
        .transition_to::<StateA>()
        .action(async |_ev, _ctx, _from, to| {
            to.value += 1;
        });

    fsm.sub_machine::<SubStateMachine>()
        .on_event::<EventSub>()
        .self_transition()
        .guard(|ev, _ctx, _| ev.n > 0)
        .action(async |_ev, ctx, _state| {
            ctx.sub_action.fetch_add(1, Ordering::SeqCst);
        });

    fsm.sub_machine::<SubStateMachine>()
        .on_event::<EventSubSecond>()
        .transition_to::<SecondSubStateMachine>();

    fsm.sub_machine::<SecondSubStateMachine>()
        .with_context(|_| SecondSubContext);

    fsm.build()
}

#[derive(Default)]
pub struct SubStateA {
    value: usize
}
#[derive(Default)]
pub struct SubStateB;
#[derive(Debug, Clone)]
pub struct SubEvent;

pub struct SubContext {
    value: usize
}

#[finny_fsm]
fn build_sub_fsm(mut fsm: FsmAsyncBuilder<SubStateMachine, SubContext>) -> BuiltFsm {
    fsm.initial_state::<SubStateA>();
    fsm.state::<SubStateA>()
        .on_entry(async |state, _ctx| {
            tokio::task::yield_now().await;
            state.value += 1;
        }).on_event::<SubEvent>()
        .transition_to::<SubStateB>()
        .action(async |_ev, _ctx, state_a, _state_b| {
            state_a.value += 1;
        });

    fsm.state::<SubStateB>();
    fsm.build()
}

#[derive(Default)]
pub struct SecondSubStateA {
    value: usize
}
#[derive(Default)]
pub struct SecondSubStateB;
#[derive(Debug, Clone)]
pub struct SecondSubEvent;

pub struct SecondSubContext;

#[finny_fsm]
fn build_second_sub_fsm(mut fsm: FsmAsyncBuilder<SecondSubStateMachine, SecondSubContext>) -> BuiltFsm {
    fsm.initial_state::<SecondSubStateA>();
    fsm.state::<SecondSubStateA>()
        .on_entry(async |state, _ctx| {
            state.value += 1;
        }).on_event::<SecondSubEvent>()
        .transition_to::<SecondSubStateB>()
        .action(async |_ev, _ctx, state_a, _state_b| {
            state_a.value += 1;
        });

    fsm.state::<SecondSubStateB>();
    fsm.build()
}


#[tokio::test]
async fn test_async_sub() -> FsmResult<()> {
    let _ = tracing_subscriber::fmt().try_init();

    let mut fsm = StateMachine::new_with(MainContext::default(), FsmEventQueueVec::new(), InspectTracing::new(), FsmTimersNull)?;

    fsm.start().await?;
    assert_eq!(FsmCurrentState::State(StateMachineCurrentState::StateA), fsm.get_current_states()[0]);

    fsm.dispatch(Event).await?;

    assert_eq!(FsmCurrentState::State(StateMachineCurrentState::SubStateMachine), fsm.get_current_states()[0]);
    let sub: &SubStateMachine = fsm.get_state();
    assert_eq!(FsmCurrentState::State(SubStateMachineCurrentState::SubStateA), sub.get_current_states()[0]);
    let state: &SubStateA = sub.get_state();
    assert_eq!(1, state.value);

    let ev: SubStateMachineEvents = SubEvent.into();
    fsm.dispatch(ev).await?;

    assert_eq!(FsmCurrentState::State(StateMachineCurrentState::SubStateMachine), fsm.get_current_states()[0]);
    let sub: &SubStateMachine = fsm.get_state();
    assert_eq!(FsmCurrentState::State(SubStateMachineCurrentState::SubStateB), sub.get_current_states()[0]);
    let state: &SubStateA = sub.get_state();
    assert_eq!(2, state.value);

    let res = fsm.dispatch(EventSub { n: 0 }).await;
    assert_eq!(Err(FsmError::NoTransition), res);
    assert_eq!(1, fsm.sub_enter.load(Ordering::SeqCst));
    assert_eq!(0, fsm.sub_exit.load(Ordering::SeqCst));
    assert_eq!(0, fsm.sub_action.load(Ordering::SeqCst));

    // a self transition exits and re-enters the sub machine, which restarts it
    fsm.dispatch(EventSub { n: 1 }).await?;
    assert_eq!(2, fsm.sub_enter.load(Ordering::SeqCst));
    assert_eq!(1, fsm.sub_exit.load(Ordering::SeqCst));
    assert_eq!(1, fsm.sub_action.load(Ordering::SeqCst));

    fsm.dispatch(Event).await?;
    assert_eq!(FsmCurrentState::State(StateMachineCurrentState::StateA), fsm.get_current_states()[0]);
    let state: &StateA = fsm.get_state();
    assert_eq!(1, state.value);

    fsm.dispatch(Event).await?;
    assert_eq!(FsmCurrentState::State(StateMachineCurrentState::SubStateMachine), fsm.get_current_states()[0]);
    let sub: &SubStateMachine = fsm.get_state();
    assert_eq!(FsmCurrentState::State(SubStateMachineCurrentState::SubStateA), sub.get_current_states()[0]);
    // the sub machine's context was created when the parent was constructed
    assert_eq!(0, sub.get_context().value);

    fsm.dispatch(EventSubSecond).await?;
    assert_eq!(FsmCurrentState::State(StateMachineCurrentState::SecondSubStateMachine), fsm.get_current_states()[0]);
    let second: &SecondSubStateMachine = fsm.get_state();
    let state: &SecondSubStateA = second.get_state();
    assert_eq!(1, state.value);

    Ok(())
}
