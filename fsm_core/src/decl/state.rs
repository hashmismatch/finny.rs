
use std::marker::PhantomData;

use crate::fsm::StateContext;

use super::fsm::FsmBuilder;


pub struct FsmStateBuilder<'a, TFsm, TContext, TState> {
	pub (crate) _state: PhantomData<TState>,
	pub (crate) _fsm: &'a FsmBuilder<TFsm, TContext>
}

impl<'a, TFsm, TContext, TState> FsmStateBuilder<'a, TFsm, TContext, TState> {
	/// Execute this action when entering the state.
	pub fn on_entry<TAction: Fn(&mut TState, &mut StateContext<'a, TFsm>)>(&self, _action: TAction) -> &Self {
		self
	}

	/// Execute this action when exiting the state.
	pub fn on_exit<TAction: Fn(&mut TState, &mut StateContext<'a, TFsm>)>(&self, _action: TAction) -> &Self {
		self
	}
}
