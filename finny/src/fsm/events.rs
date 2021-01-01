use std::ops::{Deref, DerefMut};

use crate::FsmBackend;


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
