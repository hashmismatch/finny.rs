use crate::{EventContext, FsmBackend, lib::*};

use super::{AsyncMode, FsmEventBuilderState, FsmQueueMock, FsmStateBuilder, SyncMode};


pub struct FsmSubMachineBuilder<TFsm, TContext, TSubMachine, M = SyncMode> {
	pub (crate) _fsm: PhantomData<TFsm>,
	pub (crate) _ctx: PhantomData<TContext>,
	pub (crate) _sub: PhantomData<TSubMachine>,
	pub (crate) _state_builder: FsmStateBuilder<TFsm, TContext, TSubMachine, M>
}

impl<TFsm, TContext, TSubMachine, M> FsmSubMachineBuilder<TFsm, TContext, TSubMachine, M> {
	pub (crate) fn new() -> Self {
		FsmSubMachineBuilder {
			_fsm: PhantomData::default(),
			_ctx: PhantomData::default(),
			_sub: PhantomData::default(),
			_state_builder: FsmStateBuilder::new()
		}
	}
}

impl<TFsm, TContext, TSubMachine> FsmSubMachineBuilder<TFsm, TContext, TSubMachine, SyncMode>
	where TFsm: FsmBackend<Context = TContext>,	TSubMachine: FsmBackend
{
	/// Adds a context adapter. A referenced context of the parent machine is provided, and a new
	/// instance of the submachine's context has to be instantiated.
	pub fn with_context<TCtxFactory: Fn(&TContext) -> <TSubMachine as FsmBackend>::Context>(&mut self, _sub_context_factory: TCtxFactory) -> &Self {
		self
	}

	/// Execute this action when entering the sub-machine state.
	pub fn on_entry<'a, TAction: Fn(&mut TSubMachine, &mut EventContext<'a, TFsm, FsmQueueMock<TFsm>>)>(&self, _action: TAction) -> &Self {
		self
	}

	/// Execute this action when exiting the sub-machine state.
	pub fn on_exit<'a, TAction: Fn(&mut TSubMachine, &mut EventContext<'a, TFsm, FsmQueueMock<TFsm>>)>(&self, _action: TAction) -> &Self {
		self
	}
}

impl<TFsm, TContext, TSubMachine> FsmSubMachineBuilder<TFsm, TContext, TSubMachine, AsyncMode>
	where TFsm: FsmBackend<Context = TContext>,	TSubMachine: FsmBackend
{
	/// Adds a context adapter. A referenced context of the parent machine is provided, and a new
	/// instance of the submachine's context has to be returned. It's converted into the submachine's
	/// `Arc` context with `Into`.
	pub fn with_context<TSubContext, TCtxFactory>(&mut self, _sub_context_factory: TCtxFactory) -> &Self
		where TSubContext: Into<<TSubMachine as FsmBackend>::Context>, TCtxFactory: Fn(&TContext) -> TSubContext
	{
		self
	}

	/// Execute this async action when entering the sub-machine state.
	pub fn on_entry<'a, TAction: AsyncFn(&mut TSubMachine, &mut EventContext<'a, TFsm, FsmQueueMock<TFsm>>)>(&self, _action: TAction) -> &Self {
		self
	}

	/// Execute this async action when exiting the sub-machine state.
	pub fn on_exit<'a, TAction: AsyncFn(&mut TSubMachine, &mut EventContext<'a, TFsm, FsmQueueMock<TFsm>>)>(&self, _action: TAction) -> &Self {
		self
	}
}

impl<TFsm, TContext, TSubMachine, M> FsmSubMachineBuilder<TFsm, TContext, TSubMachine, M>
	where TFsm: FsmBackend<Context = TContext>,	TSubMachine: FsmBackend
{
	/// What happens if we receive this event and we are in this submachine's state right now?
	pub fn on_event<TEvent>(&self) -> FsmEventBuilderState<'_, TFsm, TContext, TEvent, TSubMachine, M> {
		FsmEventBuilderState {
			_state_builder: &self._state_builder,
			_event: PhantomData::default()
		}
	}
}
