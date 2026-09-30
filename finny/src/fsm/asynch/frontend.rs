use crate::lib::*;
use crate::{DispatchContext, FsmBackend, FsmBackendImpl, FsmError, FsmEvent, FsmEventQueue, FsmEventQueueVec, FsmResult,
    FsmTimers, Inspect, inspect::null::InspectNull, timers::tokio::TimersTokio};

use super::{FsmAsyncDispatch, FsmTimersAsync};

/// The frontend of an async state machine, which also includes environmental services like queues,
/// timers and inspection. The usual way to use the async FSM.
pub struct FsmAsyncFrontend<F, Q, I, T>
    where F: FsmAsyncDispatch, Q: FsmEventQueue<F>, I: Inspect, T: FsmTimers<F>
{
    pub backend: FsmBackendImpl<F>,
    pub queue: Q,
    pub inspect: I,
    pub timers: T
}

impl<F, Q, I, T> FsmAsyncFrontend<F, Q, I, T>
    where F: FsmAsyncDispatch, Q: FsmEventQueue<F>, I: Inspect, T: FsmTimers<F>
{
    /// Start the FSM, initiates the transition to the initial state and runs the events
    /// enqueued by it to completition.
    pub async fn start(&mut self) -> FsmResult<()> {
        self.dispatch_single_event(FsmEvent::Start).await?;
        self.dispatch_queue().await
    }

    /// Stop the FSM: exits the active states of all the regions, including their timers and
    /// sub-machines. The FSM can be started again.
    pub async fn stop(&mut self) -> FsmResult<()> {
        self.dispatch_single_event(FsmEvent::Stop).await?;
        self.dispatch_queue().await
    }

    /// Dispatch any pending timer events into the queue, then run all the
    /// events from the queue until completition.
    pub async fn dispatch_timer_events(&mut self) -> FsmResult<()> {
        while let Some(timer_id) = self.timers.get_triggered_timer() {
            self.dispatch_single_event(FsmEvent::Timer(timer_id)).await?;
        }

        self.dispatch_queue().await
    }

    /// Dispatch this event and run it to completition.
    pub async fn dispatch<E>(&mut self, event: E) -> FsmResult<()>
        where E: Into<<F as FsmBackend>::Events>
    {
        self.dispatch_single_event(FsmEvent::Event(event.into())).await?;
        self.dispatch_queue().await
    }

    /// Dispatch only this event, do not run it to completition.
    pub async fn dispatch_single_event(&mut self, event: FsmEvent<<F as FsmBackend>::Events, <F as FsmBackend>::Timers>) -> FsmResult<()> {
        let dispatch_ctx = DispatchContext {
            backend: &mut self.backend,
            inspect: &mut self.inspect,
            queue: &mut self.queue,
            timers: &mut self.timers
        };

        F::dispatch_event(dispatch_ctx, event).await
    }

    /// Dispatch the entire event queue and run it to completition. The events that fail,
    /// usually because the current state doesn't handle them, are reported to the inspector
    /// with `Inspect::on_queued_event_error`.
    pub async fn dispatch_queue(&mut self) -> FsmResult<()> {
        // A loop, the events enqueued by the dispatched events are appended to the same queue.
        while let Some(ev) = self.queue.dequeue() {
            if let Err(e) = self.dispatch_single_event(FsmEvent::Event(ev.clone())).await {
                self.inspect.on_queued_event_error::<F>(&ev, &e);
            }
        }

        Ok(())
    }
}

enum RunNext<E, T> {
    Event(Option<E>),
    Timer(T)
}

