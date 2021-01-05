//! All of these traits will be implemented by the procedural code generator.

use crate::{EventContext, FsmBackend, FsmEventQueue, FsmFrontend, FsmStateTransitionAsMut};

/// A state's entry and exit actions.
pub trait FsmState<F: FsmBackend> {
    /// Action that is executed whenever this state is being entered.
    fn on_entry<'a, Q: FsmEventQueue<F>>(&mut self, context: &mut EventContext<'a, F, Q>);
    /// Action that is executed whenever this state is being exited.
    fn on_exit<'a, Q: FsmEventQueue<F>>(&mut self, context: &mut EventContext<'a, F, Q>);

    fn execute_on_entry<Q>(frontend: &mut FsmFrontend<F, Q>) 
        where Q: FsmEventQueue<F>, <F as FsmBackend>::States: AsMut<Self>
    {
        let mut event_context = EventContext {
            context: &mut frontend.backend.context,
            queue: &mut frontend.queue
        };

        let state: &mut Self = frontend.backend.states.as_mut();

        state.on_entry(&mut event_context);
    }

    fn execute_on_exit<Q>(frontend: &mut FsmFrontend<F, Q>) 
        where Q: FsmEventQueue<F>, <F as FsmBackend>::States: AsMut<Self>
    {
        let mut event_context = EventContext {
            context: &mut frontend.backend.context,
            queue: &mut frontend.queue
        };

        let state: &mut Self = frontend.backend.states.as_mut();

        state.on_exit(&mut event_context);
    }
}

/// Check if this transition is allowed to be entered.
pub trait FsmTransitionGuard<F: FsmBackend, E> {
    /// Return a boolean value whether this transition is usable at the moment. The check shouln't mutate any structures.
    fn guard<'a, Q: FsmEventQueue<F>>(event: &E, context: &EventContext<'a, F, Q>) -> bool;

    fn execute_guard<Q: FsmEventQueue<F>>(frontend: &mut FsmFrontend<F, Q>, event: &E) -> bool {
        let event_context = EventContext {
            context: &mut frontend.backend.context,
            queue: &mut frontend.queue
        };

        Self::guard(event, &event_context)
    }
}

/// A transition's action that can operate on both the exit and entry states.
pub trait FsmTransitionAction<F: FsmBackend, E, TStateFrom, TStateTo> {
    /// This action is executed after the first state's exit event, and just before the second event's entry action. It can mutate both states.
    fn action<'a, Q: FsmEventQueue<F>>(event: &E, context: &mut EventContext<'a, F, Q>, from: &mut TStateFrom, to: &mut TStateTo);

    fn execute_action_transition<Q: FsmEventQueue<F>>(frontend: &mut FsmFrontend<F, Q>, event: &E)
        where <F as FsmBackend>::States: FsmStateTransitionAsMut<TStateFrom, TStateTo>
    {
        let mut event_context = EventContext {
            context: &mut frontend.backend.context,
            queue: &mut frontend.queue
        };

        let states: (&mut TStateFrom, &mut TStateTo) = frontend.backend.states.as_state_transition_mut();

        Self::action(event, &mut event_context, states.0, states.1);
    }
}

/// An internal or self action can only mutate itself.
pub trait FsmAction<F: FsmBackend, E, State> {
    /// This action is executed as part of an internal or self transition.
    fn action<'a, Q: FsmEventQueue<F>>(event: &E, context: &mut EventContext<'a, F, Q>, state: &mut State);

    fn execute_action<Q: FsmEventQueue<F>>(frontend: &mut FsmFrontend<F, Q>, event: &E)
        where <F as FsmBackend>::States: AsMut<State>
    {
        let mut event_context = EventContext {
            context: &mut frontend.backend.context,
            queue: &mut frontend.queue
        };

        let state: &mut State = frontend.backend.states.as_mut();

        Self::action(event, &mut event_context, state);
    }
}