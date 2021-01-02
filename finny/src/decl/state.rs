use lib::*;

use crate::{EventContext, FsmBackend, FsmEventQueue};

use super::event::FsmEventBuilderState;

pub struct FsmStateBuilder<TFsm, TContext, TState> {
	pub (crate) _state: PhantomData<TState>,
	pub (crate) _fsm: PhantomData<TFsm>,
	pub (crate) _context: PhantomData<TContext>
}

impl<TFsm, TContext, TState> FsmStateBuilder<TFsm, TContext, TState>
	where TFsm: FsmBackend
{
	/// Execute this action when entering the state.
	pub fn on_entry<'a, TAction: Fn(&mut TState, &mut EventContext<'a, TFsm, dyn FsmEventQueue<<TFsm as FsmBackend>::Events>>)>(&self, _action: TAction) -> &Self {
		self
	}

	/// Execute this action when exiting the state.
	pub fn on_exit<'a, TAction: Fn(&mut TState, &mut EventContext<'a, TFsm, dyn FsmEventQueue<<TFsm as FsmBackend>::Events>>)>(&self, _action: TAction) -> &Self {
		self
	}

	/// What happens if we receive this event and we are in this state right now?
	pub fn on_event<TEvent>(&self) -> FsmEventBuilderState<TFsm, TContext, TEvent, TState> {
		FsmEventBuilderState {
			_state_builder: self,
			_event: PhantomData::default()
		}
	}
}
