use prelude::v1::*;

//pub trait FsmStructSerialize : ::serde::Serialize { }
//pub trait FsmStructDeserialize<'de> : ::serde::Deserialize<'de> { }

/*
pub trait FsmStructSerialize {
    fn fsm_serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ::serde::Serializer
	{
		use serde::ser::Error;
		Err(S::Error::custom("not implemented"))
	}
}
*/

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum FsmError {
	NoTransition,
	Interrupted,
	TimersImplementationRequired,
	UnknownTimerId
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum FsmQueueStatus {
	Empty,
	MoreEventsQueued
}

pub trait FsmEvent {

}
pub trait FsmEvents: Debug {
	fn new_no_event() -> Self;
}
pub trait FsmEventsRef: Debug {

}

pub trait FsmState<F: Fsm> {	
	fn on_entry(&mut self, event_context: &mut EventContext<F>) { }
	fn on_exit(&mut self, event_context: &mut EventContext<F>) { }		
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum TransitionId {
	Start,
	Table(u32),
	Stop
}

pub trait FsmInspect<F: Fsm>: Clone
	where F::C : ::serde::Serialize + ::std::fmt::Debug
{
	fn on_process_event<Ev: FsmEvent + ::serde::Serialize + ::std::fmt::Debug>(&self, state: &F::CS, event_kind: F::EventKind, event: &Ev) { }

	/* the approximate order in which these methods get called */
	
	fn on_transition_start(&self, transition_id: TransitionId, source_state: &F::RegionState, target_state: &F::RegionState, event_context: &EventContext<F>) { }
	fn on_state_exit<S: FsmState<F> + ::serde::Serialize + ::std::fmt::Debug>(&self, transition_id: TransitionId, region_state: &F::RegionState, state: &S, event_context: &EventContext<F>) { }
	
	fn on_action<Ss, St>(&self, transition_id: TransitionId, action: &'static str, source_state_kind: &F::RegionState, source_state: &Ss, target_state_kind: &F::RegionState, target_state: &St, event_context: &EventContext<F>)
		where Ss: FsmState<F> + ::serde::Serialize + ::std::fmt::Debug, St: FsmState<F> + ::serde::Serialize + ::std::fmt::Debug
		{ }

	fn on_state_entry<S: FsmState<F> + ::serde::Serialize + ::std::fmt::Debug>(&self, transition_id: TransitionId, region_state: &F::RegionState, state: &S, event_context: &EventContext<F>) { }
	fn on_transition_finish(&self, transition_id: TransitionId, source_state: &F::RegionState, target_state: &F::RegionState, event_context: &EventContext<F>) { }
	
	//fn on_no_transition<F: Fsm>(&self, current_state: &F::CS, event_context: &EventContext<F>) { }
}

#[derive(Default, Clone, Copy)]
pub struct FsmInspectNull;
impl<F: Fsm> FsmInspect<F> for FsmInspectNull where F::C : ::serde::Serialize + ::std::fmt::Debug { }

/*
impl<F: Fsm> FsmInspectNull<F> {
	pub fn new(context: &F::C) -> Self {
		FsmInspectNull {
			_fsm_ty: PhantomData
		}
	}
}
*/

/*
impl<F: Fsm> FsmInspect<F> for FsmInspectNull<F> {
	fn on_state_entry(&self, state: &F::S, event_context: &EventContext<F>) { }
	fn on_state_exit(&self, state: &F::S, event_context: &EventContext<F>) { }
	fn on_action(&self, state: &F::S, event_context: &EventContext<F>) { }
	fn on_transition(&self, source_state: &F::S, target_state: &F::S, event_context: &EventContext<F>) { }
	fn on_no_transition(&self, source_state: &F::S, target_state: &F::S) { }
}
*/

// just for the InitialState definition type
impl<F, A, B> FsmState<F> for (A, B) where F: Fsm, A: FsmState<F>, B: FsmState<F> { }
impl<F, A, B, C> FsmState<F> for (A, B, C) where F: Fsm, A: FsmState<F>, B: FsmState<F>, C: FsmState<F> { }
impl<F, A, B, C, D> FsmState<F> for (A, B, C, D) where F: Fsm, A: FsmState<F>, B: FsmState<F>, C: FsmState<F>, D: FsmState<F> { }
impl<F, A, B, C, D, E> FsmState<F> for (A, B, C, D, E) where F: Fsm, A: FsmState<F>, B: FsmState<F>, C: FsmState<F>, D: FsmState<F>, E: FsmState<F> { }

/*
// prevent usage in production, satisfy the compiler
impl<A, B> FsmStateFactory for (A, B) {
	fn new_state<C>(parent_context: &C) -> Self {
		panic!("Not supported for tuple types, just as a helper!");
	}
}
*/


/*
pub trait FsmStateSubMachineTransition<F: Fsm> {
	fn on_entry_internal(&mut self) { }
	fn on_exit_internal(&mut self) { } 	
}
*/


pub trait FsmStateFactory {
	fn new_state<C>(parent_context: &C) -> Self;
}

impl<S: Default> FsmStateFactory for S {
	fn new_state<C>(parent_context: &C) -> Self {
		Default::default()
	}
}

pub trait FsmGuard<F: Fsm, E> {
	fn guard(event: &E, event_context: &EventContext<F>, states: &F::SS) -> bool;
}

pub struct NoGuard;
impl<F: Fsm, E> FsmGuard<F, E> for NoGuard {
	#[inline]
	fn guard(event: &E, event_context: &EventContext<F>, states: &F::SS) -> bool {
		true
	}
}

pub struct NegateGuard<G>(PhantomData<G>);
impl<F, E, G> FsmGuard<F, E> for NegateGuard<G> where G: FsmGuard<F, E>, F: Fsm {
	#[inline]
	fn guard(event: &E, event_context: &EventContext<F>, states: &F::SS) -> bool {
		!G::guard(event, event_context, states)
	}
}


pub trait FsmAction<F: Fsm, S, E, T> {
	fn action(event: &E, event_context: &mut EventContext<F>, source_state: &mut S, target_state: &mut T);
}

pub trait FsmActionSelf<F: Fsm, S, E> {
	fn action(event: &E, event_context: &mut EventContext<F>, state: &mut S);
}

#[derive(PartialEq, Copy, Clone, Debug, Serialize)]
pub struct NoEvent;
impl FsmEvent for NoEvent { }

pub struct NoAction;
impl<F: Fsm, S, E, T> FsmAction<F, S, E, T> for NoAction {
	#[inline]
	fn action(event: &E, event_context: &mut EventContext<F>, source_state: &mut S, target_state: &mut T) { }
}
impl<F: Fsm, S, E> FsmActionSelf<F, S, E> for NoAction {
	#[inline]
	fn action(event: &E, event_context: &mut EventContext<F>, state: &mut S) { }
}

pub type RegionId = usize;

pub struct EventContext<'a, F: Fsm + 'a> {
	pub queue: &'a mut FsmEventQueue<F>,
	pub context: &'a mut F::C,
	//pub current_state: F::CS,
	pub region: RegionId
}


pub trait FsmEventQueue<F: Fsm> {
	fn enqueue_event(&mut self, event: F::E) -> Result<(), FsmError>;
	fn dequeue_event(&mut self) -> Option<F::E>;
	fn len(&self) -> usize;
}

pub trait FsmRetrieveState<S> {
	fn get_state(&self) -> &S;
	fn get_state_mut(&mut self) -> &mut S;
}

pub struct FsmEventQueueVec<F: Fsm> {
	queue: Vec<F::E>
}

impl<F: Fsm> FsmEventQueueVec<F> {
	pub fn new() -> Self {
		FsmEventQueueVec {
			queue: Vec::new()
		}
	}
}

impl<F: Fsm> FsmEventQueue<F> for FsmEventQueueVec<F> {
	fn enqueue_event(&mut self, event: F::E) -> Result<(), FsmError> {
		self.queue.push(event);
		Ok(())
	}

	fn dequeue_event(&mut self) -> Option<F::E> {
		if self.queue.len() > 0 {
			Some(self.queue.remove(0))
		} else {
			None
		}
	}

	fn len(&self) -> usize {
		self.queue.len()
	}
}


pub trait Fsm: FsmInfo where Self: Sized {
	type E: FsmEvents;
	type EventKind: Debug + PartialEq;
	type S;
	type C;
	type CS: Debug;
	type RegionState: Debug + PartialEq;
	type SS;

	fn get_current_state(&self) -> Self::CS;
	fn get_states(&self) -> &Self::SS;
	fn get_states_mut(&mut self) -> &mut Self::SS;
}

pub trait FsmFrontend<F: Fsm> {
	fn start(&mut self);
	fn stop(&mut self);
		
	fn get_queue(&self) -> &FsmEventQueue<F>;
	fn get_queue_mut(&mut self) -> &mut FsmEventQueue<F>;

	fn process_anonymous_transitions(&mut self) -> Result<(), FsmError> {
		loop {
			match self.process_tagged_event(F::E::new_no_event()) {
				Ok(_) => { continue; }
				Err(_) => {
					break;
				}
			}
		}

		Ok(())
	}	

	fn process_tagged_event(&mut self, event: F::E) -> Result<(), FsmError>;
	
	fn execute_queued_events(&mut self) -> FsmQueueStatus {
		if self.get_queue().len() == 0 { return FsmQueueStatus::Empty; }

		loop {
			let l = self.execute_single_queued_event();
			if l == FsmQueueStatus::Empty { break; }
		}

		FsmQueueStatus::Empty
	}

	fn execute_single_queued_event(&mut self) -> FsmQueueStatus {
		if let Some(ev) = self.get_queue_mut().dequeue_event() {
			self.process_tagged_event(ev); // should this somehow bubble?
		}

		if self.get_queue().len() == 0 { FsmQueueStatus::Empty } else { FsmQueueStatus::MoreEventsQueued }
	}
	
	fn get_message_queue_size(&self) -> usize {
		self.get_queue().len()
	}
}

pub trait FsmProcessor<F: Fsm, E> {
	fn process_event(&mut self, event: E) -> Result<(), FsmError>;
}


// codegen types

pub struct InitialState<F: Fsm, S: FsmState<F>>(PhantomData<F>, S);
pub struct ContextType<T>(T);
pub struct SubMachine<F: Fsm>(F);
pub struct ShallowHistory<F: Fsm, E: FsmEvent, StateTarget: FsmState<F> + Fsm>(PhantomData<F>, E, StateTarget);
pub struct InterruptState<F: Fsm, S: FsmState<F>, E: FsmEvent>(PhantomData<F>, S, E);
pub struct StopState<F: Fsm, S: FsmState<F>>(PhantomData<F>, S);
pub struct CopyableEvents;


pub struct Transition<F: Fsm, StateSource: FsmState<F>, E: FsmEvent, StateTarget: FsmState<F>, A: FsmAction<F, StateSource, E, StateTarget>>(PhantomData<F>, StateSource, E, StateTarget, A);
pub struct TransitionSelf<F: Fsm, State: FsmState<F>, E: FsmEvent, A: FsmActionSelf<F, State, E>>(PhantomData<F>, State, E, A);
pub struct TransitionInternal<F: Fsm, State: FsmState<F>, E: FsmEvent, A: FsmActionSelf<F, State, E>>(PhantomData<F>, State, E, A);

pub struct TransitionGuard<F: Fsm, StateSource: FsmState<F>, E: FsmEvent, StateTarget: FsmState<F>, A: FsmAction<F, StateSource, E, StateTarget>, G: FsmGuard<F, E>>(PhantomData<F>, StateSource, E, StateTarget, A, G);
pub struct TransitionSelfGuard<F: Fsm, State: FsmState<F>, E: FsmEvent, A: FsmActionSelf<F, State, E>, G: FsmGuard<F, E>>(PhantomData<F>, State, E, A, G);
pub struct TransitionInternalGuard<F: Fsm, State: FsmState<F>, E: FsmEvent, A: FsmActionSelf<F, State, E>, G: FsmGuard<F, E>>(PhantomData<F>, State, E, A, G);



pub trait StateTimeout<F: Fsm> : FsmState<F> {
	fn timeout_on_entry(&self, event_context: &mut EventContext<F>) -> Option<TimerSettings>;
}

#[derive(Copy, Clone, Debug)]
pub struct TimerSettings {
	pub timeout: TimerDuration,
	pub cancel_on_state_exit: bool
}

#[derive(Copy, Clone, Debug)]
pub struct TimerDuration {
	pub ms: u64
}

impl TimerDuration {
	pub fn from_millis(ms: u64) -> Self {
		TimerDuration { ms: ms }
	}
}


pub struct TimerStateTimeout<F: Fsm, S: StateTimeout<F>, E: FsmEvent + Default>(PhantomData<F>, S, E);



#[derive(Copy, Clone, Debug, PartialEq)]
pub struct TimerId(pub u32);

pub trait FsmTimers: Clone {
	fn implemented() -> bool { true }
	fn create_timeout_timer(&mut self, id: TimerId, duration: TimerDuration);
	fn cancel_timer(&mut self, id: TimerId);
	
	fn receive_events(&mut self) -> Vec<FsmTimerEvent>;
}

#[derive(Copy, Clone, Debug)]
pub struct FsmTimersNull;
impl FsmTimers for FsmTimersNull {
	fn implemented() -> bool { false }

	fn create_timeout_timer(&mut self, id: TimerId, duration: TimerDuration) { }
	fn cancel_timer(&mut self, id: TimerId) { }
	
	fn receive_events(&mut self) -> Vec<FsmTimerEvent> { vec![] }
}

#[derive(Copy, Clone, Debug)]
pub enum FsmTimerEvent {
	TimedOut(FsmTimerTimedOut)
}

#[derive(Copy, Clone, Debug)]
pub struct FsmTimerTimedOut {
	pub timer_id: TimerId
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












//// fsm fn decl style
pub struct FsmDecl;

impl FsmDecl {
	pub fn new_fsm<F>() -> FsmDecl2<F, ()> where F: Fsm {
		FsmDecl2 {
			fsm_ty: PhantomData::default(),
			fsm_ctx_ty: PhantomData::default()
		}
	}
}

pub struct FsmDecl2<F, Ctx> {
	fsm_ty: PhantomData<F>,
	fsm_ctx_ty: PhantomData<Ctx>
}

impl<F, Ctx> FsmDecl2<F, Ctx> where F: Fsm {	
	pub fn context_ty<C>(&self) -> FsmDecl2<F, C> {
		FsmDecl2 {
			fsm_ty: Default::default(),
			fsm_ctx_ty: Default::default()
		}
	}

	pub fn initial_state<InitialState>(&self) -> FsmDeclComplete<F, Ctx, InitialState> where InitialState: FsmState<F> {
		FsmDeclComplete {
			fsm_ty: PhantomData::default(),
			fsm_ctx_ty: PhantomData::default(),
			initial_state: PhantomData::default()
		}
	}
}

pub struct FsmDeclComplete<F, Ctx, InitialState> {
	fsm_ty: PhantomData<F>,
	fsm_ctx_ty: PhantomData<Ctx>,
	initial_state: PhantomData<InitialState>
}

impl<F, Ctx, InitialState> FsmDeclComplete<F, Ctx, InitialState> where F: Fsm, InitialState: FsmState<F> {
	pub fn new_unit_state<S>(&self) -> FsmDeclState<F, S> where S: FsmState<F> {
		FsmDeclState {
			fsm_ty: PhantomData::default(),
			state_ty: PhantomData::default()
		}
	}

	pub fn on_event<E>(&self) -> FsmDeclOnEvent<F, E> where E: FsmEvent {
		FsmDeclOnEvent {
			fsm_ty: PhantomData::default(),
			event_ty: PhantomData::default()
		}
	}
}

pub struct FsmDeclOnEvent<F, E> {
	fsm_ty: PhantomData<F>,
	event_ty: PhantomData<E>
}

impl<F, E> FsmDeclOnEvent<F, E> {
	pub fn transition_from<StateFrom>(&self) -> FsmlDeclTransitionFrom<F, E, StateFrom> where F: Fsm, E: FsmEvent, StateFrom: FsmState<F> {
		FsmlDeclTransitionFrom {
			fsm_ty: PhantomData::default(),
			event_ty: PhantomData::default(),
			state_from: PhantomData::default()
		}
	}

	pub fn transition_self<State>(&self) -> FsmDeclTransitionSingle<F, E, State> where F: Fsm, E: FsmEvent, State: FsmState<F> {
		FsmDeclTransitionSingle {
			fsm_ty: PhantomData::default(),
			event_ty: PhantomData::default(),
			state: PhantomData::default()
		}
	}

	pub fn transition_internal<State>(&self) -> FsmDeclTransitionSingle<F, E, State> where F: Fsm, E: FsmEvent, State: FsmState<F> {
		FsmDeclTransitionSingle {
			fsm_ty: PhantomData::default(),
			event_ty: PhantomData::default(),
			state: PhantomData::default()
		}
	}
}

pub struct FsmDeclTransitionSingle<F, E, State> {
	fsm_ty: PhantomData<F>,
	event_ty: PhantomData<E>,
	state: PhantomData<State>
}

impl<F, E, State> FsmDeclTransitionSingle<F, E, State> where F: Fsm, E: FsmEvent, State: FsmState<F> {
	pub fn action<FnAction: Fn(&E, &mut EventContext<F>, &mut State)>(&self, action: FnAction) -> &Self {
		self
	}

	pub fn guard<FnGuard: Fn(&E, &mut EventContext<F>, &F::SS) -> bool>(&self, guard: FnGuard) -> &Self {
		self
	}
}


pub struct FsmlDeclTransitionFrom<F, E, StateFrom> {
	fsm_ty: PhantomData<F>,
	event_ty: PhantomData<E>,
	state_from: PhantomData<StateFrom>
}

impl<F, E, StateFrom> FsmlDeclTransitionFrom<F, E, StateFrom> where F: Fsm, E: FsmEvent, StateFrom: FsmState<F> {
	pub fn to<StateTo>(&self) -> FsmDeclTransition<F, E, StateFrom, StateTo> where StateTo: FsmState<F> {
		FsmDeclTransition {
			fsm_ty: PhantomData::default(),
			event_ty: PhantomData::default(),
			state_from: PhantomData::default(),
			state_to: PhantomData::default()
		}
	}
}

pub struct FsmDeclTransition<F, E, StateFrom, StateTo> {
	fsm_ty: PhantomData<F>,
	event_ty: PhantomData<E>,
	state_from: PhantomData<StateFrom>,
	state_to: PhantomData<StateTo>
}

impl<F, E, StateFrom, StateTo> FsmDeclTransition<F, E, StateFrom, StateTo> where F: Fsm {
	// fn action(event: &E, event_context: &mut EventContext<F>, source_state: &mut S, target_state: &mut T);
	pub fn action<FnAction: Fn(&E, &mut EventContext<F>, &mut StateFrom, &mut StateTo)>(&self, action: FnAction) -> &Self {
		self
	}

	pub fn guard<FnGuard: Fn(&E, &mut EventContext<F>, &F::SS) -> bool>(&self, guard: FnGuard) -> &Self {
		self
	}
}

pub struct FsmDeclState<F, S> {
	fsm_ty: PhantomData<F>,
	state_ty: PhantomData<S>
}

impl<F, S> FsmDeclState<F, S> {
	pub fn on_entry<B: Fn(&mut S, &mut EventContext<F>)>(&self, body: B) {
		
	}

	pub fn on_exit<B: Fn(&mut S, &mut EventContext<F>)>(&self, body: B) {

	}
}