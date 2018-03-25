use super::*;


pub trait FsmInspectClone {
	fn add_fsm<FSub: Fsm>(&self) -> Self;
}

pub trait FsmInspect<F: Fsm>: FsmInspectClone
	where F::C : ::serde::Serialize + ::std::fmt::Debug
{
	fn on_process_event<Ev: FsmEvent<F> + ::serde::Serialize + ::std::fmt::Debug>(&self, state: &F::CS, event_kind: F::EventKind, event: &Ev) { }

	/* the approximate order in which these methods get called */
	
	fn on_transition_start(&self, transition_id: TransitionId, source_state: &F::RegionState, target_state: &F::RegionState, event_context: &EventContext<F>) { }
	fn on_state_exit<S: FsmState<F> + ::serde::Serialize + ::std::fmt::Debug>(&self, transition_id: TransitionId, region_state: &F::RegionState, state: &S, event_context: &EventContext<F>) { }
	
	fn on_action<Ss, St>(&self, transition_id: TransitionId, action: &'static str, source_state_kind: &F::RegionState, source_state: &Ss, target_state_kind: &F::RegionState, target_state: &St, event_context: &EventContext<F>)
		where Ss: FsmState<F> + ::serde::Serialize + ::std::fmt::Debug, St: FsmState<F> + ::serde::Serialize + ::std::fmt::Debug
		{ }

	fn on_state_entry<S: FsmState<F> + ::serde::Serialize + ::std::fmt::Debug>(&self, transition_id: TransitionId, region_state: &F::RegionState, state: &S, event_context: &EventContext<F>) { }
	fn on_transition_finish(&self, transition_id: TransitionId, source_state: &F::RegionState, target_state: &F::RegionState, event_context: &EventContext<F>) { }
	
	fn on_no_transition(&self, state: &F::CS) { }
	fn on_event_processed(&self) { }
}

#[derive(Default, Clone, Copy)]
pub struct FsmInspectNull;
impl<F: Fsm> FsmInspect<F> for FsmInspectNull where F::C : ::serde::Serialize + ::std::fmt::Debug { }
impl FsmInspectClone for FsmInspectNull {
	fn add_fsm<FSub: Fsm>(&self) -> Self {
		FsmInspectNull
	}
}
