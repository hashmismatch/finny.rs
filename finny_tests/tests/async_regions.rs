//! Async FSMs with multiple regions, executed sequentially and concurrently.

use std::{sync::atomic::{AtomicBool, AtomicUsize, Ordering}, time::Duration};

use finny::{FsmAsyncFactory, FsmCurrentState, FsmError, FsmEvent, FsmEventQueue, FsmEventQueueSender, FsmEventQueueVec, FsmResult, FsmTimersNull, decl::{BuiltFsm, FsmAsyncBuilder}, finny_fsm, inspect::tracing::InspectTracing};
use tokio::sync::Barrier;

#[derive(Default)]
pub struct StateA;
#[derive(Default)]
pub struct StateB;
#[derive(Default)]
pub struct StateX;
#[derive(Default)]
pub struct StateY;

#[derive(Clone, Debug)]
pub struct Event;
#[derive(Clone, Debug, PartialEq)]
pub struct Enqueued { region: usize }

#[derive(Default)]
pub struct Ctx {
    a_left: AtomicBool,
    x_left: AtomicBool
}

// In both machines, region 2's guard depends on the effects of region 1's transition.

#[finny_fsm]
fn build_sequential(mut fsm: FsmAsyncBuilder<SequentialMachine, Ctx>) -> BuiltFsm {
    fsm.initial_states::<(StateA, StateX)>();

    fsm.state::<StateA>()
        .on_exit(async |_, ctx| {
            ctx.a_left.store(true, Ordering::SeqCst);
        })
        .on_event::<Event>()
        .transition_to::<StateB>();
    fsm.state::<StateB>();

    fsm.state::<StateX>()
        .on_event::<Event>()
        .transition_to::<StateY>()
        .guard(|_, ctx, _| ctx.a_left.load(Ordering::SeqCst))
        .action(async |_, ctx, _, _| {
            ctx.x_left.store(true, Ordering::SeqCst);
        });
    fsm.state::<StateY>();

    fsm.build()
}

#[finny_fsm]
fn build_concurrent_guards(mut fsm: FsmAsyncBuilder<ConcurrentGuardsMachine, Ctx>) -> BuiltFsm {
    fsm.concurrent_regions();
    fsm.initial_states::<(StateA, StateX)>();

    fsm.state::<StateA>()
        .on_exit(async |_, ctx| {
            ctx.a_left.store(true, Ordering::SeqCst);
        })
        .on_event::<Event>()
        .transition_to::<StateB>();
    fsm.state::<StateB>();

    fsm.state::<StateX>()
        .on_event::<Event>()
        .transition_to::<StateY>()
        .guard(|_, ctx, _| ctx.a_left.load(Ordering::SeqCst))
        .action(async |_, ctx, _, _| {
            ctx.x_left.store(true, Ordering::SeqCst);
        });
    fsm.state::<StateY>();

    fsm.build()
}

#[tokio::test]
async fn test_sequential_regions() -> FsmResult<()> {
    let mut fsm = SequentialMachine::new(Ctx::default())?;

    fsm.start().await?;
    assert_eq!([FsmCurrentState::State(SequentialMachineCurrentState::StateA), FsmCurrentState::State(SequentialMachineCurrentState::StateX)], fsm.get_current_states());

    // the second region's guard already sees the first region's effects, like in sync FSMs
    fsm.dispatch(Event).await?;
    assert_eq!([FsmCurrentState::State(SequentialMachineCurrentState::StateB), FsmCurrentState::State(SequentialMachineCurrentState::StateY)], fsm.get_current_states());
    assert!(fsm.x_left.load(Ordering::SeqCst));

    Ok(())
}

