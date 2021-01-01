//! The public Finite State Machine traits. The derive macros will implement these for your particular
//! state machines.

use std::{collections::VecDeque, marker::PhantomData, ops::{Deref, DerefMut}};

mod fsm_impl;
mod fsm_factory;
pub use self::fsm_factory::*;
pub use self::fsm_impl::*;

pub type FsmResult<T> = std::result::Result<T, FsmError>;

#[derive(Debug, PartialEq)]
pub enum FsmError {
    NoTransition
}

pub trait FsmStates: FsmStateFactory {
    type StateKind: Clone + Copy + std::fmt::Debug + PartialEq;
}

pub enum FsmEvent<E> {
    Start,
    Stop,
    Event(E)
}

impl<E> From<E> for FsmEvent<E> {
    fn from(event: E) -> Self {
        FsmEvent::Event(event)
    }
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

    fn dispatch_event<Q>(backend: &mut FsmBackendImpl<Self>, event: &FsmEvent<Self::Events>, queue: &mut Q) -> FsmResult<()>
        where Q: FsmEventQueue<Self::Events>;
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum FsmCurrentState<S: Clone + Copy> {
    Stopped,
    State(S)
}

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

impl<'a, TFsm: FsmBackend, Q> Deref for EventContext<'a, TFsm, Q> {
    type Target = <TFsm as FsmBackend>::Context;

    fn deref(&self) -> &Self::Target {
        self.context
    }
}

impl<'a, TFsm: FsmBackend, Q> DerefMut for EventContext<'a, TFsm, Q> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.context
    }
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