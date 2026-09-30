//! The timer implementations, used directly.

use std::{thread::sleep, time::Duration};

use finny::{FsmError, FsmTimers, TimerSettings, decl::{BuiltFsm, FsmBuilder}, finny_fsm, timers::{core::{CoreTimer, TimersCore}, std::TimersStd, std_noalloc::{StdTimer, TimersStdNoAlloc}}};

#[derive(Default)]
pub struct StateA;
#[derive(Default)]
pub struct StateB;
#[derive(Clone, Debug)]
pub struct Next;

/// Only for its timers enum.
#[finny_fsm]
fn build_fsm(mut fsm: FsmBuilder<TimersFsm, ()>) -> BuiltFsm {
    fsm.initial_state::<StateA>();
    fsm.state::<StateA>()
        .on_event::<Next>()
        .transition_to::<StateB>();
    fsm.state::<StateA>()
        .on_entry_start_timer(|_, _| { }, |_, _| None)
        .with_timer_ty::<T1>();
    fsm.state::<StateB>()
        .on_entry_start_timer(|_, _| { }, |_, _| None)
        .with_timer_ty::<T2>();
    fsm.build()
}

fn interval(ms: u64) -> TimerSettings {
    TimerSettings { enabled: true, timeout: Duration::from_millis(ms), renew: true }
}

fn timeout(ms: u64) -> TimerSettings {
    TimerSettings { enabled: true, timeout: Duration::from_millis(ms), renew: false }
}

/// A cancelled interval timer that missed several ticks mustn't trigger the timer that
/// replaced it.
fn cancel_clears_pending_ticks<T: FsmTimers<TimersFsm>>(mut timers: T) {
    timers.create(TimersFsmTimers::T1, &interval(10)).unwrap();
    sleep(Duration::from_millis(55));
    assert_eq!(Some(TimersFsmTimers::T1), timers.get_triggered_timer());

    timers.cancel(TimersFsmTimers::T1).unwrap();
    timers.create(TimersFsmTimers::T1, &interval(1000)).unwrap();
    assert_eq!(None, timers.get_triggered_timer());
}

fn zero_interval_is_rejected<T: FsmTimers<TimersFsm>>(mut timers: T) {
    assert_eq!(Err(FsmError::InvalidTimerSettings), timers.create(TimersFsmTimers::T1, &interval(0)));
    assert_eq!(None, timers.get_triggered_timer());
    // a zero timeout is fine for a timer that doesn't renew
    assert_eq!(Ok(()), timers.create(TimersFsmTimers::T2, &timeout(0)));
}

fn std_noalloc() -> TimersStdNoAlloc<TimersFsm, TimersFsmTimersStorage<StdTimer>> {
    TimersStdNoAlloc::new(TimersFsmTimersStorage::default())
}

fn core() -> TimersCore<TimersFsm, TimersFsmTimersStorage<CoreTimer>, [TimersFsmTimers; 4]> {
    TimersCore::new(TimersFsmTimersStorage::default())
}

#[test]
fn test_std_cancel_clears_pending_ticks() {
    cancel_clears_pending_ticks(TimersStd::<TimersFsm>::new());
}

#[test]
fn test_std_noalloc_cancel_clears_pending_ticks() {
    cancel_clears_pending_ticks(std_noalloc());
}

#[test]
fn test_std_zero_interval_is_rejected() {
    zero_interval_is_rejected(TimersStd::<TimersFsm>::new());
}

#[test]
fn test_std_noalloc_zero_interval_is_rejected() {
    zero_interval_is_rejected(std_noalloc());
}

#[test]
fn test_core_zero_interval_is_rejected() {
    zero_interval_is_rejected(core());
}

#[tokio::test(start_paused = true)]
async fn test_tokio_zero_interval_is_rejected() {
    zero_interval_is_rejected(finny::timers::tokio::TimersTokio::<TimersFsm>::new());
}

#[test]
fn test_core_cancel_removes_triggered_timer() {
    let mut timers = core();
    timers.create(TimersFsmTimers::T1, &timeout(10)).unwrap();
    timers.tick(Duration::from_millis(20));

    // the timer was triggered, but cancelled before it was dispatched
    timers.cancel(TimersFsmTimers::T1).unwrap();
    timers.create(TimersFsmTimers::T1, &timeout(10)).unwrap();
    assert_eq!(None, timers.get_triggered_timer());
}

#[test]
fn test_core_full_buffer_keeps_the_timer_due() {
    let mut timers: TimersCore<TimersFsm, TimersFsmTimersStorage<CoreTimer>, [TimersFsmTimers; 1]> = TimersCore::new(TimersFsmTimersStorage::default());
    timers.create(TimersFsmTimers::T1, &timeout(10)).unwrap();
    timers.create(TimersFsmTimers::T2, &timeout(10)).unwrap();

    // only one of the two triggered timers fits
    timers.tick(Duration::from_millis(20));
    let first = timers.get_triggered_timer();
    assert!(first.is_some());
    assert_eq!(None, timers.get_triggered_timer());

    // the other one is still due
    timers.tick(Duration::ZERO);
    let second = timers.get_triggered_timer();
    assert!(second.is_some());
    assert_ne!(first, second);
}
