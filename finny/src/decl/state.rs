use lib::*;

use crate::{EventContext, FsmBackend, FsmEventQueue};

use super::fsm::FsmBuilder;
use super::event::FsmEventBuilderState;


pub struct FsmStateBuilder<'a, TFsm, TContext, TState> {
	pub (crate) _state: PhantomData<TState>,
	pub (crate) _fsm: &'a FsmBuilder<TFsm, TContext>
}

impl<'a, TFsm, TContext, TState> FsmStateBuilder<'a, TFsm, TContext, TState>
	where TFsm: FsmBackend
{
	/// Execute this action when entering the state.
	pub fn on_entry<TAction: Fn(&mut TState, &mut EventContext<'a, TFsm, dyn FsmEventQueue<<TFsm as FsmBackend>::Events>>)>(&self, _action: TAction) -> &Self {
		self
	}

	/// Execute this action when exiting the state.
	pub fn on_exit<TAction: Fn(&mut TState, &mut EventContext<'a, TFsm, dyn FsmEventQueue<<TFsm as FsmBackend>::Events>>)>(&self, _action: TAction) -> &Self {
		self
	}

	/// What happens if we receive this event and we are in this state right now?
	pub fn on_event<TEvent>(self) -> FsmEventBuilderState<'a, TFsm, TContext, TEvent, TState> {
		FsmEventBuilderState {
			_state_builder: self,
			_event: PhantomData::default()
		}
	}
}