#[tokio::test]
async fn test_concurrent_regions_guards_see_snapshot() -> FsmResult<()> {
    let mut fsm = ConcurrentGuardsMachine::new(Ctx::default())?;

    fsm.start().await?;

    // all the guards are evaluated before any of the regions execute its transition
    fsm.dispatch(Event).await?;
    assert_eq!([FsmCurrentState::State(ConcurrentGuardsMachineCurrentState::StateB), FsmCurrentState::State(ConcurrentGuardsMachineCurrentState::StateX)], fsm.get_current_states());
    assert!(!fsm.x_left.load(Ordering::SeqCst));

    // now region 1 has left StateA
    fsm.dispatch(Event).await?;
    assert_eq!([FsmCurrentState::State(ConcurrentGuardsMachineCurrentState::StateB), FsmCurrentState::State(ConcurrentGuardsMachineCurrentState::StateY)], fsm.get_current_states());
    assert!(fsm.x_left.load(Ordering::SeqCst));

    // neither region has a transition anymore
    assert_eq!(Err(FsmError::NoTransition), fsm.dispatch(Event).await);

    Ok(())
}


/// Both regions' entry actions wait on the same barrier: the start can only complete if they
/// run concurrently. The region that finishes first enqueues its event last.
pub struct BarrierCtx {
    barrier: Barrier,
    entered: AtomicUsize
}

impl Default for BarrierCtx {
    fn default() -> Self {
        Self { barrier: Barrier::new(2), entered: AtomicUsize::new(0) }
    }
}

#[derive(Default)]
pub struct RegionOne;
#[derive(Default)]
pub struct RegionTwo;
#[derive(Default)]
pub struct RegionTwoDone { value: usize }

#[derive(Clone, Debug)]
pub struct Next;

#[finny_fsm]
fn build_barrier(mut fsm: FsmAsyncBuilder<BarrierMachine, BarrierCtx>) -> BuiltFsm {
    fsm.concurrent_regions();
    fsm.events_debug();
    fsm.initial_states::<(RegionOne, RegionTwo)>();

    fsm.state::<RegionOne>()
        .on_entry(async |_, ctx| {
            ctx.barrier.wait().await;
            // finish after region two
            tokio::time::sleep(Duration::from_millis(20)).await;
            ctx.entered.fetch_add(1, Ordering::SeqCst);
            ctx.queue.enqueue(Enqueued { region: 1 }).unwrap();
        })
        .on_event::<Enqueued>()
        .internal_transition()
        .action(async |_, _, _| { });

    fsm.state::<RegionTwo>()
        .on_entry(async |_, ctx| {
            ctx.barrier.wait().await;
            ctx.entered.fetch_add(1, Ordering::SeqCst);
            ctx.queue.enqueue(Enqueued { region: 2 }).unwrap();
        })
        .on_event::<Next>()
        .transition_to::<RegionTwoDone>()
        .action(async |_, _, _, done| {
            done.value += 1;
        });

    fsm.state::<RegionTwoDone>();

    fsm.build()
}

/// The same machine without `concurrent_regions()`.
#[finny_fsm]
fn build_barrier_sequential(mut fsm: FsmAsyncBuilder<BarrierSequentialMachine, BarrierCtx>) -> BuiltFsm {
    fsm.initial_states::<(RegionOne, RegionTwo)>();

    fsm.state::<RegionOne>()
        .on_entry(async |_, ctx| {
            ctx.barrier.wait().await;
        });

    fsm.state::<RegionTwo>()
        .on_entry(async |_, ctx| {
            ctx.barrier.wait().await;
        });

    fsm.build()
}

#[tokio::test]
async fn test_sequential_regions_dont_run_concurrently() -> FsmResult<()> {
    let mut fsm = BarrierSequentialMachine::new(BarrierCtx::default())?;

    // the first region waits for the second one, which never starts
    let res = tokio::time::timeout(Duration::from_millis(100), fsm.start()).await;
    assert!(res.is_err());

    Ok(())
}

