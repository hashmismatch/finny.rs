use std::{marker::PhantomData, ops::Deref};

use crate::{EventContext, FsmBackend, FsmCurrentState, FsmEvent, FsmEventQueue, FsmEventQueueVec, FsmResult, FsmStates};

use super::FsmStateFactory;


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
            current_state,
            //fsm: F::new()
        };

        Ok(backend)
    }
    
    pub fn get_context(&self) -> &<F as FsmBackend>::Context {
        &self.context
    }

    pub fn get_current_state(&self) -> FsmCurrentState<<<F as FsmBackend>::States as FsmStates>::StateKind> {
        self.current_state
    }
}

pub struct FsmFrontend<Queue, F: FsmBackend> {
    queue: Queue,
    backend: FsmBackendImpl<F>
}

impl<Queue: FsmEventQueue<<F as FsmBackend>::Events>, F: FsmBackend> FsmFrontend<Queue, F> {
    pub fn new_all(queue: Queue, context: <F as FsmBackend>::Context) -> FsmResult<Self> {
        //let queue = super::FsmEventQueueVec::new();

        let frontend = Self {
            queue,
            backend: FsmBackendImpl::new(context)?
        };

        Ok(frontend)
    }

    pub fn start(&mut self) -> FsmResult<()> {
        self.dispatch(&FsmEvent::Start)
    }

    pub fn dispatch(&mut self, event: &FsmEvent<<F as FsmBackend>::Events>) -> FsmResult<()> {
        F::dispatch_event(&mut self.backend, event, &mut self.queue)
    }
}

impl<F: FsmBackend> FsmFrontend<FsmEventQueueVec<<F as FsmBackend>::Events>, F> {
    pub fn new(context: <F as FsmBackend>::Context) -> FsmResult<Self> {
        Self::new_all(super::FsmEventQueueVec::new(), context)
    }
}


impl<Queue, F: FsmBackend> Deref for FsmFrontend<Queue, F> {
    type Target = FsmBackendImpl<F>;

    fn deref(&self) -> &Self::Target {
        &self.backend
    }
}


/*

pub struct FsmBackendImpl<C, S, CS, E, Q> {
    pub context: C,
    pub states: S,
    pub queue: Q,
    pub current_state: FsmCurrentState<CS>,
    _events: PhantomData<E>
}

impl<C, S, CS, E, Q> FsmBackendImpl<C, S, CS, E, Q> {
    
}

*/