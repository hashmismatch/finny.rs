use crate::FsmResult;


pub trait FsmStates: FsmStateFactory {
    type StateKind: Clone + Copy + std::fmt::Debug + PartialEq;
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum FsmCurrentState<S: Clone + Copy> {
    Stopped,
    State(S)
}

pub trait FsmStateFactory where Self: Sized {
    fn new_state<C>(context: &C) -> FsmResult<Self>;
}

impl<TState> FsmStateFactory for TState where TState: Default {
    fn new_state<C>(_context: &C) -> FsmResult<Self> {
        Ok(Default::default())
    }
}
