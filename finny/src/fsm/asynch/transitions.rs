//! The async counterparts of the state and transition traits. Implemented by the code generator.

use crate::lib::*;
use crate::{EventContext, FsmBackend, FsmBackendImpl, FsmCurrentState, FsmDispatchResult, FsmEvent, FsmEventQueue,
    FsmStateTransitionAsMut, FsmStates, FsmTimers, Inspect, InspectFsmEvent};

use super::RegionContext;

/// A state's async entry and exit actions.
#[allow(async_fn_in_trait)]
pub trait FsmStateAsync<F: FsmBackend> where Self: Sized {
    /// Action that is executed whenever this state is being entered.
    async fn on_entry<'a, Q: FsmEventQueue<F>>(&mut self, context: &mut EventContext<'a, F, Q>);
    /// Action that is executed whenever this state is being exited.
    async fn on_exit<'a, Q: FsmEventQueue<F>>(&mut self, context: &mut EventContext<'a, F, Q>);

    fn fsm_state() -> <<F as FsmBackend>::States as FsmStates<F>>::StateKind;

    async fn execute_on_entry<S, Q, T, I>(rc: &mut RegionContext<'_, F, S, Q, T>, inspect: &I)
        where S: AsMut<Self>, Q: FsmEventQueue<F>, I: Inspect
    {
        inspect.on_state_enter::<Self>();
        inspect.on_event(&InspectFsmEvent::StateEnter(Self::fsm_state()));

        let mut event_context = EventContext {
            context: &mut *rc.context,
            queue: &mut *rc.queue,
            region: rc.region
        };
        let state: &mut Self = rc.states.as_mut();
        state.on_entry(&mut event_context).await;
    }

    async fn execute_on_exit<S, Q, T, I>(rc: &mut RegionContext<'_, F, S, Q, T>, inspect: &I)
        where S: AsMut<Self>, Q: FsmEventQueue<F>, I: Inspect
    {
        {
            let mut event_context = EventContext {
                context: &mut *rc.context,
                queue: &mut *rc.queue,
                region: rc.region
            };
            let state: &mut Self = rc.states.as_mut();
            state.on_exit(&mut event_context).await;
        }

        inspect.on_state_exit::<Self>();
        inspect.on_event(&InspectFsmEvent::StateExit(Self::fsm_state()));
    }
}

/// The transition that starts the machine, triggered using the `start()` method.
#[allow(async_fn_in_trait)]
pub trait FsmTransitionFsmStartAsync<F: FsmBackend, TInitialState> {
    async fn execute_transition<S, Q, T, I>(rc: &mut RegionContext<'_, F, S, Q, T>,
        _fsm_event: &FsmEvent<<F as FsmBackend>::Events, <F as FsmBackend>::Timers>,
        inspect: &I)
        where
            TInitialState: FsmStateAsync<F>,
            S: AsMut<TInitialState>,
            Q: FsmEventQueue<F>,
            I: Inspect,
            Self: Sized
    {
        let inspect_ctx = inspect.for_transition::<Self>();

        <TInitialState>::execute_on_entry(rc, &inspect_ctx).await;

        *rc.current_state = FsmCurrentState::State(<TInitialState>::fsm_state());
    }
}

/// A transition's async action that operates on both the exit and entry states.
#[allow(async_fn_in_trait)]
pub trait FsmTransitionActionAsync<F: FsmBackend, E, TStateFrom, TStateTo> {
    /// This action is executed after the first state's exit event, and just before the second event's entry action. It can mutate both states.
    async fn action<'a, Q: FsmEventQueue<F>>(event: &E, context: &mut EventContext<'a, F, Q>, from: &mut TStateFrom, to: &mut TStateTo);

    async fn execute_transition<S, Q, T, I>(rc: &mut RegionContext<'_, F, S, Q, T>, event: &E, inspect: &I)
        where
            S: FsmStateTransitionAsMut<TStateFrom, TStateTo> + AsMut<TStateFrom> + AsMut<TStateTo>,
            TStateFrom: FsmStateAsync<F>,
            TStateTo: FsmStateAsync<F>,
            Q: FsmEventQueue<F>,
            I: Inspect,
            Self: Sized
    {
        let inspect_ctx = inspect.for_transition::<Self>();

        <TStateFrom>::execute_on_exit(rc, &inspect_ctx).await;

        // transition action
        {
            inspect_ctx.on_action::<Self>();

            let mut event_context = EventContext {
                context: &mut *rc.context,
                queue: &mut *rc.queue,
                region: rc.region
            };
            let (from, to) = FsmStateTransitionAsMut::<TStateFrom, TStateTo>::as_state_transition_mut(&mut *rc.states);
            Self::action(event, &mut event_context, from, to).await;
        }

        <TStateTo>::execute_on_entry(rc, &inspect_ctx).await;

        *rc.current_state = FsmCurrentState::State(<TStateTo>::fsm_state());
    }
}

/// An internal or self transition's async action, it can only mutate its own state.
#[allow(async_fn_in_trait)]
pub trait FsmActionAsync<F: FsmBackend, E, State> {
    /// This action is executed as part of an internal or self transition.
    async fn action<'a, Q: FsmEventQueue<F>>(event: &E, context: &mut EventContext<'a, F, Q>, state: &mut State);
    /// Is this a self transition which should trigger the state's exit and entry actions?
    fn should_trigger_state_actions() -> bool;

    async fn execute_transition<S, Q, T, I>(rc: &mut RegionContext<'_, F, S, Q, T>, event: &E, inspect: &I)
        where
            S: AsMut<State>,
            State: FsmStateAsync<F>,
            Q: FsmEventQueue<F>,
            I: Inspect,
            Self: Sized
    {
        let inspect_ctx = inspect.for_transition::<Self>();

        if Self::should_trigger_state_actions() {
            <State>::execute_on_exit(rc, &inspect_ctx).await;
        }

        {
            inspect_ctx.on_action::<Self>();

            let mut event_context = EventContext {
                context: &mut *rc.context,
                queue: &mut *rc.queue,
                region: rc.region
            };
            let state: &mut State = rc.states.as_mut();
            Self::action(event, &mut event_context, state).await;
        }

        if Self::should_trigger_state_actions() {
            <State>::execute_on_entry(rc, &inspect_ctx).await;
        }
    }
}
