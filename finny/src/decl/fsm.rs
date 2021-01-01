use lib::*;

use crate::FsmBackend;

use super::{event::FsmEventBuilder, state::FsmStateBuilder};

#[derive(Default)]
pub struct FsmBuilder<TFsm, TContext> {
    pub _fsm: PhantomData<TFsm>,
    pub _context: PhantomData<TContext>
}

pub struct BuiltFsm;

impl<TFsm, TContext> FsmBuilder<TFsm, TContext>
	where TFsm: FsmBackend
{
	/// Sets the initial state of the state machine. Required!
	pub fn initial_state<TSTate>(&mut self) {
		
	}

	/// Adds some information about a state
	pub fn state<TState>(&mut self) -> FsmStateBuilder<TFsm, TContext, TState> {
		FsmStateBuilder {
			_state: PhantomData::default(),
			_fsm: self
		}
	}

	pub fn on_event<'a, TEvent>(&'a mut self) -> FsmEventBuilder<'a, TFsm, TContext, TEvent> {
		FsmEventBuilder {
            _event: PhantomData::default(),
            _fsm: self
        }
	}

	/// Builds the final machine. Has to be returned from the definition function.
    pub fn build(self) -> BuiltFsm {
        BuiltFsm
    }
}