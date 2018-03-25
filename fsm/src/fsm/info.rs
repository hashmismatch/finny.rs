#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum TransitionId {
	Start,
	Table(u32),
	Stop
}



pub trait FsmInfo {
	fn fsm_info_regions() -> Vec<FsmInfoRegion>;
	fn fsm_name() -> &'static str;
}

#[cfg_attr(feature="info_serializable", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
pub struct FsmInfoRegion {
	pub region_name: &'static str,
	// todo: should be a vector!
	pub initial_state: &'static str,
	pub states: Vec<FsmInfoState>,
	pub transitions: Vec<FsmInfoTransition>
}

#[cfg_attr(feature="info_serializable", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
pub struct FsmInfoState {
	pub state_name: &'static str,
	pub is_initial_state: bool,
	pub is_interrupt_state: bool
}

#[cfg_attr(feature="info_serializable", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
pub struct FsmInfoTransition {
	pub transition_id: TransitionId,

	pub state_from: &'static str,
	pub state_from_is_submachine: bool,
	pub state_to: &'static str,
	pub state_to_is_submachine: bool,
	
	pub event: &'static str,
	pub action: &'static str,
	pub guard: &'static str,
	
	pub is_shallow_history: bool,
    pub is_resume_event: bool,
    pub is_internal: bool,
    pub is_anonymous: bool,
	pub transition_type: FsmInfoTransitionType
}

#[cfg_attr(feature="info_serializable", derive(Serialize, Deserialize))]
#[derive(PartialEq, Copy, Clone, Debug)]
pub enum FsmInfoTransitionType {
    Normal,
    SelfTransition,
    Internal
}


