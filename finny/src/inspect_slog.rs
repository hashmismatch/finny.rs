use slog::{OwnedKV, debug, info, o};

use crate::{FsmBackend, FsmEvent, Inspect};
use super::lib::*;

pub struct InspectSlog {
    logger: slog::Logger
}

pub struct InspectSlogEventContext {
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
    type ContextDispatchEvent = InspectSlogEventContext;

    fn on_dispatch_event(&self, fsm: &crate::FsmBackendImpl<F>, event: &FsmEvent<<F as FsmBackend>::Events>) -> InspectSlogEventContext {
        
        let log_event = format!("{:?}", event);
        let log_current_states = format!("{:?}", fsm.current_states);
        let kv = o!("event" => log_event, "current_states" => log_current_states);
        info!(self.logger, "Dispatching the event."; &kv);
        InspectSlogEventContext {
            logger: self.logger.new(kv)
        }
    }

    fn on_state_enter<State>(&self, _fsm: &crate::FsmBackendImpl<F>, ctx: &mut InspectSlogEventContext) where <F as FsmBackend>::States: AsRef<State> {
        info!(ctx.logger, "Entering state {:?}", type_name::<State>());
    }

    fn on_state_exit<State>(&self, _fsm: &crate::FsmBackendImpl<F>, ctx: &mut InspectSlogEventContext) where <F as FsmBackend>::States: AsRef<State> {
        info!(ctx.logger, "Exiting state {:?}", type_name::<State>());
    }

    fn on_action(&self) {
        //todo!()
    }

    fn on_guard<T, Guard>(&self) {
        //todo!()
    }

    fn on_matched_transition<T>(&self, fsm: &crate::FsmBackendImpl<F>, region: crate::FsmRegionId, ctx: &mut Self::ContextDispatchEvent) {
        
    }
}


