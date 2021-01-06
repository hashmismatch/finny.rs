//! All of these traits will be implemented by the procedural code generator.

use crate::{EventContext, FsmBackend, FsmCurrentState, FsmEvent, FsmEventQueue, FsmFrontend, FsmRegionId, FsmStateTransitionAsMut, FsmStates, Inspect};

/// A state's entry and exit actions.
pub trait FsmState<F: FsmBackend> {
    /// Action that is executed whenever this state is being entered.
    fn on_entry<'a, Q: FsmEventQueue<F>>(&mut self, context: &mut EventContext<'a, F, Q>);
    /// Action that is executed whenever this state is being exited.
    fn on_exit<'a, Q: FsmEventQueue<F>>(&mut self, context: &mut EventContext<'a, F, Q>);

    fn execute_on_entry<Q, I>(frontend: &mut FsmFrontend<F, Q, I>, region: FsmRegionId) 
        where Q: FsmEventQueue<F>, I: Inspect<F>, <F as FsmBackend>::States: AsMut<Self>
    {
        let mut event_context = EventContext {
            context: &mut frontend.backend.context,
            queue: &mut frontend.queue,
            region
        };

        let state: &mut Self = frontend.backend.states.as_mut();

        state.on_entry(&mut event_context);
    }

    fn execute_on_exit<Q, I>(frontend: &mut FsmFrontend<F, Q, I>, region: FsmRegionId) 
        where Q: FsmEventQueue<F>, I: Inspect<F>, <F as FsmBackend>::States: AsMut<Self>
    {
        let mut event_context = EventContext {
            context: &mut frontend.backend.context,
            queue: &mut frontend.queue,
            region
        };

        let state: &mut Self = frontend.backend.states.as_mut();

        state.on_exit(&mut event_context);
    }

    fn fsm_state() -> <<F as FsmBackend>::States as FsmStates>::StateKind;
}

/// Check if this transition is allowed to be entered.
pub trait FsmTransitionGuard<F: FsmBackend, E> {
    /// Return a boolean value whether this transition is usable at the moment. The check shouln't mutate any structures.
    fn guard<'a, Q: FsmEventQueue<F>>(event: &E, context: &EventContext<'a, F, Q>) -> bool;

    fn execute_guard<Q: FsmEventQueue<F>, I>(frontend: &mut FsmFrontend<F, Q, I>, event: &E, region: FsmRegionId) -> bool
        where I: Inspect<F>
    {
        let event_context = EventContext {
            context: &mut frontend.backend.context,
            queue: &mut frontend.queue,
            region
        };

        Self::guard(event, &event_context)
    }
}


pub trait FsmTransitionFsmStart<F: FsmBackend, TInitialState> {
    fn execute_transition<Q: FsmEventQueue<F>, I >(frontend: &mut FsmFrontend<F, Q, I>, fsm_event: &FsmEvent<<F as FsmBackend>::Events>, region: FsmRegionId)
        //where <F as FsmBackend>::States: AsMut<TInitialState>, I: Inspect<F>
        where I: Inspect<F>, TInitialState: FsmState<F>, <F as FsmBackend>::States: AsMut<TInitialState>
    {
        <TInitialState>::execute_on_entry(frontend, region);
        
        let cs = frontend.backend.current_states.as_mut();
        cs[region] = FsmCurrentState::State(<TInitialState>::fsm_state());
    }
}

/// A transition's action that can operate on both the exit and entry states.
pub trait FsmTransitionAction<F: FsmBackend, E, TStateFrom, TStateTo> {
    /// This action is executed after the first state's exit event, and just before the second event's entry action. It can mutate both states.
    fn action<'a, Q: FsmEventQueue<F>>(event: &E, context: &mut EventContext<'a, F, Q>, from: &mut TStateFrom, to: &mut TStateTo);


    /*
    fn execute_action_transition<Q: FsmEventQueue<F>, I>(frontend: &mut FsmFrontend<F, Q, I>, event: &E, region: FsmRegionId)
        where <F as FsmBackend>::States: FsmStateTransitionAsMut<TStateFrom, TStateTo>, I: Inspect<F>
    {
        let mut event_context = EventContext {
            context: &mut frontend.backend.context,
            queue: &mut frontend.queue,
            region
        };

        let states: (&mut TStateFrom, &mut TStateTo) = frontend.backend.states.as_state_transition_mut();

        Self::action(event, &mut event_context, states.0, states.1);
    }
    */

    //fn execute_transition<Q: FsmEventQueue<F>, I>(frontend: &mut FsmFrontend<F, Q, I>, event: &FsmEvent<<F as FsmBackend>::Events>, region: FsmRegionId)
    fn execute_transition<Q: FsmEventQueue<F>, I>(frontend: &mut FsmFrontend<F, Q, I>, event: &E, region: FsmRegionId)
        where 
            I: Inspect<F>,
            <F as FsmBackend>::States: FsmStateTransitionAsMut<TStateFrom, TStateTo>
    {
        let mut event_context = EventContext {
            context: &mut frontend.backend.context,
            queue: &mut frontend.queue,
            region
        };

        let states: (&mut TStateFrom, &mut TStateTo) = frontend.backend.states.as_state_transition_mut();

        // todo: state exit
            
        Self::action(event, &mut event_context, states.0, states.1);

        // todo: state enter

        // todo: change current state
    }
}

/// An internal or self action can only mutate itself.
pub trait FsmAction<F: FsmBackend, E, State> {
    /// This action is executed as part of an internal or self transition.
    fn action<'a, Q: FsmEventQueue<F>>(event: &E, context: &mut EventContext<'a, F, Q>, state: &mut State);
    /// Is this a self transition which should trigger the state's exit and entry actions?
    fn should_trigger_state_actions() -> bool;

    fn execute_action<Q: FsmEventQueue<F>, I >(frontend: &mut FsmFrontend<F, Q, I>, event: &E, region: FsmRegionId)
        where <F as FsmBackend>::States: AsMut<State>, I: Inspect<F>
    {
        let mut event_context = EventContext {
            context: &mut frontend.backend.context,
            queue: &mut frontend.queue,
            region
        };

        let state: &mut State = frontend.backend.states.as_mut();

        Self::action(event, &mut event_context, state);
    }

    //fn execute_transition<Q: FsmEventQueue<F>, I>(frontend: &mut FsmFrontend<F, Q, I>, event: &FsmEvent<<F as FsmBackend>::Events>, region: FsmRegionId)
    fn execute_transition<Q: FsmEventQueue<F>, I>(frontend: &mut FsmFrontend<F, Q, I>, event: &E, region: FsmRegionId)
        where I: Inspect<F> {
            todo!("act")
        }
}

/*
pub trait FsmTransitionExecutor<F>
    where F: FsmBackend
{
    fn execute_transition<Q: FsmEventQueue<F>, I>(frontend: &mut FsmFrontend<F, Q, I>, event: &<F as FsmBackend>::Events, region: FsmRegionId)
        where I: Inspect<F>;
}
*/