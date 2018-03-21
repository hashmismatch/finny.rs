use fsm::*;
use std::sync::*;

use std::thread;
use std::sync::mpsc::*;
use std::collections::HashMap;

use server_ws::*;

#[derive(Clone)]
pub struct FsmInspectWebServer {
    pub inner: InspectShared
}

#[derive(Clone)]
pub struct InspectShared {
    pub inner: Arc<Mutex<Inspect>>
}

impl InspectShared {
    pub fn add_ws_client(&self, sender: Sender<WsMessage>) {
        println!("new ws client!");

        if let Ok(mut inspect) = self.inner.lock() {
            inspect.ws_clients.push(sender);
        }
    }
}

impl FsmInspectWebServer {
    /*
    fn init<F: Fsm>(&self, fsm: &F) {
        println!("init?");
        if let Ok(mut inner) = self.inner.inner.lock() {
            let fsm_info = ::data::FsmInfo {
                regions: F::fsm_info_regions(),
                name: F::fsm_name()
            };
            inner.machines.push(fsm_info);

            println!("stored data for type {}", stringify!(F));
        }
    }
    */

    pub fn new<F: Fsm>() -> Result<Self, ()> {

        let mut inspect = Inspect::new();
        let fsm_info = ::data::FsmInfo {
            regions: F::fsm_info_regions(),
            name: F::fsm_name()
        };
        inspect.machine_infos.push(fsm_info);

        let fsm_data_state = ::data::FsmDataState {
            name: F::fsm_name(),
            structures: HashMap::new()
        };
        inspect.machine_state.push(fsm_data_state);
        
        let inner = InspectShared { inner: Arc::new(Mutex::new(inspect)) };
        
        {            
            let inner = inner.clone();
            thread::spawn(move || {
                ::server_web::spawn_web_server(inner);
            });            
        }

        {
            let inner = inner.clone();
            thread::spawn(move || {
                ::server_ws::spawn_ws(inner);
            });
        }

        Ok(FsmInspectWebServer {
            inner: inner
        })
    }

    fn push_event<F: Fsm>(&self, event: FsmInspectedEvent, structures: Vec<FsmEventStruct>) {
        if let Ok(mut inspect) = self.inner.inner.lock() {
            
            let mut modified_structures = vec![];
            if let Some(ref mut fsm_info) = inspect.machine_state.iter_mut().find(|i| i.name == F::fsm_name()) {
                for s in structures {
                    if s.value == json!(null) { continue; }

                    if let Some(mut stored) = fsm_info.structures.get_mut(&s.id) {
                        if *stored != s.value {
                            *stored = s.value.clone();
                            modified_structures.push(s);
                        }
                        // rust's borrow checker snafu
                        continue;
                    }
                    
                    {
                        fsm_info.structures.insert(s.id.clone(), s.value.clone());
                        modified_structures.push(s);
                    }
                }
            }            

            let common = FsmEventCommon {
                id: inspect.get_next_id(),
                fsm: F::fsm_name().into(),
                modified_structures: modified_structures
            };

            // todo: bug with missing_clients!
            let mut missing_clients = vec![];
            for (idx, client) in inspect.ws_clients.iter().enumerate() {
                let msg = MessageToBrowserClient {
                    common: common.clone(),
                    event: event.clone()
                };

                let msg = WsMessage::FsmData(msg);

                match client.send(msg) {
                    Ok(_) => (),
                    Err(_) => {
                        missing_clients.push(idx);
                    }
                }
            }

            for idx in missing_clients {
                inspect.ws_clients.remove(idx);
            }
        }
    }
}


impl<F: Fsm> FsmInspect<F> for FsmInspectWebServer
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
}

#[derive(Serialize, Debug, Clone)]
pub enum FsmInspectedEvent {
    StateTransitionStart(FsmEventStateTransitionStart),
    StateTransitioned(FsmEventStateTransitioned),    
    ProcessingEvent(FsmEventProcessingEvent),
    StateEvent(FsmEventState),
    Action(FsmEventAction)
}

#[derive(Serialize, Debug, Clone)]
pub struct MessageToBrowserClient {
    pub common: FsmEventCommon,
    pub event: FsmInspectedEvent
}

impl MessageToBrowserClient {
    pub fn to_ws_json(&self) -> String {
        ::serde_json::to_string(&self).unwrap()
    }
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

pub struct Inspect {
    pub machine_infos: Vec<::data::FsmInfo>,
    pub machine_state: Vec<::data::FsmDataState>,
    //pub events: Vec<FsmEvent>,
    pub next_id: u64,
    pub ws_clients: Vec<Sender<WsMessage>>
}

impl Inspect {
    pub fn new() -> Self {
        Inspect {
            machine_infos: vec![],
            machine_state: vec![],
            next_id: 0,
            ws_clients: vec![]
        }
    }

    pub fn get_next_id(&mut self) -> u64 {
        let n = self.next_id;
        self.next_id += 1;
        n
    }
}

