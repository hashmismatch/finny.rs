//! Transient structures that are used for FSM construction in derivation functions.

use prelude::v1::*;
use fsm::*;
use super::timers::*;


pub trait FsmOptions {
	fn copy_events(&self) -> &Self { self }
	fn sub_machine<FSub: Fsm>(&self) -> &Self { self }
}



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
impl<F, Ctx> FsmOptions for FsmDecl2<F, Ctx> { }

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

	pub fn new_state<S>(&self) -> FsmDeclState<F, S> where S: FsmState<F> {
		FsmDeclState {
			fsm_ty: PhantomData::default(),
			state_ty: PhantomData::default()
		}
	}

	pub fn new_unit_event<E>(&self) where E: FsmEvent<F> {

	}

	pub fn new_event<E>(&self) where E: FsmEvent<F> {

	}

	pub fn interrupt_state<S>(&self) -> FsmDeclInterruptState<F, S> where S: FsmState<F> {
		FsmDeclInterruptState {
			fsm_ty: PhantomData::default(),
			state_ty: PhantomData::default()
		}
	}

	pub fn new_state_timeout<State, E, FnTimer>(&self, create_timer: FnTimer)
		where State: FsmState<F>, E: FsmEvent<F>,
		      FnTimer: Fn(EventContext<F>) -> Option<TimerSettings<E>>
	{

	}

	pub fn new_state_timeout_transition<StateFrom, StateTo, FnTimer>(&self, create_timer: FnTimer) -> FsmDeclStateTimeoutTransition<F, StateFrom, StateTo>
		where StateFrom: FsmState<F>, StateTo: FsmState<F>,
			  FnTimer: Fn(EventContext<F>) -> Option<TransitionTimerSettings>
	{
		FsmDeclStateTimeoutTransition {
			fsm: PhantomData::default(),
			state_from: PhantomData::default(),
			state_to: PhantomData::default()
		}
	}

	pub fn on_event<E>(&self) -> FsmDeclOnEvent<F, E> where E: FsmEvent<F> {
		FsmDeclOnEvent {
			fsm_ty: PhantomData::default(),
			event_ty: PhantomData::default()
		}
	}

	pub fn add_sub_machine<FSub>(&self) -> FsmDeclSubMachine<F, FSub> where FSub: Fsm {
		FsmDeclSubMachine {
			fsm: PhantomData::default(),
			fsm_sub: PhantomData::default()
		}
	}
}

pub struct FsmDeclStateTimeoutTransition<F, StateFrom, StateTo> {
	fsm: PhantomData<F>,
	state_from: PhantomData<StateFrom>,
	state_to: PhantomData<StateTo>
}

impl<F, StateFrom, StateTo> FsmDeclStateTimeoutTransition<F, StateFrom, StateTo> where F: Fsm, StateFrom: FsmState<F>, StateTo: FsmState<F> {
	pub fn action<A: Fn(&mut EventContext<F>, &mut StateFrom, &mut StateTo)>(&self, action: A) {
		
	}
}


pub struct FsmDeclSubMachine<F, FSub> {
	fsm: PhantomData<F>,
	fsm_sub: PhantomData<FSub>
}

impl<F, FSub> FsmDeclSubMachine<F, FSub> where F: Fsm, FSub: Fsm + FsmState<F> {
	pub fn on_entry<B: Fn(&mut FSub, &mut EventContext<F>)>(&self, body: B) -> &Self {
		self
	}

	pub fn on_exit<B: Fn(&mut FSub, &mut EventContext<F>)>(&self, body: B) -> &Self {
		self
	}	
}


impl<F, Ctx, InitialState> FsmOptions for FsmDeclComplete<F, Ctx, InitialState> { }

pub struct FsmDeclOnEvent<F, E> {
	fsm_ty: PhantomData<F>,
	event_ty: PhantomData<E>
}

impl<F, E> FsmDeclOnEvent<F, E> where F: Fsm, E: FsmEvent<F> {
	pub fn transition_from<StateFrom>(&self) -> FsmlDeclTransitionFrom<F, E, StateFrom> where StateFrom: FsmState<F> {
		FsmlDeclTransitionFrom {
			fsm_ty: PhantomData::default(),
			event_ty: PhantomData::default(),
			state_from: PhantomData::default()
		}
	}

	pub fn transition_from_any(&self) -> FsmDeclTransitionFromAny<F, E> {
		FsmDeclTransitionFromAny {
			fsm_ty: PhantomData::default(),
			event_ty: PhantomData::default()
		}
	}

	pub fn transition_self<State>(&self) -> FsmDeclTransitionSingle<F, E, State> where State: FsmState<F> {
		FsmDeclTransitionSingle {
			fsm_ty: PhantomData::default(),
			event_ty: PhantomData::default(),
			state: PhantomData::default()
		}
	}

	pub fn transition_internal<State>(&self) -> FsmDeclTransitionSingle<F, E, State> where State: FsmState<F> {
		FsmDeclTransitionSingle {
			fsm_ty: PhantomData::default(),
			event_ty: PhantomData::default(),
			state: PhantomData::default()
		}
	}

	pub fn shallow_history<State>(&self) -> &Self where State: FsmState<F> {
		self
	}
}

pub struct FsmDeclTransitionSingle<F, E, State> {
	fsm_ty: PhantomData<F>,
	event_ty: PhantomData<E>,
	state: PhantomData<State>
}

impl<F, E, State> FsmDeclTransitionSingle<F, E, State> where F: Fsm, E: FsmEvent<F>, State: FsmState<F> {
	pub fn action<FnAction: Fn(&E, &mut EventContext<F>, &mut State)>(&self, action: FnAction) -> &Self {
		self
	}

	pub fn guard<FnGuard: Fn(&E, &mut EventContext<F>, &F::SS) -> bool>(&self, guard: FnGuard) -> &Self {
		self
	}
}

pub struct FsmDeclTransitionFromAny<F, E> {
	fsm_ty: PhantomData<F>,
	event_ty: PhantomData<E>
}

impl<F, E> FsmDeclTransitionFromAny<F, E> where F: Fsm, E: FsmEvent<F> {
	/*
	pub fn to<StateTo>(&self) -> FsmDeclTransition<F, E, StateFrom, StateTo> where StateTo: FsmState<F> {
		FsmDeclTransition {
			fsm_ty: PhantomData::default(),
			event_ty: PhantomData::default(),
			state_from: PhantomData::default(),
			state_to: PhantomData::default()
		}
	}
	*/

	pub fn to<StateTo>(&self) {

	}
}

pub struct FsmlDeclTransitionFrom<F, E, StateFrom> {
	fsm_ty: PhantomData<F>,
	event_ty: PhantomData<E>,
	state_from: PhantomData<StateFrom>
}

impl<F, E, StateFrom> FsmlDeclTransitionFrom<F, E, StateFrom> where F: Fsm, E: FsmEvent<F>, StateFrom: FsmState<F> {
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
	pub fn on_entry<B: Fn(&mut S, &mut EventContext<F>)>(&self, body: B) -> &Self {
		self
	}

	pub fn on_exit<B: Fn(&mut S, &mut EventContext<F>)>(&self, body: B) -> &Self {
		self
	}
}

pub struct FsmDeclInterruptState<F, S> {
	fsm_ty: PhantomData<F>,
	state_ty: PhantomData<S>
}

impl<F, S> FsmDeclInterruptState<F, S> where F: Fsm {
	pub fn resume_on<E>(&self) -> &Self where E: FsmEvent<F> {
		self
	}
}
