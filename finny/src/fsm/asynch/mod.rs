//! Async FSMs: the runtime traits that the code generator implements for FSMs declared with
//! `FsmAsyncBuilder`. The actions are async, while the guards, queues, timers bookkeeping and
//! inspection stay synchronous and are shared with the synchronous FSMs.

mod frontend;
mod transitions;

pub use self::frontend::*;
pub use self::transitions::*;

use crate::lib::*;
use crate::{DispatchContext, EventContext, FsmBackend, FsmBackendImpl, FsmCurrentState, FsmDispatchResult, FsmEvent,
    FsmEventQueue, FsmEventQueueSender, FsmEventQueueSub, FsmEventQueueVec, FsmRegionId, FsmStates, FsmTimers, FsmTimersSub,
    Inspect, TimerSettings};

/// Asynchronous event dispatching, implemented by the code generator.
#[allow(async_fn_in_trait)]
pub trait FsmAsyncDispatch: FsmBackend {
    async fn dispatch_event<Q, I, T>(ctx: DispatchContext<'_, '_, '_, Self, Q, I, T>, event: FsmEvent<Self::Events, Self::Timers>) -> FsmDispatchResult
        where Q: FsmEventQueue<Self>, I: Inspect, T: FsmTimers<Self>;
}

/// Async timers. Extends the synchronous timers trait with a way to wait for the next
/// triggered timer, so the FSM can be driven by an event loop.
#[allow(async_fn_in_trait)]
pub trait FsmTimersAsync<F: FsmBackend>: FsmTimers<F> {
    /// Waits for the next triggered timer. Never completes if no timers are running.
    ///
    /// Has to be cancel safe, the event loop uses it in `tokio::select!`.
    async fn next_timer(&mut self) -> <F as FsmBackend>::Timers;
}

impl<F: FsmBackend> FsmTimersAsync<F> for crate::FsmTimersNull {
    async fn next_timer(&mut self) -> <F as FsmBackend>::Timers {
        core::future::pending().await
    }
}

/// The part of the FSM that a single region operates on while executing a transition. For
/// sequential dispatching, `S` are all the states of the FSM. With concurrent regions, `S` is a
/// generated view with the region's own states only.
pub struct RegionContext<'r, F, S, Q, T>
    where F: FsmBackend
{
    pub states: &'r mut S,
    pub context: &'r mut <F as FsmBackend>::Context,
    pub current_state: &'r mut FsmCurrentState<<<F as FsmBackend>::States as FsmStates<F>>::StateKind>,
    pub queue: &'r mut Q,
    pub timers: &'r mut T,
    pub region: FsmRegionId
}

impl<'r, F, S, Q, T> RegionContext<'r, F, S, Q, T>
    where F: FsmBackend, Q: FsmEventQueueSender<F>
{
    /// The context that is given to the actions.
    pub fn event_context(&mut self) -> EventContext<'_, F, Q> {
        EventContext {
            context: &mut *self.context,
            queue: &mut *self.queue,
            region: self.region
        }
    }
}

impl<'a, 'b, 'c, F, Q, I, T> DispatchContext<'a, 'b, 'c, F, Q, I, T>
    where F: FsmBackend, Q: FsmEventQueue<F>, I: Inspect, T: FsmTimers<F>
{
    /// A region context over all of the FSM's states.
    pub fn region_context(&mut self, region: FsmRegionId) -> RegionContext<'_, F, <F as FsmBackend>::States, Q, T> {
        RegionContext {
            states: &mut self.backend.states,
            context: &mut self.backend.context,
            current_state: &mut self.backend.current_states.as_mut()[region],
            queue: &mut *self.queue,
            timers: &mut *self.timers,
            region
        }
    }
}

/// Funnels the event down to an async sub-machine.
pub async fn dispatch_to_submachine_async<F, TSubMachine, S, Q, T, I>(rc: &mut RegionContext<'_, F, S, Q, T>,
    ev: FsmEvent<<TSubMachine as FsmBackend>::Events, <TSubMachine as FsmBackend>::Timers>, inspect: &I) -> FsmDispatchResult
    where
        F: FsmBackend,
        S: AsMut<TSubMachine>,
        <F as FsmBackend>::Events: From<<TSubMachine as FsmBackend>::Events>,
        <F as FsmBackend>::Timers: From<<TSubMachine as FsmBackend>::Timers>,
        TSubMachine: FsmAsyncDispatch + DerefMut<Target = FsmBackendImpl<TSubMachine>>,
        Q: FsmEventQueue<F>,
        T: FsmTimers<F>,
        I: Inspect
{
    let sub_fsm: &mut TSubMachine = rc.states.as_mut();

    let mut queue_adapter = FsmEventQueueSub {
        parent: &mut *rc.queue,
        _parent_fsm: PhantomData::<F>::default(),
        _sub_fsm: PhantomData::<TSubMachine>::default()
    };

    let mut timers_adapter = FsmTimersSub {
        parent: &mut *rc.timers,
        _parent_fsm: PhantomData::<F>::default(),
        _sub_fsm: PhantomData::<TSubMachine>::default()
    };

    let mut inspect = inspect.for_sub_machine::<TSubMachine>();

    let sub_dispatch_ctx = DispatchContext {
        backend: sub_fsm,
        inspect: &mut inspect,
        queue: &mut queue_adapter,
        timers: &mut timers_adapter
    };

    TSubMachine::dispatch_event(sub_dispatch_ctx, ev).await
}

