use crate::{FsmError, lib::*};
use crate::{FsmBackend, FsmResult};


pub trait FsmTimer<F, S>
    where F: FsmBackend, Self: Default
{
    fn setup(ctx: &<F as FsmBackend>::Context, settings: &mut TimerFsmSettings);
    fn trigger(ctx: &<F as FsmBackend>::Context, state: &S) -> Option< <F as FsmBackend>::Events >;
}

#[derive(Debug, Clone)]
pub struct TimerFsmSettings {
    pub enabled: bool,
    pub timeout: Duration,
    pub renew: bool,
    pub cancel_on_state_exit: bool
}

impl TimerFsmSettings {
    pub fn to_timer_settings(&self) -> TimerSettings {
        TimerSettings {
            enabled: self.enabled,
            timeout: self.timeout,
            renew: self.renew
        }
    }
}

impl Default for TimerFsmSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            timeout: Duration::from_secs(1),
            renew: false,
            cancel_on_state_exit: true
        }
    }
}


#[derive(Debug, Clone)]
pub struct TimerSettings
{
    pub enabled: bool,
    pub timeout: Duration,
    pub renew: bool
}

pub type TimerId = usize;

pub trait FsmTimers {
    fn create(&mut self, id: TimerId, settings: &TimerSettings) -> FsmResult<()>;
    fn cancel(&mut self, id: TimerId) -> FsmResult<()>;
    
    /// Return the timer that was triggered. Poll this until it returns None. The events
    /// should be dequeued in a FIFO manner.
    fn get_triggered_timer(&mut self) -> Option<TimerId>;
}


#[derive(Debug, Copy, Clone)]
pub struct FsmTimersTriggerEventsResult {
    pub triggered_events: usize
}

#[derive(Debug, Default, Copy, Clone)]
pub struct FsmTimersNull;

impl FsmTimers for FsmTimersNull {
    fn create(&mut self, id: TimerId, settings: &TimerSettings) -> FsmResult<()> {
        Err(FsmError::NotSupported)
    }

    fn cancel(&mut self, id: TimerId) -> FsmResult<()> {
        Err(FsmError::NotSupported)
    }

    fn get_triggered_timer(&mut self) -> Option<TimerId> {
        None
    }
}