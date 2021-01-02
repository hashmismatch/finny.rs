use lib::*;

use crate::FsmBackend;

use super::{FsmStateBuilder};

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

	/// Adds some information about a state.
	pub fn state<TState>(&mut self) -> FsmStateBuilder<TFsm, TContext, TState> {
		FsmStateBuilder {
			_state: PhantomData::default(),
			_fsm: PhantomData::default(),
			_context: PhantomData::default()
		}
	}

	/// Builds the final machine. Has to be returned from the definition function.
    pub fn build(self) -> BuiltFsm {
        BuiltFsm
    }
}