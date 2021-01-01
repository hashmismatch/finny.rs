use crate::{EventContext, FsmBackend, FsmEventQueue};


pub trait FsmState<F: FsmBackend> {
    fn on_entry<'a, Q: FsmEventQueue<<F as FsmBackend>::Events>>(&mut self, context: &mut EventContext<'a, F, Q>);
    fn on_exit<'a, Q: FsmEventQueue<<F as FsmBackend>::Events>>(&mut self, context: &mut EventContext<'a, F, Q>);
}

pub trait FsmTransitionGuard<F: FsmBackend, E> {
    fn guard<'a, Q: FsmEventQueue<<F as FsmBackend>::Events>>(event: &E, context: &EventContext<'a, F, Q>) -> bool;
}

pub trait FsmTransitionAction<F: FsmBackend, E, TStateFrom, TStateTo> {
    fn action<'a, Q: FsmEventQueue<<F as FsmBackend>::Events>>(event: &E, context: &mut EventContext<'a, F, Q>, from: &mut TStateFrom, to: &mut TStateTo);
}

pub trait FsmAction<F: FsmBackend, E, State> {
    fn action<'a, Q: FsmEventQueue<<F as FsmBackend>::Events>>(event: &E, context: &mut EventContext<'a, F, Q>, state: &mut State);
}