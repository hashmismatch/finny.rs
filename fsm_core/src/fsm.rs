//! The public Finite State Machine traits. The derive macros will implement these for your particular
//! state machines.

use std::{collections::VecDeque, marker::PhantomData};


pub type FsmResult<T> = std::result::Result<T, FsmError>;

#[derive(Debug)]
pub enum FsmError {
    NoTransition
}

pub trait FsmStates {
    type StateKind;
}

/// Finite State Machine core.
pub trait FsmCore where Self: Sized {
    /// The machine's context that is shared between its constructors and actions.
    type Context;
    /// The type that holds the states of the machine.
    type States: FsmStates;
    /// A tagged union type with all the supported events.
    type Events;
}

pub trait FsmCoreDispatch : FsmCore {
    fn dispatch(&mut self, event: &Self::Events) -> FsmResult<()>;
}

pub struct FsmCoreImpl<C, S, E, Q> {
    pub context: C,
    pub states: S,
    pub queue: Q,
    _events: PhantomData<E>
}

impl<C, S, E, Q> FsmCoreImpl<C, S, E, Q> {
    pub fn new_all(context: C, states: S, queue: Q) -> FsmResult<Self> {
        let f = FsmCoreImpl {
            context,
            states,
            queue,
            _events: PhantomData::default()
        };
        Ok(f)
    }
}


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

/*
pub struct StateContext<'a, TFsm: FsmCore> {
    pub context: &'a mut TFsm::Context
}
*/

pub struct EventContext<'a, TFsm: FsmCore> {
    pub context: &'a mut TFsm::Context
}

pub trait FsmState<F: FsmCore> {
    fn on_entry<'a>(&mut self, context: &mut EventContext<'a, F>);
    fn on_exit<'a>(&mut self, context: &mut EventContext<'a, F>);
}