use lib::*;

use crate::{FsmBackend, FsmCurrentState, FsmEvent, FsmEventQueue, FsmResult, FsmStates};

use super::FsmStateFactory;

/// The struct that holds the core context and state of the given Finny FSM. Doesn't include
/// environmental traits that can be changed at runtime.
pub struct FsmBackendImpl<F: FsmBackend> {
    pub context: <F as FsmBackend>::Context,
    pub states: <F as FsmBackend>::States,
    pub current_state: FsmCurrentState<<<F as FsmBackend>::States as FsmStates>::StateKind>
}

impl<F: FsmBackend> FsmBackendImpl<F> {
    pub fn new(context: <F as FsmBackend>::Context) -> FsmResult<Self> {

        let states = <<F as FsmBackend>::States>::new_state(&context)?;
        let current_state = FsmCurrentState::Stopped;

        let backend = FsmBackendImpl::<F> {
            context,
            states,
            current_state
        };

        Ok(backend)
    }
    
    pub fn get_context(&self) -> &<F as FsmBackend>::Context {
        &self.context
    }

    pub fn get_current_state(&self) -> FsmCurrentState<<<F as FsmBackend>::States as FsmStates>::StateKind> {
        self.current_state
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
pub struct FsmFrontend<Queue, F: FsmBackend> {
    pub (crate) queue: Queue,
    pub (crate) backend: FsmBackendImpl<F>
}

impl<Queue: FsmEventQueue<<F as FsmBackend>::Events>, F: FsmBackend> FsmFrontend<Queue, F> {
    /// Start the FSM, initiates the transition to the initial state.
    pub fn start(&mut self) -> FsmResult<()> {
        Self::dispatch_event(self, &FsmEvent::Start)
    }

    /// Dispatch this event and run it to completition.
    pub fn dispatch<E>(&mut self, event: E) -> FsmResult<()>
        where E: Into<<F as FsmBackend>::Events>
    {
        let ev = event.into();
        let ev = FsmEvent::Event(ev);
        Self::dispatch_event(self, &ev)
    }

    fn dispatch_event(&mut self, event: &FsmEvent<<F as FsmBackend>::Events>) -> FsmResult<()> {
        F::dispatch_event(&mut self.backend, event, &mut self.queue)
    }
}

impl<Queue, F: FsmBackend> Deref for FsmFrontend<Queue, F> {
    type Target = FsmBackendImpl<F>;

    fn deref(&self) -> &Self::Target {
        &self.backend
    }
}