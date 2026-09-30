//! The inspectors see a consistent picture of the dispatching.

use std::time::Duration;

use finny::{FsmAsyncFactory, FsmBackend, FsmEventQueueVec, FsmFactory, FsmResult, FsmTimers, FsmTimersNull, TimerSettings,
    decl::{BuiltFsm, FsmAsyncBuilder, FsmBuilder}, finny_fsm, inspect::chain::InspectChain};
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

#[derive(Default)]
pub struct Waiting;
#[derive(Default)]
pub struct Paused;
#[derive(Clone, Debug)]
pub struct Tick;
#[derive(Clone, Debug)]
pub struct Pause;

#[finny_fsm]
fn build_ticker(mut fsm: FsmBuilder<Ticker, ()>) -> BuiltFsm {
    fsm.initial_state::<Waiting>();
    fsm.state::<Waiting>()
        .on_event::<Tick>()
        .internal_transition();
    fsm.state::<Waiting>()
        .on_event::<Pause>()
        .transition_to::<Paused>();
    fsm.state::<Waiting>()
        .on_entry_start_timer(|_, timer| {
            timer.timeout = Duration::from_millis(10);
            timer.renew = true;
        }, |_, _| Some(Tick.into()))
        .with_timer_ty::<TickTimer>();
    fsm.state::<Paused>()
        .on_entry_start_timer(|_, timer| {
            timer.enabled = false;
        }, |_, _| None)
        .with_timer_ty::<NeverTimer>();
    fsm.build()
}

/// Triggers the timers on demand.
struct ManualTimers<F: FsmBackend>(Vec<F::Timers>);

impl<F: FsmBackend> FsmTimers<F> for ManualTimers<F> {
    fn create(&mut self, _id: F::Timers, _settings: &TimerSettings) -> FsmResult<()> { Ok(()) }
    fn cancel(&mut self, _id: F::Timers) -> FsmResult<()> { Ok(()) }
    fn get_triggered_timer(&mut self) -> Option<F::Timers> { self.0.pop() }
}

#[test]
fn test_the_timers_lifecycle_is_inspected() -> FsmResult<()> {
    let (a, b) = (EventCounts::default(), EventCounts::default());
    let inspect = InspectChain::new_pair(a.clone(), b.clone());
    let mut fsm = Ticker::new_with((), FsmEventQueueVec::new(), inspect, ManualTimers(vec![]))?;
    fsm.start()?;
    fsm.timers.0.push(TickerTimers::TickTimer);
    fsm.dispatch_timer_events()?;
    fsm.dispatch(Pause)?;

    let expected = vec!["TickTimer Started", "TickTimer Triggered", "TickTimer Cancelled", "NeverTimer Disabled"];
    assert_eq!(expected, *a.timers.borrow());
    // forwarded by the chain
    assert_eq!(expected, *b.timers.borrow());

    // the timers service fails to create it
    let c = EventCounts::default();
    let mut fsm = Ticker::new_with((), FsmEventQueueVec::new(), c.clone(), FsmTimersNull)?;
    fsm.start()?;
    assert_eq!(vec!["TickTimer Failed"], *c.timers.borrow());
    Ok(())
}
