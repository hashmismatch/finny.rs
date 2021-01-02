use lib::*;

use crate::{FsmBackend, FsmEventQueue, fsm::EventContext};

use super::{FsmStateBuilder, fsm::FsmBuilder};

pub struct FsmEventBuilderState<'a, TFsm, TContext, TEvent, TState> {
    pub (crate) _state_builder: &'a FsmStateBuilder<TFsm, TContext, TState>,
    pub (crate) _event: PhantomData<TEvent>
}

impl<'a, TFsm, TContext, TEvent, TState> FsmEventBuilderState<'a, TFsm, TContext, TEvent, TState> {
    /// An internal transition doesn't trigger the state's entry and exit actions, as opposed to self-transitions.
    pub fn internal_transition<'b>(&'b self) -> FsmEventBuilderTransition<'b, TFsm, TContext, TEvent, TState> {
        FsmEventBuilderTransition {
            _state_event_builder: self
        }
    }

    /// A self transition triggers this state's entry and exit actions, while an internal transition does not.
    pub fn self_transition<'b>(&'b self) -> FsmEventBuilderTransition<'b, TFsm, TContext, TEvent, TState> {
        FsmEventBuilderTransition {
            _state_event_builder: self
        }
    }

    /// Transition into this state. The transition can have a guard and an action.
    pub fn transition_to<'b, TStateTo>(&'b self) -> FsmEventBuilderTransitionFull<'b, TFsm, TContext, TEvent, TState, TStateTo> {
        FsmEventBuilderTransitionFull {
            _transition_from: self,
            _state_to: PhantomData::default()
        }
    }
}


pub struct FsmEventBuilderTransition<'a, TFsm, TContext, TEvent, TState> {
    _state_event_builder: &'a FsmEventBuilderState<'a, TFsm, TContext, TEvent, TState>
}

impl<'a, TFsm, TContext, TEvent, TState> FsmEventBuilderTransition<'a, TFsm, TContext, TEvent, TState>
    where TFsm: FsmBackend
{
    pub fn action<TAction: Fn(&TEvent, &mut EventContext<'a, TFsm, dyn FsmEventQueue<<TFsm as FsmBackend>::Events>>, &mut TState)>(&mut self, _action: TAction) -> &mut Self {
        self
    }

    pub fn guard<TGuard: Fn(&TEvent, &EventContext<'a, TFsm, dyn FsmEventQueue<<TFsm as FsmBackend>::Events>>) -> bool>(&mut self, _guard: TGuard) -> &mut Self {
        self
    }
}


pub struct FsmEventBuilderTransitionFull<'a, TFsm, TContext, TEvent, TStateFrom, TStateTo> {
    _transition_from: &'a FsmEventBuilderState<'a, TFsm, TContext, TEvent, TStateFrom>,
    _state_to: PhantomData<TStateTo>
}

impl<'a, TFsm, TContext, TEvent, TStateFrom, TStateTo> FsmEventBuilderTransitionFull<'a, TFsm, TContext, TEvent, TStateFrom, TStateTo> 
    where TFsm: FsmBackend
{
    pub fn action<TAction: Fn(&TEvent, &mut EventContext<'a, TFsm, dyn FsmEventQueue<<TFsm as FsmBackend>::Events>>, &mut TStateFrom, &mut TStateTo)>(&mut self, _action: TAction) -> &mut Self {
        self
    }

    pub fn guard<TGuard: Fn(&TEvent, &EventContext<'a, TFsm, dyn FsmEventQueue<<TFsm as FsmBackend>::Events>>) -> bool>(&mut self, _guard: TGuard) -> &mut Self {
        self
    }
}