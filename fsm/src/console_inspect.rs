use prelude::v1::*;
use machine::*;

#[derive(Copy, Clone, Debug)]
pub struct FsmInspectStdOut;

impl<F: Fsm> FsmInspect<F> for FsmInspectStdOut where F::C : ::serde::Serialize + ::std::fmt::Debug {
    /*
	fn on_state_entry(&self, state: &F::RegionState, event_context: &EventContext<F>) {
        println!("Entering state: {:?}", state);
    }

	fn on_state_exit(&self, state: &F::RegionState, event_context: &EventContext<F>) {
        println!("Exiting state: {:?}", state);
    }

	fn on_action(&self, action: &'static str, event_context: &EventContext<F>) {
        println!("Performed action {}.", action);
    }

	fn on_transition(&self, source_state: &F::RegionState, target_state: &F::RegionState, event_context: &EventContext<F>) {
        println!("Transitioned from {:?} to {:?}", source_state, target_state);
    }
    */

    /*
	fn on_no_transition<F: Fsm>(&self, current_state: &F::CS, event_context: &EventContext<F>) {
        println!("No transition found! Current state: {:?}", event_context.current_state);
    }
    */
}
