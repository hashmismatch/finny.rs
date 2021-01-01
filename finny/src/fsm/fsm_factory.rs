use lib::*;
use crate::{FsmBackend, FsmBackendImpl, FsmFrontend, FsmResult};

#[cfg(feature="std")]
use crate::FsmEventQueueVec;

/// Builds a frontend for running your FSM.
pub trait FsmFactory {
    type Fsm: FsmBackend;

    #[cfg(feature="std")]
    fn new(context: <Self::Fsm as FsmBackend>::Context) -> FsmResult<FsmFrontend<FsmEventQueueVec<<Self::Fsm as FsmBackend>::Events>, Self::Fsm>> {
        let frontend = FsmFrontend {
            queue: FsmEventQueueVec::new(),
            backend: FsmBackendImpl::new(context)?
        };

        Ok(frontend)
    }
}