/// Starts an async sub-machine after its state was entered, unless it is already running.
pub async fn start_submachine_async<F, TSubMachine, S, Q, T, I>(rc: &mut RegionContext<'_, F, S, Q, T>, inspect: &I) -> FsmDispatchResult
    where
        F: FsmBackend,
        S: AsMut<TSubMachine>,
        <F as FsmBackend>::Events: From<<TSubMachine as FsmBackend>::Events>,
        <F as FsmBackend>::Timers: From<<TSubMachine as FsmBackend>::Timers>,
        TSubMachine: FsmAsyncDispatch + DerefMut<Target = FsmBackendImpl<TSubMachine>>,
        Q: FsmEventQueue<F>,
        T: FsmTimers<F>,
        I: Inspect
{
    let sub_fsm: &mut TSubMachine = rc.states.as_mut();
    let states = sub_fsm.get_current_states();
    if FsmCurrentState::all_stopped(states.as_ref()) {
        return dispatch_to_submachine_async::<F, TSubMachine, S, Q, T, I>(rc, FsmEvent::Start, inspect).await;
    }

    Ok(())
}

/// Stops an async sub-machine before its state is exited: exits its active states, including
/// their timers and the nested sub-machines.
pub async fn stop_submachine_async<F, TSubMachine, S, Q, T, I>(rc: &mut RegionContext<'_, F, S, Q, T>, inspect: &I) -> FsmDispatchResult
    where
        F: FsmBackend,
        S: AsMut<TSubMachine>,
        <F as FsmBackend>::Events: From<<TSubMachine as FsmBackend>::Events>,
        <F as FsmBackend>::Timers: From<<TSubMachine as FsmBackend>::Timers>,
        TSubMachine: FsmAsyncDispatch + DerefMut<Target = FsmBackendImpl<TSubMachine>>,
        Q: FsmEventQueue<F>,
        T: FsmTimers<F>,
        I: Inspect
{
    let sub_fsm: &mut TSubMachine = rc.states.as_mut();
    if FsmCurrentState::all_stopped(sub_fsm.get_current_states().as_ref()) {
        return Ok(());
    }

    dispatch_to_submachine_async::<F, TSubMachine, S, Q, T, I>(rc, FsmEvent::Stop, inspect).await
}

/// The FSM's timers, shared by the regions that execute concurrently. The regions run within
/// the same task and use the timers synchronously, so the lock is never contended.
pub struct FsmTimersShared<'a, F, T>
    where F: FsmBackend, T: FsmTimers<F>
{
    timers: std::sync::Mutex<&'a mut T>,
    _fsm: PhantomData<fn() -> F>
}

impl<'a, F, T> FsmTimersShared<'a, F, T>
    where F: FsmBackend, T: FsmTimers<F>
{
    pub fn new(timers: &'a mut T) -> Self {
        Self {
            timers: std::sync::Mutex::new(timers),
            _fsm: PhantomData
        }
    }

    /// The timers for one of the regions.
    pub fn region(&self) -> FsmTimersRegion<'_, 'a, F, T> {
        FsmTimersRegion { shared: self }
    }

    fn with<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        let mut timers = self.timers.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        f(&mut **timers)
    }
}

/// A region's access to the shared timers.
pub struct FsmTimersRegion<'s, 'a, F, T>
    where F: FsmBackend, T: FsmTimers<F>
{
    shared: &'s FsmTimersShared<'a, F, T>
}

impl<'s, 'a, F, T> FsmTimers<F> for FsmTimersRegion<'s, 'a, F, T>
    where F: FsmBackend, T: FsmTimers<F>
{
    fn create(&mut self, id: <F as FsmBackend>::Timers, settings: &TimerSettings) -> crate::FsmResult<()> {
        self.shared.with(|timers| timers.create(id, settings))
    }

    fn cancel(&mut self, id: <F as FsmBackend>::Timers) -> crate::FsmResult<()> {
        self.shared.with(|timers| timers.cancel(id))
    }

    fn get_triggered_timer(&mut self) -> Option<<F as FsmBackend>::Timers> {
        None
    }
}

/// Appends the events that a concurrent region enqueued to the FSM's queue.
pub fn merge_region_queue<F, Q, I>(mut region_queue: FsmEventQueueVec<F>, queue: &mut Q, inspect: &I)
    where F: FsmBackend, Q: FsmEventQueue<F>, I: Inspect
{
    while let Some(ev) = region_queue.dequeue() {
        if let Err(ref e) = queue.enqueue(ev) {
            inspect.on_error("Failed to enqueue an event of a concurrent region", e);
        }
    }
}
