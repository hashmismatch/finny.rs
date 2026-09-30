//! The async port of `fsm_timers.rs`: timers driven by the event loop, on tokio's paused clock.

use std::{sync::atomic::{AtomicBool, AtomicUsize, Ordering}, time::Duration};

use finny::{AllVariants, FsmAsyncFactory, FsmCurrentState, FsmEventQueueVec, FsmResult, decl::{BuiltFsm, FsmAsyncBuilder}, finny_fsm, inspect::tracing::InspectTracing, timers::tokio::TimersTokio};
use tokio::sync::mpsc;

#[derive(Debug, Default)]
pub struct TimersMachineContext {
    exit_a: AtomicBool
}

#[derive(Default)]
pub struct StateA {
    timers: usize
}
#[derive(Default)]
pub struct StateB {

}
#[derive(Clone, Debug)]
pub struct EventClick;
#[derive(Clone, Debug)]
pub struct EventTimer { n: usize }

#[finny_fsm]
fn build_fsm(mut fsm: FsmAsyncBuilder<TimersMachine, TimersMachineContext>) -> BuiltFsm {
    fsm.events_debug();
    fsm.initial_states::<(StateA, BlinkerMachine)>();
    fsm.sub_machine::<BlinkerMachine>();

    fsm.state::<StateA>()
        .on_exit(async |_state, ctx| {
            ctx.exit_a.store(true, Ordering::SeqCst);
        })
        .on_event::<EventClick>()
        .transition_to::<StateB>()
        .guard(|_ev, _ctx, states| {
            let state: &StateA = states.as_ref();
            state.timers >= 5
        });

    fsm.state::<StateA>()
        .on_event::<EventTimer>()
        .internal_transition()
        .action(async |ev, _ctx, state| {
            tokio::task::yield_now().await;
            assert!(ev.n <= 1);
            state.timers += 1;
        });

    fsm.state::<StateA>()
        .on_entry_start_timer(|_ctx, timer| {
            timer.timeout = Duration::from_millis(100);
            timer.renew = true;
            timer.cancel_on_state_exit = true;
        }, |_ctx, _state| {
            Some( EventTimer {n: 0}.into() )
        })
        .with_timer_ty::<Timer1>();

    fsm.state::<StateA>()
        .on_entry_start_timer(|_ctx, timer| {
            timer.timeout = Duration::from_millis(200);
            timer.renew = false;
            timer.cancel_on_state_exit = true;
        }, |_ctx, _state| {
            Some( EventTimer {n: 1}.into() )
        })
        .with_timer_ty::<Timer2>();

    fsm.state::<StateB>();

    fsm.build()
}

#[derive(Default, Debug)]
pub struct LightOn;
#[derive(Default, Debug)]
pub struct LightOff;
#[derive(Default, Debug)]
pub struct BlinkingOn;
#[derive(Default, Clone, Debug)]
pub struct BlinkToggle;
#[derive(Default)]
pub struct BlinkerContext {
    toggles: AtomicUsize
}

/// The blinker runs its regions concurrently: its timer is started by the second region.
#[finny_fsm]
fn build_blinker_fsm(mut fsm: FsmAsyncBuilder<BlinkerMachine, BlinkerContext>) -> BuiltFsm {
    fsm.events_debug();
    fsm.concurrent_regions();
    fsm.initial_states::<(LightOff, BlinkingOn)>();

    fsm.state::<LightOff>()
        .on_event::<BlinkToggle>()
        .transition_to::<LightOn>()
        .action(async |_, ctx, _, _| {
            ctx.toggles.fetch_add(1, Ordering::SeqCst);
        });

    fsm.state::<LightOn>()
        .on_event::<BlinkToggle>()
        .transition_to::<LightOff>()
        .action(async |_, ctx, _, _| {
            ctx.toggles.fetch_add(1, Ordering::SeqCst);
        });

    fsm.state::<BlinkingOn>()
        .on_entry_start_timer(|_ctx, settings| {
            settings.timeout = Duration::from_millis(100);
            settings.renew = true;
        }, |_ctx, _state| {
            Some( BlinkToggle.into() )
        })
        .with_timer_ty::<BlinkingTimer>();

    fsm.build()
}


#[tokio::test(start_paused = true)]
async fn test_async_timers() -> FsmResult<()> {
    let _ = tracing_subscriber::fmt().try_init();

    let sub_timers_variants: Vec<_> = BlinkerMachineTimers::iter().collect();
    assert_eq!(&[BlinkerMachineTimers::BlinkingTimer], sub_timers_variants.as_slice());

    let mut fsm = TimersMachine::new_with(TimersMachineContext::default(), FsmEventQueueVec::new(), InspectTracing::new(), TimersTokio::new())?;
    let (tx, mut rx) = mpsc::channel(16);

    fsm.start().await?;

    // The paused clock advances to the next timer whenever the runtime is idle. Stop the
    // event loop after 450ms: Timer1 has triggered 4 times, Timer2 once.
    let res = tokio::time::timeout(Duration::from_millis(450), fsm.run(&mut rx)).await;
    assert!(res.is_err(), "the event loop should still be running");

    let state_a: &StateA = fsm.get_state();
    assert_eq!(5, state_a.timers);

    tx.send(EventClick.into()).await.unwrap();

    // until 650ms
    let res = tokio::time::timeout(Duration::from_millis(200), fsm.run(&mut rx)).await;
    assert!(res.is_err(), "the event loop should still be running");

    assert_eq!(FsmCurrentState::State(TimersMachineCurrentState::StateB), fsm.get_current_states()[0]);

    // StateA's timers were cancelled on exit
    let state_a: &StateA = fsm.get_state();
    assert_eq!(5, state_a.timers);
    assert!(fsm.exit_a.load(Ordering::SeqCst));

    // the sub machine's interval timer kept going, every 100ms
    let sub_machine: &BlinkerMachine = fsm.get_state();
    assert_eq!(6, sub_machine.toggles.load(Ordering::SeqCst));

    // the event loop finishes once all the senders are gone
    drop(tx);
    fsm.run(&mut rx).await?;

    Ok(())
}

#[tokio::test(start_paused = true)]
async fn test_async_timers_dispatch_timer_events() -> FsmResult<()> {
    // Without the event loop, the timers can still be polled.
    let mut fsm = TimersMachine::new(TimersMachineContext::default())?;
    fsm.start().await?;

    tokio::time::advance(Duration::from_millis(250)).await;
    fsm.dispatch_timer_events().await?;

    // Timer1 twice, Timer2 once
    let state_a: &StateA = fsm.get_state();
    assert_eq!(3, state_a.timers);

    Ok(())
}

/// The FSM and its event loop can run on another task of the multi-threaded runtime.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_async_fsm_is_send() -> FsmResult<()> {
    let mut fsm = TimersMachine::new_with(TimersMachineContext::default(), FsmEventQueueVec::new(), InspectTracing::new(), TimersTokio::new())?;
    let (tx, mut rx) = mpsc::channel(16);

    let task = tokio::spawn(async move {
        fsm.start().await?;
        fsm.run(&mut rx).await?;
        Ok::<_, finny::FsmError>(fsm)
    });

    tx.send(EventClick.into()).await.unwrap();
    drop(tx);

    let fsm = task.await.unwrap()?;
    // not enough timers yet for the guard
    assert_eq!(FsmCurrentState::State(TimersMachineCurrentState::StateA), fsm.get_current_states()[0]);

    Ok(())
}
