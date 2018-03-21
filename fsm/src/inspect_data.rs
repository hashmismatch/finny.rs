use prelude::v1::*;

use machine::*;

use std::collections::HashMap;

#[derive(Serialize, Clone)]
pub struct FsmInfo {
    pub regions: Vec<FsmInfoRegion>,
    pub name: &'static str
}

#[derive(Serialize, Debug, Clone)]
pub enum FsmInspectedEvent {
    StateTransitionStart(FsmEventStateTransitionStart),
    StateTransitioned(FsmEventStateTransitioned),    
    ProcessingEvent(FsmEventProcessingEvent),
    StateEvent(FsmEventState),
    Action(FsmEventAction),
    EventProcessed,
    NoTransition
}


#[derive(Serialize, Debug, Clone)]
pub struct FsmEventCommon {
    pub id: u64,
    pub fsm: String,
    pub modified_structures: Vec<FsmEventStruct>
}

#[derive(Serialize, Debug, Clone)]
pub struct FsmEventStateTransitionStart {
    pub transition_id: TransitionId,
    pub from: String,
    pub to: String,
    pub region_id: RegionId
}

#[derive(Serialize, Debug, Clone)]
pub struct FsmEventStateTransitioned {
    pub transition_id: TransitionId,
    pub from: String,
    pub to: String,
    pub region_id: RegionId
}

#[derive(Serialize, Debug, Clone)]
pub struct FsmEventProcessingEvent {
    pub event_kind: String,
    pub event_data: ::serde_json::Value
}

#[derive(Serialize, Debug, Clone)]
pub enum FsmEventStateKind {
    Enter,
    Exit
}

#[derive(Serialize, Debug, Clone)]
pub struct FsmEventState {
    pub transition_id: TransitionId,
    pub state_event_kind: FsmEventStateKind,
    pub state_name: String
}

#[derive(Serialize, Debug, Clone)]
pub struct FsmEventAction {
    pub transition_id: TransitionId,
    pub action_name: String
}

#[derive(Serialize, Debug, Clone)]
pub struct FsmEventStruct {
    pub id: FsmEventStructKind,
    pub value: ::serde_json::Value
}

#[derive(Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum FsmEventStructKind {
    Context,
    State(String)
}




#[derive(Clone)]
pub struct FsmEventEntry {
    pub common: FsmEventCommon,
    pub event: FsmInspectedEvent
}

#[derive(Clone)]
pub struct FsmDataState {
    pub fsm_name: &'static str,
    inner: Arc<Mutex<FsmDataStateInner>>
}

struct FsmDataStateInner {    
    structures: HashMap<FsmEventStructKind, ::serde_json::Value>,
    events: Vec<FsmEventEntry>
}


impl FsmDataState {
    pub fn new<F: Fsm>() -> Self {
        let inner = FsmDataStateInner {            
            structures: HashMap::new(),
            events: vec![]
        };

        FsmDataState {
            fsm_name: F::fsm_name(),
            inner: Arc::new(Mutex::new(inner))
        }
    }

    pub fn flush_events(&self) -> Vec<FsmEventEntry> {
        use std::mem;
        let mut ret = vec![];
        if let Ok(ref mut inner) = self.inner.lock() {            
            mem::swap(&mut ret, &mut inner.events);            
        }
        ret
    }

    fn push_event<F: Fsm>(&self, event: FsmInspectedEvent, structures: Vec<FsmEventStruct>) {
        if let Ok(ref mut inner) = self.inner.lock() {
            if self.fsm_name != F::fsm_name() { return; }

            let mut modified_structures = vec![];    
            for s in structures {
                if s.value == json!(null) { continue; }

                if let Some(mut stored) = inner.structures.get_mut(&s.id) {
                    if *stored != s.value {
                        *stored = s.value.clone();
                        modified_structures.push(s);
                    }
                    // rust's borrow checker snafu
                    continue;
                }
                
                {
                    inner.structures.insert(s.id.clone(), s.value.clone());
                    modified_structures.push(s);
                }
            }

            let common = FsmEventCommon {
                id: 0,
                fsm: F::fsm_name().into(),
                modified_structures: modified_structures
            };

            inner.events.push(FsmEventEntry {
                common: common,
                event: event
            });
        }
    }
}

impl FsmInspectClone for FsmDataState {
	fn add_fsm<FSub: Fsm>(&self) -> Self {
		FsmDataState::new::<FSub>()
	}
}

