//! Combine two FSM inspectors into one.

use prelude::v1::*;
use fsm::*;
use fsm::info::*;
use fsm::inspect::*;

pub struct FsmInspectCombinator<F, InspectA, InspectB>
    where F: Fsm,
          F::C : ::serde::Serialize + ::std::fmt::Debug,
          InspectA: FsmInspect<F>,
          InspectB: FsmInspect<F>
{
    fsm: PhantomData<F>,
    inspect_a: InspectA,
    inspect_b: InspectB
}



/*
impl<F, InspectA, InspectB> Clone for FsmInspectCombinator<F, InspectA, InspectB>
    where F: Fsm,
          F::C : ::serde::Serialize + ::std::fmt::Debug,
          InspectA: FsmInspect<F>,
          InspectB: FsmInspect<F>
{
    fn clone(&self) -> Self {
        FsmInspectCombinator {
            fsm: PhantomData::default(),
            inspect_a: self.inspect_a.clone(),
            inspect_b: self.inspect_b.clone()
        }
    }
}
*/

impl<F, InspectA, InspectB> FsmInspectCombinator<F, InspectA, InspectB>
    where F: Fsm,
          F::C : ::serde::Serialize + ::std::fmt::Debug,
          InspectA: FsmInspect<F>,
          InspectB: FsmInspect<F>
{
    pub fn new(inspect_a: InspectA, inspect_b: InspectB) -> Self {
        FsmInspectCombinator {
            fsm: PhantomData::default(),
            inspect_a: inspect_a,
            inspect_b: inspect_b
        }
    }

    pub fn get_inspect_a(&self) -> &InspectA {
        &self.inspect_a
    }

    pub fn get_inspect_a_mut(&mut self) -> &mut InspectA {
        &mut self.inspect_a
    }

    pub fn get_inspect_b(&self) -> &InspectB {
        &self.inspect_b
    }

    pub fn get_inspect_b_mut(&mut self) -> &mut InspectB {
        &mut self.inspect_b
    }
}

impl<F, InspectA, InspectB> FsmInspectClone for FsmInspectCombinator<F, InspectA, InspectB>
    where F: Fsm,
          F::C : ::serde::Serialize + ::std::fmt::Debug,
          InspectA: FsmInspect<F>,
          InspectB: FsmInspect<F>
{
    fn add_fsm<FSub: Fsm>(&self) -> Self {
        FsmInspectCombinator {
            fsm: PhantomData::default(),
            inspect_a: self.inspect_a.add_fsm::<FSub>(),
            inspect_b: self.inspect_b.add_fsm::<FSub>()
        }
    }
}

impl<F, InspectA, InspectB> FsmInspect<F> for FsmInspectCombinator<F, InspectA, InspectB>
    where F: Fsm,
          F::C : ::serde::Serialize + ::std::fmt::Debug,
          InspectA: FsmInspect<F>,
          InspectB: FsmInspect<F>
{
	fn on_process_event<Ev: FsmEvent<F> + ::serde::Serialize + ::std::fmt::Debug>(&self, state: &F::CS, event_kind: F::EventKind, event: &Ev) {
        self.inspect_a.on_process_event(state, event_kind.clone(), event);
        self.inspect_b.on_process_event(state, event_kind.clone(), event);
    }
    
	fn on_transition_start(&self, transition_id: TransitionId, source_state: &F::RegionState, target_state: &F::RegionState, event_context: &EventContext<F>) {
        self.inspect_a.on_transition_start(transition_id, source_state, target_state, event_context);
        self.inspect_b.on_transition_start(transition_id, source_state, target_state, event_context);
    }

	fn on_state_exit<S: FsmState<F> + ::serde::Serialize + ::std::fmt::Debug>(&self, transition_id: TransitionId, region_state: &F::RegionState, state: &S, event_context: &EventContext<F>) {
        self.inspect_a.on_state_exit(transition_id, region_state, state, event_context);
        self.inspect_b.on_state_exit(transition_id, region_state, state, event_context);
    }
	
	fn on_action<Ss, St>(&self, transition_id: TransitionId, action: &'static str, source_state_kind: &F::RegionState, source_state: &Ss, target_state_kind: &F::RegionState, target_state: &St, event_context: &EventContext<F>)
		where Ss: FsmState<F> + ::serde::Serialize + ::std::fmt::Debug, St: FsmState<F> + ::serde::Serialize + ::std::fmt::Debug
    {
        self.inspect_a.on_action(transition_id, action, source_state_kind, source_state, target_state_kind, target_state, event_context);
        self.inspect_b.on_action(transition_id, action, source_state_kind, source_state, target_state_kind, target_state, event_context);
    }

	fn on_state_entry<S: FsmState<F> + ::serde::Serialize + ::std::fmt::Debug>(&self, transition_id: TransitionId, region_state: &F::RegionState, state: &S, event_context: &EventContext<F>) {
        self.inspect_a.on_state_entry(transition_id, region_state, state, event_context);
        self.inspect_b.on_state_entry(transition_id, region_state, state, event_context);
    }
	
    fn on_transition_finish(&self, transition_id: TransitionId, source_state: &F::RegionState, target_state: &F::RegionState, event_context: &EventContext<F>) {
        self.inspect_a.on_transition_finish(transition_id, source_state, target_state, event_context);
        self.inspect_b.on_transition_finish(transition_id, source_state, target_state, event_context);
    }
}
