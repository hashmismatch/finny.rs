use std::{collections::{HashMap, hash_map::RandomState}, time::{Duration, Instant}};
use crate::{FsmTimers, TimerId};

pub struct TimersStd {
    timers: Vec<(TimerId, StdTimer)>
}

enum StdTimer {
    Oneshot { started_at: Instant, duration: Duration }
}

impl TimersStd {
    pub fn new() -> Self {
        Self {
            timers: vec![]
        }
    }
}

impl FsmTimers for TimersStd {
    fn create(&mut self, id: crate::TimerId, settings: &crate::TimerSettings) -> crate::FsmResult<()> {
        self.timers.push((id, StdTimer::Oneshot { started_at: Instant::now(), duration: settings.timeout }));
        Ok(())
    }

    fn cancel(&mut self, id: crate::TimerId) -> crate::FsmResult<()> {
        todo!()
    }

    fn get_triggered_timer(&mut self) -> Option<crate::TimerId> {
        let mut timed_out_idx = None;
        let now = Instant::now();
        for (idx, (_, timer)) in self.timers.iter().enumerate() {
            match timer {
                StdTimer::Oneshot { started_at, duration } if now.duration_since(*started_at) >= *duration => {
                    timed_out_idx = Some(idx);
                    break;
                },
                _ => ()
            }
        }

        if let Some(idx) = timed_out_idx {
            let (id, _) = self.timers.remove(idx);
            return Some(id);
        }

        None
    }
}