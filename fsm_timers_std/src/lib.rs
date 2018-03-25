extern crate fsm;
extern crate tokio_timer;
extern crate futures;

use fsm::*;
use fsm::timers::*;
use tokio_timer::*;
use futures::*;
use std::time::*;

use std::sync::*;

#[derive(Clone)]
pub struct FsmTimersStd {
    inner: Arc<Mutex<FsmTimersStdInner>>
}

struct FsmTimersStdInner {
    timer: Timer,
    timeout_timers: Vec<(TimerId, Sleep)>
}

impl FsmTimersStd {
    pub fn new() -> Self {
        let inner = FsmTimersStdInner {
            timer: Timer::default(),
            timeout_timers: vec![]
        };

        FsmTimersStd {
            inner: Arc::new(Mutex::new(inner))
        }
    }
}

impl FsmTimers for FsmTimersStd {
    fn create_timeout_timer(&mut self, id: TimerId, duration: TimerDuration) {
        if let Ok(mut inner) = self.inner.lock() {
            let sleep = inner.timer.sleep(Duration::from_millis(duration.ms));
            inner.timeout_timers.push((id, sleep));
        }
    }

	fn cancel_timer(&mut self, id: TimerId) {
        if let Ok(mut inner) = self.inner.lock() {
            inner.timeout_timers.retain(|t| t.0 != id)
        }
    }
	
	fn receive_events(&mut self) -> Vec<FsmTimerEvent> {
        let mut ret = vec![];
        let mut remove_timer_ids = vec![];

        if let Ok(mut inner) = self.inner.lock() {
            for &(ref id, ref timer) in &inner.timeout_timers {
                if timer.is_expired() {
                    ret.push(FsmTimerEvent::TimedOut(FsmTimerTimedOut {
                        timer_id: *id
                    }));
                    remove_timer_ids.push(*id);
                }
            }

            for id in remove_timer_ids {
                if let Some(idx) = inner.timeout_timers.iter().position(|t| t.0 == id) {
                    inner.timeout_timers.remove(idx);
                }
            }
        }

        ret
    }
}
