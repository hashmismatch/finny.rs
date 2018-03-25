use prelude::v1::*;
use fsm::*;
use fsm::info::*;
use fsm::inspect::*;

use inspect_data::*;

#[derive(Clone)]
pub struct FsmInspectStdOut {
    shared: Arc<Mutex<InspectStateShared>>
}

use std::collections::HashMap;

impl FsmInspectStdOut {
    pub fn new<F: Fsm>() -> Self {
        let shared = InspectStateShared {
            data: vec![
                FsmDataState::new::<F>()
            ]
        };

        FsmInspectStdOut {
            shared: Arc::new(Mutex::new(shared))
        }
    }

    pub fn flush(&self) {
        if let Ok(shared) = self.shared.lock() {
            let max_fsm_name_len = shared.data.iter().map(|f| f.fsm_name.len()).max().unwrap_or(10);

            for fsm in &shared.data {
                let events = fsm.flush_events();

                for event in events {
                    match event.event {
                        FsmInspectedEvent::EventProcessed => {
                            println!("");
                        },
                        _ => {
                            println!("{:width$}  {}",
                                event.common.fsm,
                                FsmInspectedEventConsole(&event.event),
                                width = max_fsm_name_len
                                );

                            match event.event {
                                FsmInspectedEvent::ProcessingEvent(ref p) => {
                                    if p.event_data != json!(null) {
                                        println!("{:width$} {}", 
                                            "",
                                            p.event_data,
                                            width = max_fsm_name_len + 5
                                            );
                                    }
                                },
                                _ => ()
                            }

                            // changed structures
                            for structure in &event.common.modified_structures {
                                println!("{:width$}|   {:?}",
                                    "",
                                    structure.id,
                                    width = max_fsm_name_len + 4
                                );

                                {
                                    let mut diff = ::json_diff::JsonDiff::new(&structure.old_value, &structure.value);
                                    diff.padding = format!("{}|   ", " ".repeat(max_fsm_name_len + 4)).into();
                                    diff.mode = ::json_diff::JsonDiffMode::DiffOnly;

                                    print!("{}", diff);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}


pub struct FsmInspectedEventConsole<'a>(&'a FsmInspectedEvent);

impl<'a> ::std::fmt::Display for FsmInspectedEventConsole<'a> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), ::std::fmt::Error> {
        match *self.0 {
            FsmInspectedEvent::ProcessingEvent(ref p) => {
                write!(f, "* Processing event {}", p.event_kind)
            }
            FsmInspectedEvent::StateTransitionStart(ref t) => {
                write!(f, "  / Transitioning from {} to {}", t.from, t.to)
            },
            FsmInspectedEvent::StateEvent(ref s) => {
                match s.state_event_kind {
                    FsmEventStateKind::Enter => {
                        write!(f, "  | Entering state {}", s.state_name)
                    },
                    FsmEventStateKind::Exit => {
                        write!(f, "  | Exiting state {}", s.state_name)
                    }
                }                
            },
            FsmInspectedEvent::Action(ref a) => {
                write!(f, "  | Performing action {}", a.action_name)
            }
            FsmInspectedEvent::StateTransitioned(ref t) => {
                write!(f, "  \\ Transition finished.")
            },
            FsmInspectedEvent::EventProcessed => {
                write!(f, "Event processed.")
            },
            FsmInspectedEvent::NoTransition(ref n) => {
                write!(f, "No transition found. Current state: {}", n.current_state)
            }
        }
    }
}


struct InspectStateShared {
    data: Vec<FsmDataState>
}

impl FsmInspectClone for FsmInspectStdOut {
	fn add_fsm<FSub: Fsm>(&self) -> Self {
        let shared = self.shared.clone();
        if let Ok(ref mut shared) = shared.lock() {
            shared.data.push(FsmDataState::new::<FSub>());
        }

        FsmInspectStdOut {
            shared: shared
        }
	}
}

impl<F: Fsm> FsmInspect<F> for FsmInspectStdOut where F::C : ::serde::Serialize + ::std::fmt::Debug {
    fn on_process_event<Ev: FsmEvent<F> + ::serde::Serialize + ::std::fmt::Debug>(&self, state: &F::CS, event_kind: F::EventKind, event: &Ev) {
        if let Ok(shared) = self.shared.lock() {
            for fsm in &shared.data {
                fsm.on_process_event(state, event_kind, event);
            }
        }
    }

	/* the approximate order in which these methods get called */
	
	fn on_transition_start(&self, transition_id: TransitionId, source_state: &F::RegionState, target_state: &F::RegionState, event_context: &EventContext<F>) {
        if let Ok(shared) = self.shared.lock() {
            for fsm in &shared.data {
                fsm.on_transition_start(transition_id, source_state, target_state, event_context);
            }
        }
    }

	fn on_state_exit<S: FsmState<F> + ::serde::Serialize + ::std::fmt::Debug>(&self, transition_id: TransitionId, region_state: &F::RegionState, state: &S, event_context: &EventContext<F>) {
        if let Ok(shared) = self.shared.lock() {
            for fsm in &shared.data {
                fsm.on_state_exit(transition_id, region_state, state, event_context);
            }
        }
    }
	
	fn on_action<Ss, St>(&self, transition_id: TransitionId, action: &'static str, source_state_kind: &F::RegionState, source_state: &Ss, target_state_kind: &F::RegionState, target_state: &St, event_context: &EventContext<F>)
		where Ss: FsmState<F> + ::serde::Serialize + ::std::fmt::Debug, St: FsmState<F> + ::serde::Serialize + ::std::fmt::Debug
	{
        if let Ok(shared) = self.shared.lock() {
            for fsm in &shared.data {
                fsm.on_action(transition_id, action, source_state_kind, source_state, target_state_kind, target_state, event_context);
            }
        }
    }

	fn on_state_entry<S: FsmState<F> + ::serde::Serialize + ::std::fmt::Debug>(&self, transition_id: TransitionId, region_state: &F::RegionState, state: &S, event_context: &EventContext<F>) {
        if let Ok(shared) = self.shared.lock() {
            for fsm in &shared.data {
                fsm.on_state_entry(transition_id, region_state, state, event_context);
            }
        }
    }

	fn on_transition_finish(&self, transition_id: TransitionId, source_state: &F::RegionState, target_state: &F::RegionState, event_context: &EventContext<F>) {
        if let Ok(shared) = self.shared.lock() {
            for fsm in &shared.data {
                fsm.on_transition_finish(transition_id, source_state, target_state, event_context);
            }
        }
    }

	fn on_no_transition(&self, state: &F::CS) {
        if let Ok(shared) = self.shared.lock() {
            for fsm in &shared.data {
                FsmInspect::<F>::on_no_transition(fsm, state);
            }
        }
    }

	fn on_event_processed(&self) {
        if let Ok(shared) = self.shared.lock() {
            for fsm in &shared.data {
                FsmInspect::<F>::on_event_processed(fsm);
            }
        }

        self.flush();
    }    
}
