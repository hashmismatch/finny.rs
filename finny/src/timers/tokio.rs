//! Timers for async FSMs, based on the tokio runtime's clock. Honors tokio's paused test clock.

use std::time::Duration;
use ::tokio::time::{Instant, sleep_until};

use crate::{FsmBackend, FsmResult, FsmTimers, FsmTimersAsync, TimerSettings};

struct TokioTimer<T> {
    id: T,
    deadline: Instant,
    interval: Option<Duration>
}

/// Timers for async FSMs. The pending timers are kept in a list, no tasks are spawned. Waiting
/// for the next timer sleeps until the earliest deadline.
pub struct TimersTokio<F>
    where F: FsmBackend
{
    timers: Vec<TokioTimer<<F as FsmBackend>::Timers>>
}

impl<F> TimersTokio<F>
    where F: FsmBackend
{
    pub fn new() -> Self {
        Self {
            timers: vec![]
        }
    }
}

impl<F> Default for TimersTokio<F>
    where F: FsmBackend
{
    fn default() -> Self {
        Self::new()
    }
}

impl<F> FsmTimers<F> for TimersTokio<F>
    where F: FsmBackend
{
    fn create(&mut self, id: <F as FsmBackend>::Timers, settings: &TimerSettings) -> FsmResult<()> {
        settings.validate()?;

        // replace any existing ones
        self.cancel(id.clone())?;

        self.timers.push(TokioTimer {
            id,
            deadline: Instant::now() + settings.timeout,
            interval: if settings.renew { Some(settings.timeout) } else { None }
        });

        Ok(())
    }

    fn cancel(&mut self, id: <F as FsmBackend>::Timers) -> FsmResult<()> {
        self.timers.retain(|t| t.id != id);
        Ok(())
    }

    /// Returns the earliest timer that is due. An interval timer that missed several ticks is
    /// returned once for every missed tick.
    fn get_triggered_timer(&mut self) -> Option<<F as FsmBackend>::Timers> {
        let now = Instant::now();
        let (idx, _) = self.timers.iter()
            .enumerate()
            .filter(|(_, t)| t.deadline <= now)
            .min_by_key(|(_, t)| t.deadline)?;

        let timer = &mut self.timers[idx];
        match timer.interval {
            Some(interval) => {
                timer.deadline += interval;
                Some(timer.id.clone())
            },
            None => Some(self.timers.remove(idx).id)
        }
    }
}

impl<F> FsmTimersAsync<F> for TimersTokio<F>
    where F: FsmBackend
{
    async fn next_timer(&mut self) -> <F as FsmBackend>::Timers {
        loop {
            match self.timers.iter().map(|t| t.deadline).min() {
                // The state is only changed after the sleep has finished, which makes this cancel safe.
                Some(deadline) => sleep_until(deadline).await,
                None => core::future::pending().await
            }

            if let Some(id) = self.get_triggered_timer() {
                return id;
            }
        }
    }
}
