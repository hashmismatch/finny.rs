//! A minimal, internal FSM for unit tests, manually written.

use crate::{FsmBackend, FsmCurrentState, FsmStates};
use derive_more::From;

#[derive(Default)]
pub struct StateA;
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct EventA { pub n: usize }

pub struct TestFsm;

#[derive(Default)]
pub struct States {
    state_a: StateA
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum StateKind {
    StateA
}

impl FsmStates for States {
    type StateKind = StateKind;
    type CurrentState = [FsmCurrentState<StateKind>; 1];
}

#[derive(Debug, Copy, Clone, PartialEq, From)]
pub enum Events {
    EventA(EventA)
}


impl FsmBackend for TestFsm {
    type Context = ();
    type States = States;
    type Events = Events;

    fn dispatch_event<Q>(_backend: &mut crate::FsmBackendImpl<Self>, _event: &crate::FsmEvent<Self::Events>, _queue: &mut Q) -> crate::FsmResult<()>
        where Q: crate::FsmEventQueue<Self> {
        todo!()
    }
}