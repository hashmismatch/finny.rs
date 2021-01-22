use crate::{DispatchContext, FsmError, FsmEventQueue, Inspect, lib::*};
use crate::{FsmBackend, FsmResult};

#[derive(Debug, Clone, Copy)]
pub struct TimerInstance {
    pub id: TimerId,
    pub settings: TimerFsmSettings
}

pub trait FsmTimer<F, S>
    where F: FsmBackend, Self: Default
{
    fn setup(ctx: &<F as FsmBackend>::Context, settings: &mut TimerFsmSettings);
    fn trigger(ctx: &<F as FsmBackend>::Context, state: &S) -> Option< <F as FsmBackend>::Events >;

    fn get_instance(&self) -> &Option<TimerInstance>;
    fn get_instance_mut(&mut self) -> &mut Option<TimerInstance>;

    fn execute_on_enter<I: Inspect, T: FsmTimers>(&mut self, id: TimerId, ctx: &<F as FsmBackend>::Context, inspect: &mut I, timers: &mut T) {
        let log = inspect.for_timer(id);
        let mut settings = TimerFsmSettings::default();
        Self::setup(ctx, &mut settings);
        if settings.enabled {
            match timers.create(id, &settings.to_timer_settings()) {
                Ok(_) => {
                    let instance = self.get_instance_mut();
                    *instance = Some( TimerInstance { id, settings } );
                    log.info("Started the timer.");
                },
                Err(ref e) => {
                    log.on_error("Failed to create a timer", e);
                }
            }
        } else {
            log.info("The timer wasn't enabled.");
        }
    }

    fn execute_on_exit<I: Inspect, T: FsmTimers>(&mut self, id: TimerId, inspect: &mut I, timers: &mut T) {
        let log = inspect.for_timer(id);
        match self.get_instance_mut() {
            Some(instance) => {
                if id == instance.id && instance.settings.cancel_on_state_exit {
                    match timers.cancel(id) {
                        Ok(_) => {
                            *self.get_instance_mut() = None;
                            log.info("Cancelled the timer.");
                        },
                        Err(ref e) => {
                            log.on_error("Failed to cancel the timer", e);
                        }
                    }
                }
            },
            _ => ()
        }
    }

    fn execute_trigger<'a, 'b, 'c, 'd, Q, I, T>(id: TimerId, context: &'d mut DispatchContext<'a, 'b, 'c, F, Q, I, T>, inspect: &mut I)
        where Q: FsmEventQueue<F>, I: Inspect, <F as FsmBackend>::States: AsRef<S>, T: FsmTimers
    {
        let inspect = inspect.for_timer(id);
        //match self.get_instance() {
        //    Some(_) => {                
                match Self::trigger(&context.backend.context, context.backend.states.as_ref()) {
                    Some(ev) => {
                        match context.queue.enqueue(ev) {
                            Ok(_) => {
                                inspect.info("The event triggered by the timer was enqueued.");
                            },
                            Err(e) => {
                                inspect.on_error("The event triggered by the timer couldn't be enqueued.", &e);
                            }
                        }
                    },
                    _ => ()
                }

            //},
            //None => {
            //    let error = FsmError::TimerNotStarted(id);
            //    inspect.on_error("Timer hasn't been started.", &error);
            //}
        //}
    }
}

#[derive(Debug, Clone, Copy)]
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