use crate::{FsmBackend, lib::*};

use super::{FsmEventBuilderState, FsmStateBuilder};


pub struct FsmSubMachineBuilder<TFsm, TContext, TSubMachine> {
	pub (crate) _fsm: PhantomData<TFsm>,
	pub (crate) _ctx: PhantomData<TContext>,
	pub (crate) _sub: PhantomData<TSubMachine>,
	pub (crate) _state_builder: FsmStateBuilder<TFsm, TContext, TSubMachine>
}

impl<TFsm, TContext, TSubMachine> FsmSubMachineBuilder<TFsm, TContext, TSubMachine>
	where TFsm: FsmBackend<Context = TContext>,	TSubMachine: FsmBackend
{
	/// Adds a context adapter. A referenced context of the parent machine is provided, and a new
	/// instance of the submachine's context has to be instantiated.
	pub fn with_context<TCtxFactory: Fn(&<TFsm as FsmBackend>::Context) -> <TSubMachine as FsmBackend>::Context>(&mut self, _sub_context_factory: TCtxFactory) -> &Self {
		self
	}

	/// What happens if we receive this event and we are in this submachine's state right now?
	pub fn on_event<TEvent>(&self) -> FsmEventBuilderState<TFsm, TContext, TEvent, TSubMachine> {
		FsmEventBuilderState {
			_state_builder: &self._state_builder,
			_event: PhantomData::default()
		}
	}
}