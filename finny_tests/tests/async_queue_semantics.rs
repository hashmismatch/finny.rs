//! Run-to-completion semantics of the event queue, for async FSMs.

use finny::{FsmAsyncFactory, FsmCurrentState, FsmError, FsmEventQueueSender, FsmEventQueueVec, FsmResult, FsmTimersNull, decl::{BuiltFsm, FsmAsyncBuilder}, finny_fsm};
use finny_tests::test_utils::QueuedErrors;

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
fn build_start_enqueues(mut fsm: FsmAsyncBuilder<StartEnqueues, ()>) -> BuiltFsm {
    fsm.initial_state::<StateA>();

    fsm.state::<StateA>()
        .on_entry(async |_, ctx| {
            ctx.queue.enqueue(EventB).unwrap();
        })
        .on_event::<EventB>()
        .transition_to::<StateB>();

    fsm.state::<StateB>()
        .on_entry(async |_, ctx| {
            ctx.queue.enqueue(Unhandled).unwrap();
        })
        .on_event::<EventC>()
        .transition_to::<StateC>();

    fsm.state::<StateC>()
        .on_event::<Unhandled>()
        .internal_transition()
        .action(async |_, _, _| { });

    fsm.build()
}

#[tokio::test]
async fn test_async_start_runs_to_completion() -> FsmResult<()> {
    let inspect = QueuedErrors::default();
    let mut fsm = StartEnqueues::new_with((), FsmEventQueueVec::new(), inspect.clone(), FsmTimersNull)?;

    fsm.start().await?;
    assert_eq!(FsmCurrentState::State(StartEnqueuesCurrentState::StateB), fsm.get_current_states()[0]);
    // B's entry enqueued an event that B doesn't handle
    assert_eq!(vec![("Unhandled".to_string(), FsmError::NoTransition)], *inspect.errors.borrow());

    fsm.dispatch(EventC).await?;
    assert_eq!(FsmCurrentState::State(StartEnqueuesCurrentState::StateC), fsm.get_current_states()[0]);

    Ok(())
}
