
pub mod declaration;
pub mod timers;
pub mod inspect;
pub mod info;

use self::info::*;

use prelude::v1::*;

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

pub trait FsmEvent<F: Fsm> {
	fn on_dispatch(&self, event_context: &mut EventContext<F>) { }
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

// just for the InitialState definition type
impl<F, A, B> FsmState<F> for (A, B) where F: Fsm, A: FsmState<F>, B: FsmState<F> { }
impl<F, A, B, C> FsmState<F> for (A, B, C) where F: Fsm, A: FsmState<F>, B: FsmState<F>, C: FsmState<F> { }
impl<F, A, B, C, D> FsmState<F> for (A, B, C, D) where F: Fsm, A: FsmState<F>, B: FsmState<F>, C: FsmState<F>, D: FsmState<F> { }
impl<F, A, B, C, D, E> FsmState<F> for (A, B, C, D, E) where F: Fsm, A: FsmState<F>, B: FsmState<F>, C: FsmState<F>, D: FsmState<F>, E: FsmState<F> { }



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
impl<F: Fsm> FsmEvent<F> for NoEvent { }

/*
pub struct NoAction;
impl<F: Fsm, S, E, T> FsmAction<F, S, E, T> for NoAction {
	#[inline]
	fn action(event: &E, event_context: &mut EventContext<F>, source_state: &mut S, target_state: &mut T) { }
}
impl<F: Fsm, S, E> FsmActionSelf<F, S, E> for NoAction {
	#[inline]
	fn action(event: &E, event_context: &mut EventContext<F>, state: &mut S) { }
}
*/

pub type RegionId = usize;

pub struct EventContext<'a, F: Fsm + 'a> {
	// todo: hide the queue object, just expose the API
	pub queue: &'a mut FsmEventQueue<F>,
	pub context: &'a mut F::C,
	//pub current_state: F::CS,
	pub region: RegionId
}

impl<'a, F: Fsm + 'a> EventContext<'a, F> {
	pub fn enqueue_event(&mut self, event: F::E) -> Result<(), FsmError> {
		self.queue.enqueue_event(event)
	}
}

use std::ops::{Deref, DerefMut};
impl<'a, F: Fsm + 'a> Deref for EventContext<'a, F> {
	type Target = F::C;

	fn deref(&self) -> &F::C {
		&self.context
	}
}
impl<'a, F: Fsm + 'a> DerefMut  for EventContext<'a, F> {
	fn deref_mut(&mut self) -> &mut F::C {
		&mut self.context
	}
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
	type EventKind: Debug + PartialEq + Clone + Copy;
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
