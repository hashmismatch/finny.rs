//! The public Finite State Machine traits. The derive macros will implement these for your particular
//! state machines.

use std::{collections::VecDeque, marker::PhantomData};

mod fsm_impl;
pub use self::fsm_impl::*;

pub type FsmResult<T> = std::result::Result<T, FsmError>;

#[derive(Debug, PartialEq)]
pub enum FsmError {
    NoTransition
}

pub trait FsmStates: FsmStateFactory {
    type StateKind;
}

/// Finite State Machine backend. Handles the dispatching, the types are
/// defined by the code generator.
pub trait FsmBackend where Self: Sized {
    /// The machine's context that is shared between its constructors and actions.
    type Context;
    /// The type that holds the states of the machine.
    type States: FsmStates;
    /// A tagged union type with all the supported events.
    type Events;

    fn dispatch_event<Q>(&mut self, event: &Self::Events, event_context: &mut EventContext<Self, Q>) -> FsmResult<()>;
}

/*
/// Finite state machine's frontend. Adds the queueing and inspection
/// types which can be exchanged by the end-user.
pub trait FsmFrontend<Queue>
    where Queue: FsmEventQueue<<Self::Backend as FsmBackend>::Events>
{
    type Backend: FsmBackend;
    

}
*/

/*
pub trait FsmCoreDispatch : FsmBackend {
    fn dispatch_event(&mut self, event: &Self::Events) -> FsmResult<()>;
}
*/

pub enum FsmCurrentState<S> {
    Stopped,
    State(S)
}

/*
pub struct FsmCoreImpl<C, S, CS, E, Q> {
    pub context: C,
    pub states: S,
    pub queue: Q,
    pub current_state: FsmCurrentState<CS>,
    _events: PhantomData<E>
}

impl<C, S, CS, E, Q> FsmCoreImpl<C, S, CS, E, Q>
    //where Self: FsmCore<Context = C>
{
    pub fn new_all(context: C, states: S, queue: Q) -> FsmResult<Self> {
        let f = FsmCoreImpl {
            context,
            states,
            queue,
            current_state: FsmCurrentState::<CS>::Stopped,
            _events: PhantomData::default()
        };
        Ok(f)
    }
}
*/

/*
pub struct Fsm<TFsm, TQueue> {
    fsm: TFsm,
    queue: TQueue
}

impl<TFsmCore, TQueue> Fsm<TFsmCore, TQueue>
    where TFsmCore: FsmCore, TQueue: FsmEventQueue<TFsmCore>
{
    pub fn new(context: TFsmCore::Context, queue: TQueue) -> Result<Self> {
        let fsm_core = TFsmCore::new(context)?;
        let fsm = Fsm {
            fsm: fsm_core,
            queue
        };
        Ok(fsm)
    }

    pub fn process(&mut self, event: TFsmCore::Events) -> Result<()> {
        todo!()
    }
}
*/

pub trait FsmEventQueue<T> {
    fn enqueue(&mut self, event: T) -> FsmResult<()>;
    fn dequeue(&mut self) -> Option<T>;
}

pub struct FsmEventQueueVec<T> {
    queue: VecDeque<T>
}

impl<T> FsmEventQueue<T> for FsmEventQueueVec<T> {
    fn enqueue(&mut self, event: T) -> FsmResult<()> {
        self.queue.push_back(event);
        Ok(())
    }

    fn dequeue(&mut self) -> Option<T> {
        self.queue.pop_front()
    }
}

impl<T> FsmEventQueueVec<T> {
    pub fn new() -> Self {
        FsmEventQueueVec {
            queue: VecDeque::new()
        }
    }
}

pub trait FsmStateFactory where Self: Sized {
    fn new_state<C>(context: &C) -> FsmResult<Self>;
}

impl<TState> FsmStateFactory for TState where TState: Default {
    fn new_state<C>(_context: &C) -> FsmResult<Self> {
        Ok(Default::default())
    }
}

pub struct EventContext<'a, TFsm: FsmBackend, Q> {
    pub context: &'a mut TFsm::Context,
    pub queue: &'a mut Q
}

pub trait FsmState<F: FsmBackend> {
    fn on_entry<'a, Q: FsmEventQueue<<F as FsmBackend>::Events>>(&mut self, context: &mut EventContext<'a, F, Q>);
    fn on_exit<'a, Q: FsmEventQueue<<F as FsmBackend>::Events>>(&mut self, context: &mut EventContext<'a, F, Q>);
}

pub trait FsmTransitionGuard<F: FsmBackend, E> {
    fn guard<'a, Q: FsmEventQueue<<F as FsmBackend>::Events>>(event: &E, context: &EventContext<'a, F, Q>) -> bool;
}

pub trait FsmTransitionAction<F: FsmBackend, E, TStateFrom, TStateTo> {
    fn action<'a, Q: FsmEventQueue<<F as FsmBackend>::Events>>(event: &E, context: &mut EventContext<'a, F, Q>, from: &mut TStateFrom, to: &mut TStateTo);
}