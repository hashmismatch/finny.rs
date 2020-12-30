use std::marker::PhantomData;

use crate::{FsmCore, FsmEventQueue, fsm::EventContext};

use super::fsm::FsmBuilder;

pub struct FsmEventBuilder<'a, TFsm, TContext, TEvent> {
    pub (crate) _event: PhantomData<TEvent>,
	pub (crate) _fsm: &'a FsmBuilder<TFsm, TContext>
}

impl<'a, TFsm, TContext, TEvent> FsmEventBuilder<'a, TFsm, TContext, TEvent> {
    pub fn transition_from<TStateFrom>(self) -> FsmEventBuilderTransition<'a, TFsm, TContext, TEvent, TStateFrom> {
        FsmEventBuilderTransition {
            _event_builder: self,
            _state_from: PhantomData::default()
        }
    }
}

pub struct FsmEventBuilderTransition<'a, TFsm, TContext, TEvent, TStateFrom> {
    _event_builder: FsmEventBuilder<'a, TFsm, TContext, TEvent>,
    _state_from: PhantomData<TStateFrom>
}

impl<'a, TFsm, TContext, TEvent, TStateFrom> FsmEventBuilderTransition<'a, TFsm, TContext, TEvent, TStateFrom> {
    pub fn to<TStateTo>(self) -> FsmEventBuilderTransitionFull<'a, TFsm, TContext, TEvent, TStateFrom, TStateTo> {
        FsmEventBuilderTransitionFull {
            _transition_from: self,
            _state_to: PhantomData::default()
        }
    }
}

pub struct FsmEventBuilderTransitionFull<'a, TFsm, TContext, TEvent, TStateFrom, TStateTo> {
    _transition_from: FsmEventBuilderTransition<'a, TFsm, TContext, TEvent, TStateFrom>,
    _state_to: PhantomData<TStateTo>
}

impl<'a, TFsm, TContext, TEvent, TStateFrom, TStateTo> FsmEventBuilderTransitionFull<'a, TFsm, TContext, TEvent, TStateFrom, TStateTo> 
    where TFsm: FsmCore
{
    pub fn action<TAction: Fn(&TEvent, &mut EventContext<'a, TFsm, dyn FsmEventQueue<<TFsm as FsmCore>::Events>>, &mut TStateFrom, &mut TStateTo)>(&mut self, _action: TAction) -> &mut Self {
        self
    }

    pub fn guard<TGuard: Fn(&TEvent, &EventContext<'a, TFsm, dyn FsmEventQueue<<TFsm as FsmCore>::Events>>) -> bool>(&mut self, _guard: TGuard) -> &mut Self {
        self
    }
}