use crate::{EventContext, FsmBackend, FsmEventQueue};


/// A state's entry and exit actions.
pub trait FsmState<F: FsmBackend> {
    /// Action that is executed whenever this state is being entered.
    fn on_entry<'a, Q: FsmEventQueue<<F as FsmBackend>::Events>>(&mut self, context: &mut EventContext<'a, F, Q>);
    /// Action that is executed whenever this state is being exited.
    fn on_exit<'a, Q: FsmEventQueue<<F as FsmBackend>::Events>>(&mut self, context: &mut EventContext<'a, F, Q>);
}

/// Check if this transition is allowed to be entered.
pub trait FsmTransitionGuard<F: FsmBackend, E> {
    /// Return a boolean value whether this transition is usable at the moment. The check shouln't mutate any structures.
    fn guard<'a, Q: FsmEventQueue<<F as FsmBackend>::Events>>(event: &E, context: &EventContext<'a, F, Q>) -> bool;
}

/// A transition's action that can operate on both the exit and entry states.
pub trait FsmTransitionAction<F: FsmBackend, E, TStateFrom, TStateTo> {
    /// This action is executed after the first state's exit event, and just before the second event's entry action. It can mutate both states.
    fn action<'a, Q: FsmEventQueue<<F as FsmBackend>::Events>>(event: &E, context: &mut EventContext<'a, F, Q>, from: &mut TStateFrom, to: &mut TStateTo);
}

/// An internal or self action can only mutate itself.
pub trait FsmAction<F: FsmBackend, E, State> {
    /// This action is executed as part of an internal or self transition.
    fn action<'a, Q: FsmEventQueue<<F as FsmBackend>::Events>>(event: &E, context: &mut EventContext<'a, F, Q>, state: &mut State);
}