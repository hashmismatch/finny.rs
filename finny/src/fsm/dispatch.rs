use crate::{FsmTimers, FsmTimersSub, lib::*};
use crate::{EventContext, FsmBackend, FsmBackendImpl, FsmCurrentState, FsmDispatch, FsmEvent, FsmEventQueue, FsmEventQueueSub, FsmRegionId, FsmResult, Inspect};

pub struct DispatchContext<'a, 'b, 'c, F, Q, I, T>
    where F: FsmBackend,
    Q: FsmEventQueue<F>,
    I: Inspect,
    T: FsmTimers<F>
{
    pub queue: &'a mut Q,
    pub inspect: &'b mut I,
    pub backend: &'c mut FsmBackendImpl<F>,
    pub timers: &'a mut T
}

impl<'a, 'b, 'c, F, Q, I, T> DispatchContext<'a, 'b, 'c, F, Q, I, T>
where F: FsmBackend,
    Q: FsmEventQueue<F>,
    I: Inspect,
    T: FsmTimers<F>
{

    pub fn to_event_context(&'a mut self, region: FsmRegionId) -> EventContext<'a, F, Q>
    {
        EventContext {
            context: &mut self.backend.context,
            queue: self.queue,
            region
        }
    } 
}

/// Used to funnel the event down to the sub-machine.
pub fn dispatch_to_submachine<'a, 'b, 'c, TFsm, TSubMachine, Q, I, T>(ctx: &mut DispatchContext<'a, 'b, 'c, TFsm, Q, I, T>,
        ev: FsmEvent<<TSubMachine as FsmBackend>::Events, <TSubMachine as FsmBackend>::Timers>, inspect_event_ctx: &I)
    -> FsmResult<()>
    where
        TFsm: FsmBackend,
        <TFsm as FsmBackend>::States: AsMut<TSubMachine>,
        <TFsm as FsmBackend>::Events: From<<TSubMachine as FsmBackend>::Events>,
        <TFsm as FsmBackend>::Timers: From<<TSubMachine as FsmBackend>::Timers>,
        TSubMachine: FsmDispatch + DerefMut<Target = FsmBackendImpl<TSubMachine>>,
        Q: FsmEventQueue<TFsm>,
        I: Inspect,
        T: FsmTimers<TFsm>,
{
    let sub_fsm: &mut TSubMachine = ctx.backend.states.as_mut();

    let mut queue_adapter = FsmEventQueueSub {
        parent: ctx.queue,
        _parent_fsm: core::marker::PhantomData::<TFsm>::default(),
        _sub_fsm: core::marker::PhantomData::<TSubMachine>::default()
    };

    let mut timers_adapter = FsmTimersSub {
        parent: ctx.timers,
        _parent_fsm: core::marker::PhantomData::<TFsm>::default(),
        _sub_fsm: core::marker::PhantomData::<TSubMachine>::default()
    };

    let mut inspect = inspect_event_ctx.for_sub_machine::<TSubMachine>();

    let sub_dispatch_ctx = DispatchContext {
        backend: sub_fsm,
        inspect: &mut inspect,
        queue: &mut queue_adapter,
        timers: &mut timers_adapter
    };
    
    <TSubMachine>::dispatch_event(sub_dispatch_ctx, ev)
}
/// Starts the sub-machine after its state was entered, unless it is already running.
pub fn start_submachine<'a, 'b, 'c, TFsm, TSubMachine, Q, I, T>(ctx: &mut DispatchContext<'a, 'b, 'c, TFsm, Q, I, T>, inspect_event_ctx: &I)
    -> FsmResult<()>
    where
        TFsm: FsmBackend,
        <TFsm as FsmBackend>::States: AsMut<TSubMachine>,
        <TFsm as FsmBackend>::Events: From<<TSubMachine as FsmBackend>::Events>,
        <TFsm as FsmBackend>::Timers: From<<TSubMachine as FsmBackend>::Timers>,
        TSubMachine: FsmDispatch + DerefMut<Target = FsmBackendImpl<TSubMachine>>,
        Q: FsmEventQueue<TFsm>,
        I: Inspect,
        T: FsmTimers<TFsm>,
{
    let sub_fsm: &mut TSubMachine = ctx.backend.states.as_mut();
    if !FsmCurrentState::all_stopped(sub_fsm.get_current_states().as_ref()) {
        return Ok(());
    }

    dispatch_to_submachine::<TFsm, TSubMachine, Q, I, T>(ctx, FsmEvent::Start, inspect_event_ctx)
}

/// Stops the sub-machine before its state is exited: exits its active states, including their
/// timers and the nested sub-machines.
pub fn stop_submachine<'a, 'b, 'c, TFsm, TSubMachine, Q, I, T>(ctx: &mut DispatchContext<'a, 'b, 'c, TFsm, Q, I, T>, inspect_event_ctx: &I)
    -> FsmResult<()>
    where
        TFsm: FsmBackend,
        <TFsm as FsmBackend>::States: AsMut<TSubMachine>,
        <TFsm as FsmBackend>::Events: From<<TSubMachine as FsmBackend>::Events>,
        <TFsm as FsmBackend>::Timers: From<<TSubMachine as FsmBackend>::Timers>,
        TSubMachine: FsmDispatch + DerefMut<Target = FsmBackendImpl<TSubMachine>>,
        Q: FsmEventQueue<TFsm>,
        I: Inspect,
        T: FsmTimers<TFsm>,
{
    let sub_fsm: &mut TSubMachine = ctx.backend.states.as_mut();
    if FsmCurrentState::all_stopped(sub_fsm.get_current_states().as_ref()) {
        return Ok(());
    }

    dispatch_to_submachine::<TFsm, TSubMachine, Q, I, T>(ctx, FsmEvent::Stop, inspect_event_ctx)
}
