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

impl AsRef<str> for Events {
    fn as_ref(&self) -> &'static str {
        todo!()
    }
}


impl FsmBackend for TestFsm {
    type Context = ();
    type States = States;
    type Events = Events;

    fn dispatch_event<Q, I>(_frontend: &mut crate::FsmFrontend<Self, Q, I>, _event: &crate::FsmEvent<Self::Events>) -> crate::FsmResult<()>
        where Q: crate::FsmEventQueue<Self>, I: crate::Inspect
    {
        todo!()
    }
}