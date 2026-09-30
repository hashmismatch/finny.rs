//! The async port of `fsm_fn.rs`.

use std::sync::atomic::{AtomicUsize, Ordering};

use finny::{FsmAsyncFactory, FsmCurrentState, FsmError, FsmEventQueueVec, FsmResult, FsmTimersNull, decl::{BuiltFsm, FsmAsyncBuilder}, finny_fsm, inspect::tracing::InspectTracing};

/// Async FSMs share the context through an `Arc`, the mutable parts use interior mutability.
#[derive(Debug, Default)]
pub struct StateMachineContext {
    count: AtomicUsize,
    total_time: AtomicUsize
}

#[derive(Default)]
pub struct StateA {
    enter: usize,
    exit: usize
}
#[derive(Default)]
pub struct StateB {
    counter: usize
}
#[derive(Clone, Debug)]
pub struct EventClick { time: usize }
#[derive(Clone, Debug)]
pub struct EventEnter { shift: bool }

/// Something to await on in the actions.
async fn io() {
    tokio::task::yield_now().await;
}

#[finny_fsm]
fn build_fsm(mut fsm: FsmAsyncBuilder<StateMachine, StateMachineContext>) -> BuiltFsm {
    fsm.events_debug();
    fsm.initial_state::<StateA>();

    fsm.state::<StateA>()
        .on_entry(async |state_a, ctx| {
            io().await;
            ctx.count.fetch_add(1, Ordering::SeqCst);
            state_a.enter += 1;
        })
        .on_exit(async |state_a, ctx| {
            io().await;
            ctx.count.fetch_add(1, Ordering::SeqCst);
            state_a.exit += 1;
        })
        .on_event::<EventClick>()
        .transition_to::<StateB>()
        .guard(|ev, _, states| {
            let state_a: &StateA = states.as_ref();
            ev.time > 100 && state_a.enter == 1
        })
        .action(async |ev, ctx, state_from, _state_to| {
            io().await;
            ctx.total_time.fetch_add(ev.time, Ordering::SeqCst);
            assert_eq!(0, state_from.exit - 1);
        });

    fsm.state::<StateB>()
        .on_entry(async |state_b, _| {
            io().await;
            state_b.counter += 1;
        })
        .on_event::<EventEnter>()
        .internal_transition()
        .guard(|ev, _, _| {
            ev.shift == false
        })
        .action(async |_, ctx, state_b| {
            io().await;
            state_b.counter += 1;
            // the context can be moved into spawned tasks
            let ctx = ctx.context.clone();
            tokio::spawn(async move { ctx.count.fetch_add(100, Ordering::SeqCst); }).await.unwrap();
        });

    fsm.build()
}


#[tokio::test]
async fn test_async_fsm() -> FsmResult<()> {
    let _ = tracing_subscriber::fmt().try_init();

    let ctx = StateMachineContext::default();

    let mut fsm = StateMachine::new_with(ctx, FsmEventQueueVec::new(), InspectTracing::new(), FsmTimersNull)?;

    let current_state = fsm.get_current_states()[0];
    let state: &StateA = fsm.get_state();
    assert_eq!(0, state.enter);
    assert_eq!(FsmCurrentState::Stopped, current_state);
    assert_eq!(0, fsm.get_context().count.load(Ordering::SeqCst));

    fsm.start().await?;

    assert_eq!(FsmCurrentState::State(StateMachineCurrentState::StateA), fsm.get_current_states()[0]);
    assert_eq!(1, fsm.get_context().count.load(Ordering::SeqCst));
    let state: &StateA = fsm.get_state();
    assert_eq!(1, state.enter);

    let ret = fsm.dispatch(EventClick { time: 99 }).await;
    assert_eq!(Err(FsmError::NoTransition), ret);

    fsm.dispatch(EventClick { time: 123 }).await?;

    assert_eq!(2, fsm.get_context().count.load(Ordering::SeqCst));
    assert_eq!(123, fsm.get_context().total_time.load(Ordering::SeqCst));

    let state_b: &StateB = fsm.get_state();
    assert_eq!(1, state_b.counter);

    let ret = fsm.dispatch(EventEnter { shift: true }).await;
    assert_eq!(Err(FsmError::NoTransition), ret);

    fsm.dispatch(EventEnter { shift: false }).await?;
    let state_b: &StateB = fsm.get_state();
    assert_eq!(2, state_b.counter);
    assert_eq!(102, fsm.get_context().count.load(Ordering::SeqCst));

    Ok(())
}

#[tokio::test]
async fn test_async_fsm_default_frontend() -> FsmResult<()> {
    let mut fsm = StateMachine::new(StateMachineContext::default())?;
    fsm.start().await?;
    fsm.dispatch(EventClick { time: 123 }).await?;
    assert_eq!(FsmCurrentState::State(StateMachineCurrentState::StateB), fsm.get_current_states()[0]);
    Ok(())
}
