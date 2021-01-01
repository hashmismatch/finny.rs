use crate::{FsmBackend, FsmBackendImpl, FsmEventQueueVec, FsmFrontend, FsmResult};


/// Builds a frontend for running your FSM.
pub trait FsmFactory {
    type Fsm: FsmBackend;

    fn new(context: <Self::Fsm as FsmBackend>::Context) -> FsmResult<FsmFrontend<FsmEventQueueVec<<Self::Fsm as FsmBackend>::Events>, Self::Fsm>> {
        let frontend = FsmFrontend {
            queue: FsmEventQueueVec::new(),
            backend: FsmBackendImpl::new(context)?
        };

        Ok(frontend)
    }
}