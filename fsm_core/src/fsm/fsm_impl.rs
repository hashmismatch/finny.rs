use std::marker::PhantomData;

use crate::{FsmBackend, FsmCurrentState, FsmEventQueueVec, FsmResult, FsmStates};

use super::FsmStateFactory;


pub struct FsmBackendImpl<F: FsmBackend> {
    context: <F as FsmBackend>::Context,
    states: <F as FsmBackend>::States,
    current_state: FsmCurrentState<<<F as FsmBackend>::States as FsmStates>::StateKind>
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
}

pub struct FsmFrontend<Queue, F: FsmBackend> {
    queue: Queue,
    backend: FsmBackendImpl<F>
}

impl<Queue, F: FsmBackend> FsmFrontend<Queue, F> {
    pub fn new_all(queue: Queue, context: <F as FsmBackend>::Context) -> FsmResult<Self> {
        //let queue = super::FsmEventQueueVec::new();

        let frontend = Self {
            queue,
            backend: FsmBackendImpl::new(context)?
        };

        Ok(frontend)
    }
}

impl<F: FsmBackend> FsmFrontend<FsmEventQueueVec<<F as FsmBackend>::Events>, F> {
    pub fn new(context: <F as FsmBackend>::Context) -> FsmResult<Self> {
        Self::new_all(super::FsmEventQueueVec::new(), context)
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