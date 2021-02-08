//! An implementation of timers that relies just on the `Duration`. Has to be called with
//! a reasonable period rate to trigger the timers.

use crate::{FsmBackend, FsmTimers};
use crate::lib::*;
use Duration;
use arraydeque::{Array, ArrayDeque};

pub struct TimersCore<F, Q>
    where F: FsmBackend,
          Q: Array<Item = <F as FsmBackend>::Timers>
{
    timers: Vec<(<F as FsmBackend>::Timers, CoreTimer)>,
    pending_events: ArrayDeque<Q>
    //pending_intervals: Option<(<F as FsmBackend>::Timers, usize)>
}

#[derive(Debug)]
enum CoreTimer {
    Timeout { time_remaining: Duration },
}

impl<F, Q> TimersCore<F, Q>
    where F: FsmBackend,
    Q: Array<Item = <F as FsmBackend>::Timers>
{
    pub fn new() -> Self {
        Self {
            timers: vec![],
            pending_events: ArrayDeque::new()
            //pending_intervals: None
        }
    }

    pub fn tick(&mut self, elapsed_since_last_tick: Duration) {

    }
}

impl<F, Q> FsmTimers<F> for TimersCore<F, Q>
    where F: FsmBackend,
    Q: Array<Item = <F as FsmBackend>::Timers>
{
    fn create(&mut self, id: <F as FsmBackend>::Timers, settings: &crate::TimerSettings) -> crate::FsmResult<()> {
        todo!()
    }

    fn cancel(&mut self, id: <F as FsmBackend>::Timers) -> crate::FsmResult<()> {
        self.timers.retain(|(timer_id, _)| *timer_id != id);
        Ok(())
    }

    fn get_triggered_timer(&mut self) -> Option<<F as FsmBackend>::Timers> {
        todo!()
    }
}