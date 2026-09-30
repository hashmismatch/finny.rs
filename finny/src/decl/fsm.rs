use crate::{FsmBackendImpl, lib::*};

use crate::FsmBackend;
use super::{FsmStateBuilder, FsmSubMachineBuilder};

/// The main builder-API for defining your Finny state machine.
#[derive(Default)]
pub struct FsmBuilder<TFsm, TContext> {
    pub _fsm: PhantomData<TFsm>,
    pub _context: PhantomData<TContext>
}

/// The consumed struct of the FSM, ensures that all of the builder's references are released.
pub struct BuiltFsm;

impl<TFsm, TContext> FsmBuilder<TFsm, TContext>
	where TFsm: FsmBackend<Context = TContext>
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

	/// Require the `Debug` trait on the Events.
	pub fn events_debug(&mut self) {
		
	}

	/// Serialize the FSM's context, states and events with `serde`, for example for the
	/// inspection frontends. Requires `serde::Serialize` on all of them, and on the sub-machines
	/// that also have to opt in. Needs the `serde` feature.
	pub fn serde(&mut self) {

	}

	/// Adds some information about a state.
	pub fn state<TState>(&mut self) -> FsmStateBuilder<TFsm, TContext, TState> {
		FsmStateBuilder::new()
	}

	/// Adds a sub machine
	pub fn sub_machine<TSubFsm>(&mut self) -> FsmSubMachineBuilder<TFsm, TContext, TSubFsm>
		where TSubFsm: FsmBackend
	{
		FsmSubMachineBuilder::new()
	}

	/// Builds the final machine. Has to be returned from the definition function.
    pub fn build(self) -> BuiltFsm {
        BuiltFsm
    }
}