#[tokio::test]
async fn test_concurrent_regions() -> FsmResult<()> {
    let _ = tracing_subscriber::fmt().try_init();

    let mut fsm = BarrierMachine::new_with(BarrierCtx::default(), FsmEventQueueVec::new(), InspectTracing::new(), FsmTimersNull)?;

    // only the start event, keep the enqueued events in the queue
    tokio::time::timeout(Duration::from_secs(5), fsm.dispatch_single_event(FsmEvent::Start))
        .await
        .expect("the regions didn't run concurrently")?;

    assert_eq!(2, fsm.entered.load(Ordering::SeqCst));
    assert_eq!([FsmCurrentState::State(BarrierMachineCurrentState::RegionOne), FsmCurrentState::State(BarrierMachineCurrentState::RegionTwo)], fsm.get_current_states());

    // the events are enqueued in the order of the regions
    assert_eq!(2, fsm.queue.len());
    let first = fsm.queue.dequeue();
    let second = fsm.queue.dequeue();
    assert!(matches!(first, Some(BarrierMachineEvents::Enqueued(Enqueued { region: 1 }))), "{:?}", first);
    assert!(matches!(second, Some(BarrierMachineEvents::Enqueued(Enqueued { region: 2 }))), "{:?}", second);

    // only one of the regions has a transition for this event
    fsm.dispatch(Next).await?;
    assert_eq!([FsmCurrentState::State(BarrierMachineCurrentState::RegionOne), FsmCurrentState::State(BarrierMachineCurrentState::RegionTwoDone)], fsm.get_current_states());
    let done: &RegionTwoDone = fsm.get_state();
    assert_eq!(1, done.value);

    Ok(())
}


#[derive(Default)]
pub struct Replaceable {
    value: usize
}
#[derive(Default)]
pub struct Idle;
#[derive(Default)]
pub struct Replacing;
#[derive(Default)]
pub struct Replaced;
#[derive(Clone, Debug)]
pub struct Replace;

/// The second region replaces the whole shared context.
#[finny_fsm]
fn build_replace(mut fsm: FsmAsyncBuilder<ReplaceMachine, Replaceable>) -> BuiltFsm {
    fsm.concurrent_regions();
    fsm.initial_states::<(Idle, Replacing)>();

    fsm.state::<Idle>();

    fsm.state::<Replacing>()
        .on_event::<Replace>()
        .transition_to::<Replaced>()
        .action(async |_, ctx, _, _| {
            *ctx.context = std::sync::Arc::new(Replaceable { value: ctx.value + 42 });
        });
    fsm.state::<Replaced>();

    fsm.build()
}

#[tokio::test]
async fn test_concurrent_regions_keep_a_replaced_context() -> FsmResult<()> {
    let mut fsm = ReplaceMachine::new(Replaceable { value: 1 })?;
    fsm.start().await?;
    fsm.dispatch(Replace).await?;
    assert_eq!(43, fsm.get_context().value);
    Ok(())
}


#[derive(Default)]
pub struct Timed;

/// A timer in a concurrent region, without a timers service.
#[finny_fsm]
fn build_timed(mut fsm: FsmAsyncBuilder<TimedMachine, ()>) -> BuiltFsm {
    fsm.concurrent_regions();
    fsm.initial_states::<(Timed, Idle)>();

    fsm.state::<Timed>()
        .on_entry_start_timer(|_, timer| {
            timer.timeout = Duration::from_millis(10);
        }, |_, _| None)
        .with_timer_ty::<TimedTimer>();
    fsm.state::<Idle>();

    fsm.build()
}

#[tokio::test]
async fn test_concurrent_regions_failed_timer_isnt_started() -> FsmResult<()> {
    use finny::FsmTimer;

    let mut fsm = TimedMachine::new_with((), FsmEventQueueVec::new(), finny::inspect::null::InspectNull::new(), FsmTimersNull)?;
    fsm.start().await?;

    // the timers service doesn't support timers
    let timer: &TimedTimer = fsm.get_state();
    assert!(timer.get_instance().is_none());
    Ok(())
}
