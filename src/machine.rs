use prelude::v1::*;


#[derive(Copy, Clone, Debug, PartialEq)]
pub enum FsmError {
	NoTransition,
	Interrupted
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum FsmQueueStatus {
	Empty,
	MoreEventsQueued
}

pub trait FsmEvent {

}
pub trait FsmEvents<F: Fsm> {
	fn new_no_event() -> Self;
}

pub trait FsmState<F: Fsm> {	
	fn on_entry(&mut self, event_context: &mut EventContext<F>) { }
	fn on_exit(&mut self, event_context: &mut EventContext<F>) { }		
}

pub trait FsmInspect<F: Fsm> {
	fn new_from_context(context: &F::C) -> Self;

	fn on_state_entry(&self, state: &F::S, event_context: &EventContext<F>) { }
	fn on_state_exit(&self, state: &F::S, event_context: &EventContext<F>) { }
	fn on_action(&self, state: &F::S, event_context: &EventContext<F>) { }
	fn on_transition(&self, source_state: &F::S, target_state: &F::S, event_context: &EventContext<F>) { }
	fn on_no_transition(&self, current_state: &F::S, event_context: &EventContext<F>) { }
}

#[derive(Default)]
pub struct FsmInspectNull<F: Fsm> {
	_fsm_ty: PhantomData<F>
}

impl<F: Fsm> FsmInspect<F> for FsmInspectNull<F> {
	fn new_from_context(context: &F::C) -> Self {
		FsmInspectNull {
			_fsm_ty: PhantomData
		}
	}
}

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

pub trait FsmGuard<F: Fsm> {
	fn guard(event_context: &EventContext<F>) -> bool;
}

pub struct NoGuard;
impl<F: Fsm> FsmGuard<F> for NoGuard {
	#[inline]
	fn guard(event_context: &EventContext<F>) -> bool {
		true
	}
}


pub trait FsmAction<F: Fsm, S, T> {
	fn action(event_context: &mut EventContext<F>, source_state: &mut S, target_state: &mut T);
}

pub trait FsmActionSelf<F: Fsm, S> {
	fn action(event_context: &mut EventContext<F>, state: &mut S);
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub struct NoEvent;
impl FsmEvent for NoEvent { }

pub struct NoAction;
impl<F: Fsm, S, T> FsmAction<F, S, T> for NoAction {
	#[inline]
	fn action(event_context: &mut EventContext<F>, source_state: &mut S, target_state: &mut T) { }
}
impl<F: Fsm, S> FsmActionSelf<F, S> for NoAction {
	#[inline]
	fn action(event_context: &mut EventContext<F>, state: &mut S) { }
}


pub struct EventContext<'a, F: Fsm + 'a> {
	pub event: &'a F::E,
	pub queue: &'a mut FsmEventQueue<F>,
	pub context: &'a mut F::C
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


pub trait Fsm where Self: Sized {
	type E: FsmEvents<Self>;
	type S;
	type C;
	type CS: Debug;

	fn new(context: Self::C) -> Self;

	fn start(&mut self);
	fn stop(&mut self);

	fn get_queue(&self) -> &FsmEventQueue<Self>;
	fn get_queue_mut(&mut self) -> &mut FsmEventQueue<Self>;

	fn get_current_state(&self) -> Self::CS;
	
	fn process_anonymous_transitions(&mut self) -> Result<(), FsmError> {
		loop {
			match self.process_event(Self::E::new_no_event()) {
				Ok(_) => { continue; }
				Err(_) => {
					break;
				}
			}
		}

		Ok(())
	}	

	fn process_event(&mut self, event: Self::E) -> Result<(), FsmError>;

	
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
			self.process_event(ev); // should this somehow bubble?
		}

		if self.get_queue().len() == 0 { FsmQueueStatus::Empty } else { FsmQueueStatus::MoreEventsQueued }
	}
	
	fn get_message_queue_size(&self) -> usize {
		self.get_queue().len()
	}
}


// codegen types

pub struct InitialState<F: Fsm, S: FsmState<F>>(PhantomData<F>, S);
pub struct ContextType<T>(T);
pub struct InspectionType<F: Fsm, T: FsmInspect<F>>(PhantomData<F>, T);
pub struct SubMachine<F: Fsm>(F);
pub struct ShallowHistory<F: Fsm, E: FsmEvent, StateTarget: FsmState<F> + Fsm>(PhantomData<F>, E, StateTarget);
pub struct InterruptState<F: Fsm, S: FsmState<F>, E: FsmEvent>(PhantomData<F>, S, E);



pub struct Transition<F: Fsm, StateSource: FsmState<F>, E: FsmEvent, StateTarget: FsmState<F>, A: FsmAction<F, StateSource, StateTarget>>(PhantomData<F>, StateSource, E, StateTarget, A);
pub struct TransitionSelf<F: Fsm, State: FsmState<F>, E: FsmEvent, A: FsmActionSelf<F, State>>(PhantomData<F>, State, E, A);
pub struct TransitionInternal<F: Fsm, State: FsmState<F>, E: FsmEvent, A: FsmActionSelf<F, State>>(PhantomData<F>, State, E, A);

pub struct TransitionGuard<F: Fsm, StateSource: FsmState<F>, E: FsmEvent, StateTarget: FsmState<F>, A: FsmAction<F, StateSource, StateTarget>, G: FsmGuard<F>>(PhantomData<F>, StateSource, E, StateTarget, A, G);
pub struct TransitionSelfGuard<F: Fsm, State: FsmState<F>, E: FsmEvent, A: FsmActionSelf<F, State>, G: FsmGuard<F>>(PhantomData<F>, State, E, A, G);
pub struct TransitionInternalGuard<F: Fsm, State: FsmState<F>, E: FsmEvent, A: FsmActionSelf<F, State>, G: FsmGuard<F>>(PhantomData<F>, State, E, A, G);
