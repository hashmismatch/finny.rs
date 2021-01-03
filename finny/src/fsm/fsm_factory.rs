use crate::lib::*;
use crate::{FsmBackend, FsmBackendImpl, FsmEventQueue, FsmFrontend, FsmResult};

#[cfg(feature="std")]
use crate::FsmEventQueueVec;

/// Builds a frontend for running your FSM.
pub trait FsmFactory {
    type Fsm: FsmBackend;

    /// Build a new frontend for the FSM with all the environmental services provided by the caller.
    fn new_with<Q>(context: <Self::Fsm as FsmBackend>::Context, queue: Q) -> FsmResult<FsmFrontend<Q, Self::Fsm>>
        where Q: FsmEventQueue<Self::Fsm>
    {
        let frontend = FsmFrontend {
            queue,
            backend: FsmBackendImpl::new(context)?
        };
        
        Ok(frontend)
    }

    /// Build a new frontend for the FSM with a `FsmEventQueueVec` queue.
    #[cfg(feature="std")]
    fn new(context: <Self::Fsm as FsmBackend>::Context) -> FsmResult<FsmFrontend<FsmEventQueueVec<Self::Fsm>, Self::Fsm>> {
        let frontend = FsmFrontend {
            queue: FsmEventQueueVec::new(),
            backend: FsmBackendImpl::new(context)?
        };

        Ok(frontend)
    }
}