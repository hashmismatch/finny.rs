use crate::{FsmBackend, FsmBackendImpl, FsmEventQueue, FsmFrontend, FsmResult, FsmTimers, FsmTimersNull, Inspect};

#[cfg(feature="std")]
use crate::{FsmEventQueueVec, timers::std::TimersStd};

/// Builds a frontend for running your FSM.
pub trait FsmFactory {
    type Fsm: FsmBackend;

    /// For submachines, for use with codegen.
    fn new_submachine_backend(backend: FsmBackendImpl<Self::Fsm>) -> FsmResult<Self> where Self: Sized;

    /// Build a new frontend for the FSM with all the environmental services provided by the caller.
    fn new_with<Q, I, T>(context: <Self::Fsm as FsmBackend>::Context, queue: Q, inspect: I, timers: T) -> FsmResult<FsmFrontend<Self::Fsm, Q, I, T>>
        where Q: FsmEventQueue<Self::Fsm>, I: Inspect, T: FsmTimers<Self::Fsm>
    {
        let frontend = FsmFrontend {
            queue,
            inspect,
            backend: FsmBackendImpl::new(context)?,
            timers
        };
        
        Ok(frontend)
    }

    /// Build a new frontend for the FSM with a `FsmEventQueueVec` queue, `TimersStd` for timers and no logging.
    #[cfg(feature="std")]
    fn new(context: <Self::Fsm as FsmBackend>::Context) -> FsmResult<FsmFrontend<Self::Fsm, FsmEventQueueVec<Self::Fsm>, crate::inspect::null::InspectNull, TimersStd<Self::Fsm>>> {
        use crate::inspect::null::InspectNull;

        let frontend = FsmFrontend {
            queue: FsmEventQueueVec::new(),
            backend: FsmBackendImpl::new(context)?,
            inspect: InspectNull::new(),
            timers: TimersStd::new(),
        };

        Ok(frontend)
    }

    /// Build a frontend for a deserialized machine, with all the environmental services provided
    /// by the caller. The machine continues from its restored states, it doesn't have to be
    /// started. Its running timers are re-created with their full timeouts.
    #[cfg(feature = "serde")]
    fn restore_with<Q, I, T>(backend: FsmBackendImpl<Self::Fsm>, queue: Q, inspect: I, timers: T) -> FsmResult<FsmFrontend<Self::Fsm, Q, I, T>>
        where Q: FsmEventQueue<Self::Fsm>, I: Inspect, T: FsmTimers<Self::Fsm>
    {
        let mut frontend = FsmFrontend {
            queue,
            inspect,
            backend,
            timers
        };
        <Self::Fsm as FsmBackend>::restore_timers(&mut frontend.backend, &frontend.inspect, &mut frontend.timers);

        Ok(frontend)
    }

    /// Build a frontend for a deserialized machine with a `FsmEventQueueVec` queue, `TimersStd`
    /// for timers and no logging. See [`restore_with`](Self::restore_with).
    #[cfg(all(feature = "std", feature = "serde"))]
    fn restore(backend: FsmBackendImpl<Self::Fsm>) -> FsmResult<FsmFrontend<Self::Fsm, FsmEventQueueVec<Self::Fsm>, crate::inspect::null::InspectNull, TimersStd<Self::Fsm>>> {
        Self::restore_with(backend, FsmEventQueueVec::new(), crate::inspect::null::InspectNull::new(), TimersStd::new())
    }
}
