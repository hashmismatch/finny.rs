use std::{collections::{HashMap, hash_map::RandomState}, time::{Duration, Instant}};
use crate::{FsmTimers, TimerId};

pub struct TimersStd {
    timers: Vec<(TimerId, StdTimer)>,
    pending_intervals: Option<(TimerId, usize)>
}

enum StdTimer {
    Timeout { started_at: Instant, duration: Duration },
    Interval { started_at: Instant, interval: Duration }
}

impl TimersStd {
    pub fn new() -> Self {
        Self {
            timers: vec![],
            pending_intervals: None
        }
    }
}

impl FsmTimers for TimersStd {
    fn create(&mut self, id: crate::TimerId, settings: &crate::TimerSettings) -> crate::FsmResult<()> {
        if settings.renew {
            self.timers.push((id, StdTimer::Interval { started_at: Instant::now(), interval: settings.timeout }));
        } else {
            self.timers.push((id, StdTimer::Timeout { started_at: Instant::now(), duration: settings.timeout }));
        }
        Ok(())
    }

    fn cancel(&mut self, id: crate::TimerId) -> crate::FsmResult<()> {
        todo!("cancel timer")
    }

    fn get_triggered_timer(&mut self) -> Option<crate::TimerId> {
        if let Some((id, mut times)) = self.pending_intervals.take() {
            times -= 1;
            if times > 0 {
                self.pending_intervals = Some((id, times));
            }

            return Some(id);
        }


        let mut timed_out_idx = None;
        let now = Instant::now();
        for (idx, (timer_id, timer)) in self.timers.iter_mut().enumerate() {
            match timer {
                StdTimer::Timeout { started_at, duration } if now.duration_since(*started_at) >= *duration => {
                    timed_out_idx = Some(idx);
                    break;
                },
                StdTimer::Interval { ref mut started_at, interval } if now.duration_since(*started_at) >= *interval => {
                    let t = now.duration_since(*started_at);
                    let times = ((t.as_secs_f32() / interval.as_secs_f32()).floor() as usize) - 1;
                    if times > 0 {
                        self.pending_intervals = Some((*timer_id, times));
                    }
                    *started_at = now;
                    return Some(*timer_id);
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