//! The inspectors see a consistent picture of the dispatching.

use finny::{FsmAsyncFactory, FsmEventQueueVec, FsmFactory, FsmResult, FsmTimersNull, decl::{BuiltFsm, FsmAsyncBuilder, FsmBuilder}, finny_fsm};
use finny_tests::test_utils::EventCounts;

#[derive(Default)]
pub struct StateA;
#[derive(Clone, Debug)]
pub struct Enter;
#[derive(Default)]
pub struct SubA;
#[derive(Default)]
pub struct SubB;
#[derive(Clone, Debug)]
pub struct Next;

#[finny_fsm]
fn build_parent(mut fsm: FsmBuilder<Parent, ()>) -> BuiltFsm {
    fsm.initial_state::<StateA>();
    fsm.state::<StateA>().on_event::<Enter>().transition_to::<Sub>();
    fsm.sub_machine::<Sub>();
    fsm.build()
}

#[finny_fsm]
fn build_sub(mut fsm: FsmBuilder<Sub, ()>) -> BuiltFsm {
    fsm.initial_state::<SubA>();
    fsm.state::<SubA>().on_event::<Next>().transition_to::<SubB>();
    fsm.state::<SubB>();
    fsm.build()
}

#[test]
fn test_every_dispatched_event_is_done() -> FsmResult<()> {
    let inspect = EventCounts::default();
    let mut fsm = Parent::new_with((), FsmEventQueueVec::new(), inspect.clone(), FsmTimersNull)?;
    fsm.start()?;
    fsm.dispatch(Enter)?;
    // forwarded to the sub machine
    let ev: SubEvents = Next.into();
    fsm.dispatch(ev)?;

    assert_eq!(*inspect.new_events.borrow(), *inspect.events_done.borrow());
    Ok(())
}

#[test]
fn test_start_enters_the_state_once() -> FsmResult<()> {
    let inspect = EventCounts::default();
    let mut fsm = Parent::new_with((), FsmEventQueueVec::new(), inspect.clone(), FsmTimersNull)?;
    fsm.start()?;

    assert_eq!(1, *inspect.state_enters.borrow());
    Ok(())
}

#[finny_fsm]
fn build_async_parent(mut fsm: FsmAsyncBuilder<AsyncParent, ()>) -> BuiltFsm {
    fsm.initial_state::<StateA>();
    fsm.state::<StateA>().on_event::<Enter>().transition_to::<AsyncSub>();
    fsm.sub_machine::<AsyncSub>();
    fsm.build()
}

#[finny_fsm]
fn build_async_sub(mut fsm: FsmAsyncBuilder<AsyncSub, ()>) -> BuiltFsm {
    fsm.initial_state::<SubA>();
    fsm.state::<SubA>().on_event::<Next>().transition_to::<SubB>();
    fsm.state::<SubB>();
    fsm.build()
}

#[tokio::test]
async fn test_async_every_dispatched_event_is_done() -> FsmResult<()> {
    let inspect = EventCounts::default();
    let mut fsm = AsyncParent::new_with((), FsmEventQueueVec::new(), inspect.clone(), FsmTimersNull)?;
    fsm.start().await?;
    fsm.dispatch(Enter).await?;
    let ev: AsyncSubEvents = Next.into();
    fsm.dispatch(ev).await?;

    assert_eq!(*inspect.new_events.borrow(), *inspect.events_done.borrow());
    // StateA, Sub and its SubA, SubB
    assert_eq!(4, *inspect.state_enters.borrow());
    Ok(())
}
