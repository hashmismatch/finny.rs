use crate::{DispatchContext, Inspect, lib::*};
use crate::{FsmBackend, FsmEvent, FsmEventQueue, FsmResult, FsmStates};

use super::FsmStateFactory;

/// The struct that holds the core context and state of the given Finny FSM. Doesn't include
/// environmental traits that can be changed at runtime.
pub struct FsmBackendImpl<F: FsmBackend> {
    pub context: <F as FsmBackend>::Context,
    pub states: <F as FsmBackend>::States,
    pub current_states: <<F as FsmBackend>::States as FsmStates<F>>::CurrentState
}

impl<F: FsmBackend> FsmBackendImpl<F> {
    pub fn new(context: <F as FsmBackend>::Context) -> FsmResult<Self> {

        let states = <<F as FsmBackend>::States>::new_state(&context)?;
        let current_states = <<<F as FsmBackend>::States as FsmStates<F>>::CurrentState>::default();

        let backend = FsmBackendImpl::<F> {
            context,
            states,
            current_states
        };

        Ok(backend)
    }
    
    pub fn get_context(&self) -> &<F as FsmBackend>::Context {
        &self.context
    }

    pub fn get_current_states(&self) -> <<F as FsmBackend>::States as FsmStates<F>>::CurrentState {
        self.current_states
    }

    pub fn get_state<S>(&self) -> &S
        where <F as FsmBackend>::States : AsRef<S>
    {
        self.states.as_ref()
    }
}

impl<F: FsmBackend> Deref for FsmBackendImpl<F> {
    type Target = <F as FsmBackend>::Context;

    fn deref(&self) -> &Self::Target {
        &self.context
    }
}

/// The frontend of a state machine which also includes environmental services like queues
/// and inspection. The usual way to use the FSM.
pub struct FsmFrontend<F, Q, I> 
    where F: FsmBackend, Q: FsmEventQueue<F>, I: Inspect
{
    pub backend: FsmBackendImpl<F>,
    pub queue: Q,
    pub inspect: I    
}

impl<F, Q, I> FsmFrontend<F, Q, I>
    where F: FsmBackend, Q: FsmEventQueue<F>, I: Inspect
{
    /// Start the FSM, initiates the transition to the initial state.
    pub fn start(&mut self) -> FsmResult<()> {
        Self::dispatch_single_event(self, FsmEvent::Start)
    }

    /// Dispatch this event and run it to completition.
    pub fn dispatch<E>(&mut self, event: E) -> FsmResult<()>
        where E: Into<<F as FsmBackend>::Events>
    {
        let ev = event.into();
        let ev = FsmEvent::Event(ev);
        Self::dispatch_single_event(self, ev)?;

        while let Some(ev) = self.queue.dequeue() {
            let ev: <F as FsmBackend>::Events = ev.into();
            Self::dispatch(self, ev)?;
        }

        Ok(())
    }

    /// Dispatch only this event, do not run it to completition.
    pub fn dispatch_single_event(&mut self, event: FsmEvent<<F as FsmBackend>::Events>) -> FsmResult<()> {
        let dispatch_ctx = DispatchContext {
            backend: &mut self.backend,
            inspect: &mut self.inspect,
            queue: &mut self.queue
        };

        F::dispatch_event(dispatch_ctx, event)
    }
}

impl<F, Q, I> Deref for FsmFrontend<F, Q, I>
    where F: FsmBackend, Q: FsmEventQueue<F>, I: Inspect
{
    type Target = FsmBackendImpl<F>;

    fn deref(&self) -> &Self::Target {
        &self.backend
    }
}