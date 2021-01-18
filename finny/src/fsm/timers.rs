use crate::lib::*;
use crate::{FsmBackend, FsmResult};


pub trait FsmTimer<F, S>
    where F: FsmBackend
{
    fn setup(ctx: &<F as FsmBackend>::Context, settings: &mut TimerSettings);
    fn trigger(ctx: &<F as FsmBackend>::Context, state: &S) -> Option< <F as FsmBackend>::Events >;
}

#[derive(Debug, Clone)]
pub struct TimerFsmSettings {
    pub timeout: Duration,
    pub renew: bool,
    pub cancel_on_state_exit: bool
}

impl TimerFsmSettings {
    pub fn to_timer_settings(&self) -> TimerSettings {
        TimerSettings {
            timeout: self.timeout,
            renew: self.renew
        }
    }
}

impl Default for TimerFsmSettings {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(1),
            renew: false,
            cancel_on_state_exit: true
        }
    }
}


#[derive(Debug, Clone)]
pub struct TimerSettings
{
    pub timeout: Duration,
    pub renew: bool
}

pub type TimerId = usize;

pub trait FsmTimers {
    fn create(&mut self, id: TimerId, settings: TimerSettings) -> FsmResult<()>;
    fn cancel(&mut self, id: TimerId) -> FsmResult<()>;
    
    /// Return the latest timer that was triggered. Poll this until it returns None.
    fn get_triggered_timer(&mut self) -> Option<TimerId>;
}


#[derive(Debug, Copy, Clone)]
pub struct FsmTimersTriggerEventsResult {
    pub triggered_events: usize
}