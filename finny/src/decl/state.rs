use crate::{TimerFsmSettings, lib::*};

use crate::{EventContext, FsmBackend};
use super::{AsyncMode, FsmQueueMock, SyncMode, event::FsmEventBuilderState};

pub struct FsmStateBuilder<TFsm, TContext, TState, M = SyncMode> {
	pub (crate) _state: PhantomData<TState>,
	pub (crate) _fsm: PhantomData<TFsm>,
	pub (crate) _context: PhantomData<TContext>,
	pub (crate) _mode: PhantomData<M>
}

impl<TFsm, TContext, TState, M> FsmStateBuilder<TFsm, TContext, TState, M> {
	pub (crate) fn new() -> Self {
		FsmStateBuilder {
			_state: PhantomData::default(),
			_fsm: PhantomData::default(),
			_context: PhantomData::default(),
			_mode: PhantomData::default()
		}
	}
}

impl<TFsm, TContext, TState> FsmStateBuilder<TFsm, TContext, TState, SyncMode>
	where TFsm: FsmBackend
{
	/// Execute this action when entering the state.
	pub fn on_entry<'a, TAction: Fn(&mut TState, &mut EventContext<'a, TFsm, FsmQueueMock<TFsm>>)>(&self, _action: TAction) -> &Self {
		self
	}

	/// Execute this action when exiting the state.
	pub fn on_exit<'a, TAction: Fn(&mut TState, &mut EventContext<'a, TFsm, FsmQueueMock<TFsm>>)>(&self, _action: TAction) -> &Self {
		self
	}
}

impl<TFsm, TContext, TState> FsmStateBuilder<TFsm, TContext, TState, AsyncMode>
	where TFsm: FsmBackend
{
	/// Execute this async action when entering the state. Use an async closure: `async |state, ctx| { .. }`.
	pub fn on_entry<'a, TAction: AsyncFn(&mut TState, &mut EventContext<'a, TFsm, FsmQueueMock<TFsm>>)>(&self, _action: TAction) -> &Self {
		self
	}

	/// Execute this async action when exiting the state. Use an async closure: `async |state, ctx| { .. }`.
	pub fn on_exit<'a, TAction: AsyncFn(&mut TState, &mut EventContext<'a, TFsm, FsmQueueMock<TFsm>>)>(&self, _action: TAction) -> &Self {
		self
	}
}

impl<TFsm, TContext, TState, M> FsmStateBuilder<TFsm, TContext, TState, M>
	where TFsm: FsmBackend
{
	/// What happens if we receive this event and we are in this state right now?
	pub fn on_event<TEvent>(&self) -> FsmEventBuilderState<'_, TFsm, TContext, TEvent, TState, M> {
		FsmEventBuilderState {
			_state_builder: self,
			_event: PhantomData::default()
		}
	}

	/// Start a new timer when entering this state. The timer should be unit struct with a implemented
	/// Default trait. The timer is setup within a closure and the trigger is another closure
	/// that returns an event to be enqueued in the FSM.
	pub fn on_entry_start_timer<FSetup, FTrigger>(&self, _setup: FSetup, _trigger: FTrigger) -> FsmStateTimerBuilder<'_, TFsm, TContext, TState, M>
		where
			FSetup: Fn(&mut TContext, &mut TimerFsmSettings),
			FTrigger: Fn(&TContext, &TState) -> Option< <TFsm as FsmBackend>::Events >
	{
		FsmStateTimerBuilder {
			_state: self
		}
	}
}

pub struct FsmStateTimerBuilder<'a, TFsm, TContext, TState, M = SyncMode> {
	_state: &'a FsmStateBuilder<TFsm, TContext, TState, M>
}

impl<'a, TFsm, TContext, TState, M> FsmStateTimerBuilder<'a, TFsm, TContext, TState, M>
	where TFsm: FsmBackend
{
	/// Assign this type to the timer. The struct for it will be auto-generated.
	pub fn with_timer_ty<TTimer>(self) {

	}
}
