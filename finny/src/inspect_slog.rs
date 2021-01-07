use slog::{OwnedKV, debug, info, o};

use crate::{FsmBackend, FsmDispatchResult, FsmEvent, Inspect};
use super::lib::*;

pub struct InspectSlog {
    logger: slog::Logger
}

pub struct InspectSlogSubContext {
    logger: slog::Logger
}

impl InspectSlog {
    pub fn new(logger: Option<slog::Logger>) -> Self {
        InspectSlog {
            logger: logger.unwrap_or(slog::Logger::root(slog::Discard, o!()))
        }
    }
}

impl<F> Inspect<F> for InspectSlog 
    where F: FsmBackend, <F as FsmBackend>::Events: Debug
{
    type CtxEvent = InspectSlogSubContext;
    type CtxTransition = InspectSlogSubContext;

    fn on_dispatch_event(&self, fsm: &crate::FsmBackendImpl<F>, event: &FsmEvent<<F as FsmBackend>::Events>) -> InspectSlogSubContext {
        
        let log_event = format!("{:?}", event);
        let log_current_states = format!("{:?}", fsm.current_states);
        let kv = o!("event" => log_event, "current_states" => log_current_states);
        info!(self.logger, "Dispatching the event."; &kv);
        InspectSlogSubContext {
            logger: self.logger.new(kv)
        }
    }

    fn on_dispatched_event(&self, _fsm: &crate::FsmBackendImpl<F>, ctx: Self::CtxEvent, result: &FsmDispatchResult) {
        info!(ctx.logger, "Finished the dispatching.");
    }

    fn on_state_enter<State>(&self, _fsm: &crate::FsmBackendImpl<F>, ctx: &mut InspectSlogSubContext) where <F as FsmBackend>::States: AsRef<State> {
        info!(ctx.logger, "Entering state {:?}", type_name::<State>());
    }

    fn on_state_exit<State>(&self, _fsm: &crate::FsmBackendImpl<F>, ctx: &mut InspectSlogSubContext) where <F as FsmBackend>::States: AsRef<State> {
        info!(ctx.logger, "Exiting state {:?}", type_name::<State>());
    }

    fn on_action<T>(&self, _fsm: &crate::FsmBackendImpl<F>, ctx: &mut InspectSlogSubContext) {
        info!(ctx.logger, "Executing the action for {transition}", transition = type_name::<T>());
    }

    fn on_guard<T>(&self, _fsm: &crate::FsmBackendImpl<F>, ctx: &mut Self::CtxEvent, guard_result: bool) {
        let guard_result = format!("{}", guard_result);
        let transition = type_name::<T>();
        info!(ctx.logger, "The guard for {transition} evaluated to {guard_result}", transition = transition, guard_result = &guard_result);
    }

    fn on_matched_transition<T>(&self, fsm: &crate::FsmBackendImpl<F>, region: crate::FsmRegionId, ctx: &mut InspectSlogSubContext) -> InspectSlogSubContext {
        let transition = type_name::<T>();
        let kv = o!("transition" => transition, "region" => region);
        info!(ctx.logger, "Matched transition {transition} in region {region}", transition = transition, region = region);

        InspectSlogSubContext {
            logger: ctx.logger.new(kv)
        }
    }
}


