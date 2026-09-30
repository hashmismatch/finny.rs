use crate::lib::*;

use crate::{FsmBackend, fsm::EventContext};
use super::{AsyncMode, FsmQueueMock, FsmStateBuilder, SyncMode};

pub struct FsmEventBuilderState<'a, TFsm, TContext, TEvent, TState, M = SyncMode> {
    pub (crate) _state_builder: &'a FsmStateBuilder<TFsm, TContext, TState, M>,
    pub (crate) _event: PhantomData<TEvent>
}

impl<'a, TFsm, TContext, TEvent, TState, M> FsmEventBuilderState<'a, TFsm, TContext, TEvent, TState, M> {
    /// An internal transition doesn't trigger the state's entry and exit actions, as opposed to self-transitions.
    pub fn internal_transition<'b>(&'b self) -> FsmEventBuilderTransition<'b, TFsm, TContext, TEvent, TState, M> {
        FsmEventBuilderTransition {
            _state_event_builder: self
        }
    }

    /// A self transition triggers this state's entry and exit actions, while an internal transition does not.
    pub fn self_transition<'b>(&'b self) -> FsmEventBuilderTransition<'b, TFsm, TContext, TEvent, TState, M> {
        FsmEventBuilderTransition {
            _state_event_builder: self
        }
    }

    /// Transition into this state. The transition can have a guard and an action.
    pub fn transition_to<'b, TStateTo>(&'b self) -> FsmEventBuilderTransitionFull<'b, TFsm, TContext, TEvent, TState, TStateTo, M> {
        FsmEventBuilderTransitionFull {
            _transition_from: self,
            _state_to: PhantomData::default()
        }
    }
}


pub struct FsmEventBuilderTransition<'a, TFsm, TContext, TEvent, TState, M = SyncMode> {
    _state_event_builder: &'a FsmEventBuilderState<'a, TFsm, TContext, TEvent, TState, M>
}

impl<'a, TFsm, TContext, TEvent, TState> FsmEventBuilderTransition<'a, TFsm, TContext, TEvent, TState, SyncMode>
    where TFsm: FsmBackend
{
    /// An action that happens when the currently active state receives this event. No transitions.
    pub fn action<TAction: Fn(&TEvent, &mut EventContext<'a, TFsm, FsmQueueMock<TFsm>>, &mut TState)>(&mut self, _action: TAction) -> &mut Self {
        self
    }
}

impl<'a, TFsm, TContext, TEvent, TState> FsmEventBuilderTransition<'a, TFsm, TContext, TEvent, TState, AsyncMode>
    where TFsm: FsmBackend
{
    /// An async action that happens when the currently active state receives this event. No transitions.
    /// Use an async closure: `async |event, ctx, state| { .. }`.
    pub fn action<TAction: AsyncFn(&TEvent, &mut EventContext<'a, TFsm, FsmQueueMock<TFsm>>, &mut TState)>(&mut self, _action: TAction) -> &mut Self {
        self
    }
}

impl<'a, TFsm, TContext, TEvent, TState, M> FsmEventBuilderTransition<'a, TFsm, TContext, TEvent, TState, M>
    where TFsm: FsmBackend
{
    /// A guard for executing this action. Guards are always synchronous.
    pub fn guard<TGuard: Fn(&TEvent, &EventContext<'a, TFsm, FsmQueueMock<TFsm>>, &<TFsm as FsmBackend>::States) -> bool>(&mut self, _guard: TGuard) -> &mut Self {
        self
    }

    /// A type for this transition. The struct for the transition will be generated.
    pub fn with_transition_ty<TTransition>(&mut self) -> &mut Self {
        self
    }
}


pub struct FsmEventBuilderTransitionFull<'a, TFsm, TContext, TEvent, TStateFrom, TStateTo, M = SyncMode> {
    _transition_from: &'a FsmEventBuilderState<'a, TFsm, TContext, TEvent, TStateFrom, M>,
    _state_to: PhantomData<TStateTo>
}

impl<'a, TFsm, TContext, TEvent, TStateFrom, TStateTo> FsmEventBuilderTransitionFull<'a, TFsm, TContext, TEvent, TStateFrom, TStateTo, SyncMode>
    where TFsm: FsmBackend
{
    /// An action that happens between the transitions from the two states.
    pub fn action<TAction: Fn(&TEvent, &mut EventContext<'a, TFsm, FsmQueueMock<TFsm>>, &mut TStateFrom, &mut TStateTo)>(&mut self, _action: TAction) -> &mut Self {
        self
    }
}

impl<'a, TFsm, TContext, TEvent, TStateFrom, TStateTo> FsmEventBuilderTransitionFull<'a, TFsm, TContext, TEvent, TStateFrom, TStateTo, AsyncMode>
    where TFsm: FsmBackend
{
    /// An async action that happens between the transitions from the two states.
    /// Use an async closure: `async |event, ctx, from, to| { .. }`.
    pub fn action<TAction: AsyncFn(&TEvent, &mut EventContext<'a, TFsm, FsmQueueMock<TFsm>>, &mut TStateFrom, &mut TStateTo)>(&mut self, _action: TAction) -> &mut Self {
        self
    }
}

impl<'a, TFsm, TContext, TEvent, TStateFrom, TStateTo, M> FsmEventBuilderTransitionFull<'a, TFsm, TContext, TEvent, TStateFrom, TStateTo, M>
    where TFsm: FsmBackend
{
    /// A guard for starting this transition from one state to another, including executing the action.
    /// Guards are always synchronous.
    pub fn guard<TGuard: Fn(&TEvent, &EventContext<'a, TFsm, FsmQueueMock<TFsm>>, &<TFsm as FsmBackend>::States) -> bool>(&mut self, _guard: TGuard) -> &mut Self {
        self
    }

    /// A type for this transition. The struct for the transition will be generated.
    pub fn with_transition_ty<TTransition>(&mut self) -> &mut Self {
        self
    }
}
