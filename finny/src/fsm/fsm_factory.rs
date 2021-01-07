use crate::{FsmBackend, FsmBackendImpl, FsmEventQueue, FsmFrontend, FsmResult, Inspect, InspectNull};

#[cfg(feature="std")]
use crate::FsmEventQueueVec;

/// Builds a frontend for running your FSM.
pub trait FsmFactory {
    type Fsm: FsmBackend;

    /// Build a new frontend for the FSM with all the environmental services provided by the caller.
    fn new_with<Q, I>(context: <Self::Fsm as FsmBackend>::Context, queue: Q, inspect: I) -> FsmResult<FsmFrontend<Self::Fsm, Q, I>>
        where Q: FsmEventQueue<Self::Fsm>, I: Inspect<Self::Fsm>
    {
        let frontend = FsmFrontend {
            queue,
            inspect,
            backend: FsmBackendImpl::new(context)?
        };
        
        Ok(frontend)
    }

    /// Build a new frontend for the FSM with a `FsmEventQueueVec` queue and no logging.
    #[cfg(feature="std")]
    fn new(context: <Self::Fsm as FsmBackend>::Context) -> FsmResult<FsmFrontend<Self::Fsm, FsmEventQueueVec<Self::Fsm>, InspectNull<Self::Fsm>>> {
        let frontend = FsmFrontend {
            queue: FsmEventQueueVec::new(),
            backend: FsmBackendImpl::new(context)?,
            inspect: InspectNull::new()
        };

        Ok(frontend)
    }
}