impl<F, Q, I, T> FsmAsyncFrontend<F, Q, I, T>
    where F: FsmAsyncDispatch, Q: FsmEventQueue<F>, I: Inspect, T: FsmTimersAsync<F>
{
    /// Runs the FSM: dispatches the received events and the triggered timers, each one run to
    /// completition, until the sender side of the channel is closed. Events without a matching
    /// transition are skipped, other errors stop the loop.
    ///
    /// Start the machine with [`start`](Self::start) first. Cancelling the returned future is
    /// safe while it is waiting for the next event or timer, but not while it is dispatching
    /// one, as that can leave the FSM in the middle of a transition.
    pub async fn run(&mut self, events: &mut ::tokio::sync::mpsc::Receiver<<F as FsmBackend>::Events>) -> FsmResult<()> {
        loop {
            // Only the waiting is raced, the dispatching itself is never cancelled.
            let next = ::tokio::select! {
                biased;
                ev = events.recv() => RunNext::Event(ev),
                timer_id = self.timers.next_timer() => RunNext::Timer(timer_id)
            };

            let result = match next {
                RunNext::Event(Some(ev)) => self.dispatch(ev).await,
                RunNext::Event(None) => return Ok(()),
                RunNext::Timer(timer_id) => {
                    match self.dispatch_single_event(FsmEvent::Timer(timer_id)).await {
                        Ok(()) => self.dispatch_queue().await,
                        Err(e) => Err(e)
                    }
                }
            };

            match result {
                Ok(()) | Err(FsmError::NoTransition) => (),
                Err(e) => return Err(e)
            }
        }
    }
}

impl<F, Q, I, T> Deref for FsmAsyncFrontend<F, Q, I, T>
    where F: FsmAsyncDispatch, Q: FsmEventQueue<F>, I: Inspect, T: FsmTimers<F>
{
    type Target = FsmBackendImpl<F>;

    fn deref(&self) -> &Self::Target {
        &self.backend
    }
}

impl<F, Q, I, T> DerefMut for FsmAsyncFrontend<F, Q, I, T>
    where F: FsmAsyncDispatch, Q: FsmEventQueue<F>, I: Inspect, T: FsmTimers<F>
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.backend
    }
}

/// Builds a frontend for running your async FSM.
pub trait FsmAsyncFactory {
    type Fsm: FsmAsyncDispatch;

    /// For submachines, for use with codegen.
    fn new_submachine_backend(backend: FsmBackendImpl<Self::Fsm>) -> FsmResult<Self> where Self: Sized;

    /// Build a new frontend for the FSM with all the environmental services provided by the caller.
    fn new_with<Q, I, T>(context: impl Into<<Self::Fsm as FsmBackend>::Context>, queue: Q, inspect: I, timers: T) -> FsmResult<FsmAsyncFrontend<Self::Fsm, Q, I, T>>
        where Q: FsmEventQueue<Self::Fsm>, I: Inspect, T: FsmTimers<Self::Fsm>
    {
        let frontend = FsmAsyncFrontend {
            queue,
            inspect,
            backend: FsmBackendImpl::new(context.into())?,
            timers
        };

        Ok(frontend)
    }

    /// Build a new frontend for the FSM with a `FsmEventQueueVec` queue, `TimersTokio` for timers and no logging.
    fn new(context: impl Into<<Self::Fsm as FsmBackend>::Context>) -> FsmResult<FsmAsyncFrontend<Self::Fsm, FsmEventQueueVec<Self::Fsm>, InspectNull, TimersTokio<Self::Fsm>>> {
        Self::new_with(context, FsmEventQueueVec::new(), InspectNull::new(), TimersTokio::new())
    }

    /// Build a frontend for a deserialized machine, with all the environmental services provided
    /// by the caller. The machine continues from its restored states, it doesn't have to be
    /// started. Its running timers are re-created with their full timeouts.
    #[cfg(feature = "serde")]
    fn restore_with<Q, I, T>(backend: FsmBackendImpl<Self::Fsm>, queue: Q, inspect: I, timers: T) -> FsmResult<FsmAsyncFrontend<Self::Fsm, Q, I, T>>
        where Q: FsmEventQueue<Self::Fsm>, I: Inspect, T: FsmTimers<Self::Fsm>
    {
        let mut frontend = FsmAsyncFrontend {
            queue,
            inspect,
            backend,
            timers
        };
        <Self::Fsm as FsmBackend>::restore_timers(&mut frontend.backend, &frontend.inspect, &mut frontend.timers);

        Ok(frontend)
    }

    /// Build a frontend for a deserialized machine with a `FsmEventQueueVec` queue, `TimersTokio`
    /// for timers and no logging. See [`restore_with`](Self::restore_with).
    #[cfg(feature = "serde")]
    fn restore(backend: FsmBackendImpl<Self::Fsm>) -> FsmResult<FsmAsyncFrontend<Self::Fsm, FsmEventQueueVec<Self::Fsm>, InspectNull, TimersTokio<Self::Fsm>>> {
        Self::restore_with(backend, FsmEventQueueVec::new(), InspectNull::new(), TimersTokio::new())
    }
}
