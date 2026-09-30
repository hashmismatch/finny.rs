//! Run-to-completion semantics of the event queue.

use finny::{FsmCurrentState, FsmError, FsmEventQueueSender, FsmEventQueueVec, FsmFactory, FsmResult, FsmTimersNull, decl::{BuiltFsm, FsmBuilder}, finny_fsm};
use finny_tests::test_utils::QueuedErrors;

#[derive(Default)]
pub struct PingPongCtx {
    count: usize,
    limit: usize
}
#[derive(Default)]
pub struct Ping;
#[derive(Default)]
pub struct Pong;
#[derive(Clone, Debug)]
pub struct Go;

/// Every entered state enqueues the event that leaves it, until the limit is reached.
#[finny_fsm]
fn build_ping_pong(mut fsm: FsmBuilder<PingPong, PingPongCtx>) -> BuiltFsm {
    fsm.initial_state::<Ping>();

    fsm.state::<Ping>()
        .on_entry(|_, ctx| {
            ctx.context.count += 1;
            if ctx.context.count < ctx.context.limit {
                ctx.queue.enqueue(Go).unwrap();
            }
        })
        .on_event::<Go>()
        .transition_to::<Pong>();

    fsm.state::<Pong>()
        .on_entry(|_, ctx| {
            ctx.context.count += 1;
            if ctx.context.count < ctx.context.limit {
                ctx.queue.enqueue(Go).unwrap();
            }
        })
        .on_event::<Go>()
        .transition_to::<Ping>();

    fsm.build()
}

#[test]
fn test_deep_event_chain_doesnt_overflow_the_stack() -> FsmResult<()> {
    let limit = 100_000;
    let mut fsm = PingPong::new_with(PingPongCtx { count: 0, limit }, FsmEventQueueVec::new(), finny::inspect::null::InspectNull::new(), FsmTimersNull)?;

    fsm.start()?;
    fsm.dispatch_queue()?;
    assert_eq!(limit, fsm.count);

    Ok(())
}


#[derive(Default)]
pub struct StateA;
#[derive(Default)]
pub struct StateB;
#[derive(Default)]
pub struct StateC;
#[derive(Clone, Debug)]
pub struct EventB;
#[derive(Clone, Debug)]
pub struct EventC;
#[derive(Clone, Debug)]
pub struct Unhandled;

/// The initial state enqueues an event that moves the machine into B.
#[finny_fsm]
fn build_start_enqueues(mut fsm: FsmBuilder<StartEnqueues, ()>) -> BuiltFsm {
    fsm.initial_state::<StateA>();

    fsm.state::<StateA>()
        .on_entry(|_, ctx| {
            ctx.queue.enqueue(EventB).unwrap();
        })
        .on_event::<EventB>()
        .transition_to::<StateB>();

    fsm.state::<StateB>()
        .on_entry(|_, ctx| {
            ctx.queue.enqueue(Unhandled).unwrap();
        })
        .on_event::<EventC>()
        .transition_to::<StateC>();

    fsm.state::<StateC>()
        .on_event::<Unhandled>()
        .internal_transition()
        .action(|_, _, _| { });

    fsm.build()
}

#[test]
fn test_start_runs_to_completion() -> FsmResult<()> {
    let mut fsm = StartEnqueues::new(())?;

    fsm.start()?;
    assert_eq!(FsmCurrentState::State(StartEnqueuesCurrentState::StateB), fsm.get_current_states()[0]);

    // the events enqueued while starting don't overtake a later event
    fsm.dispatch(EventC)?;
    assert_eq!(FsmCurrentState::State(StartEnqueuesCurrentState::StateC), fsm.get_current_states()[0]);

    Ok(())
}

#[test]
fn test_queued_event_errors_are_reported() -> FsmResult<()> {
    let inspect = QueuedErrors::default();
    let mut fsm = StartEnqueues::new_with((), FsmEventQueueVec::new(), inspect.clone(), FsmTimersNull)?;

    // B's entry enqueues an event that B doesn't handle
    fsm.start()?;
    assert_eq!(FsmCurrentState::State(StartEnqueuesCurrentState::StateB), fsm.get_current_states()[0]);
    assert_eq!(vec![("Unhandled".to_string(), FsmError::NoTransition)], *inspect.errors.borrow());

    Ok(())
}
