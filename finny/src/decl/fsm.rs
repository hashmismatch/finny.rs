use lib::*;

use crate::FsmBackend;

use super::{FsmStateBuilder};

/// The main builder-API for defining your Finny state machine.
#[derive(Default)]
pub struct FsmBuilder<TFsm, TContext> {
    pub _fsm: PhantomData<TFsm>,
    pub _context: PhantomData<TContext>
}

/// The consumed struct of the FSM, ensures that all of the builder's references are released.
pub struct BuiltFsm;

impl<TFsm, TContext> FsmBuilder<TFsm, TContext>
	where TFsm: FsmBackend
{
	/// Sets the initial state of the state machine. Required!
	pub fn initial_state<TSTate>(&mut self) {
		
	}

	/// Defines multiple initial states for multiple regions of the FSM. The type has to be a tuple
	/// of the initial states for each region.
	///
	/// Example : `fsm.initial_states<(StateA, StateX)>()`
	pub fn initial_states<TStates>(&mut self) {

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