impl<F: Fsm> FsmInspect<F> for FsmDataState
    where F::C : ::serde::Serialize + ::std::fmt::Debug
{
    fn on_process_event<Ev: FsmEvent<F> + ::serde::Serialize + ::std::fmt::Debug>(&self, state: &F::CS, event_kind: F::EventKind, event: &Ev) {
        let ev = FsmEventProcessingEvent {
            event_kind: format!("{:?}", event_kind),
            event_data: ::serde_json::to_value(&event).unwrap_or(json!(null))
        };

        self.push_event::<F>(FsmInspectedEvent::ProcessingEvent(ev), vec![]);
    }

	fn on_state_entry<S: FsmState<F> + ::serde::Serialize + ::std::fmt::Debug>(&self, transition_id: TransitionId, region_state: &F::RegionState, state: &S, event_context: &EventContext<F>) {
        let ev = FsmEventState {
            transition_id: transition_id,
            state_event_kind: FsmEventStateKind::Enter,
            state_name: format!("{:?}", region_state)
        };

        let structs = vec![
            FsmEventStruct {
                id: FsmEventStructKind::Context,
                value: ::serde_json::to_value(&event_context.context).unwrap_or(json!(null))
            },
            FsmEventStruct {
                id: FsmEventStructKind::State(ev.state_name.clone()),
                value: ::serde_json::to_value(&state).unwrap_or(json!(null))
            }
        ];

        self.push_event::<F>(FsmInspectedEvent::StateEvent(ev), structs)
    }

	fn on_state_exit<S: FsmState<F> + ::serde::Serialize + ::std::fmt::Debug>(&self, transition_id: TransitionId, region_state: &F::RegionState, state: &S, event_context: &EventContext<F>) {
        let ev = FsmEventState {
            transition_id: transition_id,
            state_event_kind: FsmEventStateKind::Exit,
            state_name: format!("{:?}", region_state)
        };

        let structs = vec![
            FsmEventStruct {
                id: FsmEventStructKind::Context,
                value: ::serde_json::to_value(&event_context.context).unwrap_or(json!(null))
            },
            FsmEventStruct {
                id: FsmEventStructKind::State(ev.state_name.clone()),
                value: ::serde_json::to_value(&state).unwrap_or(json!(null))
            }
        ];

        self.push_event::<F>(FsmInspectedEvent::StateEvent(ev), structs)
    }

    fn on_action<Ss, St>(&self, transition_id: TransitionId, action: &'static str, source_state_kind: &F::RegionState, source_state: &Ss, target_state_kind: &F::RegionState, target_state: &St, event_context: &EventContext<F>)
		where Ss: FsmState<F> + ::serde::Serialize + ::std::fmt::Debug, St: FsmState<F> + ::serde::Serialize + ::std::fmt::Debug
	{
        let ev = FsmEventAction {
            transition_id: transition_id,
            action_name: action.into()
        };

        let mut structs = vec![
            FsmEventStruct {
                id: FsmEventStructKind::Context,
                value: ::serde_json::to_value(&event_context.context).unwrap_or(json!(null))
            },
            FsmEventStruct {
                id: FsmEventStructKind::State(format!("{:?}", source_state_kind)),
                value: ::serde_json::to_value(&source_state).unwrap_or(json!(null))
            }
        ];

        if source_state_kind != target_state_kind {
            structs.push(FsmEventStruct {
                id: FsmEventStructKind::State(format!("{:?}", target_state_kind)),
                value: ::serde_json::to_value(&target_state).unwrap_or(json!(null))
            });
        }

        self.push_event::<F>(FsmInspectedEvent::Action(ev), structs)
    }
    
	fn on_transition_start(&self, transition_id: TransitionId, source_region_state: &F::RegionState, target_region_state: &F::RegionState, event_context: &EventContext<F>) {
        let ev = FsmEventStateTransitionStart {
            transition_id: transition_id,
            from: format!("{:?}", source_region_state),
            to: format!("{:?}", target_region_state),
            region_id: event_context.region
        };

        let structs = vec![
            FsmEventStruct {
                id: FsmEventStructKind::Context,
                value: ::serde_json::to_value(&event_context.context).unwrap_or(json!(null))
            }
        ];

        self.push_event::<F>(FsmInspectedEvent::StateTransitionStart(ev), structs);
    }    

	fn on_transition_finish(&self, transition_id: TransitionId, source_region_state: &F::RegionState, target_region_state: &F::RegionState, event_context: &EventContext<F>) {
        let ev = FsmEventStateTransitioned {
            transition_id: transition_id,
            from: format!("{:?}", source_region_state),
            to: format!("{:?}", target_region_state),
            region_id: event_context.region
        };

        let structs = vec![
            FsmEventStruct {
                id: FsmEventStructKind::Context,
                value: ::serde_json::to_value(&event_context.context).unwrap_or(json!(null))
            }
        ];

        self.push_event::<F>(FsmInspectedEvent::StateTransitioned(ev), structs);
    }

	fn on_no_transition(&self) {
        self.push_event::<F>(FsmInspectedEvent::NoTransition, vec![]);
    }

	fn on_event_processed(&self) {
        self.push_event::<F>(FsmInspectedEvent::EventProcessed, vec![]);
    }
}