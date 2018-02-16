#![feature(proc_macro)]

extern crate fsm;
#[macro_use]
extern crate fsm_codegen;


use fsm::*;
use fsm_codegen::fsm_fn;

extern crate serde;
#[macro_use]
extern crate serde_derive;


#[derive(Default, Debug, Serialize)]
pub struct FsmOneContext {
	guard1_exec: usize	
}

#[fsm_fn]
fn fsm_1() {
    let fsm = FsmDecl::new_fsm::<FsmOne>()
        .initial_state::<Initial>();

	#[derive(Clone, PartialEq, Default, Debug, Serialize)]
	pub struct Initial {
		entry: usize,
		exit: usize
	}
	fsm.new_state::<Initial>()
		.on_entry(|state, ctx| {
			state.entry += 1;
		})
		.on_exit(|state, ctx| {
			state.exit += 1;
		});

	fsm.on_event::<NoEvent>()
	   .transition_from::<Initial>().to::<State1>()
	   .action(|ev, ctx, initial, state1| {
		   println!("Init action!");
	   });

	#[derive(Clone, PartialEq, Default, Debug, Serialize)]
	pub struct State1 {
		entry: usize,
		exit: usize,
		internal_action: usize
	}
	fsm.new_state::<State1>()
		.on_entry(|state, ctx| {
			state.entry += 1;
		})
		.on_exit(|state, ctx| {
			state.exit += 1;
		});

	fsm.new_unit_event::<Event1>();
	fsm.on_event::<Event1>()
		.transition_from::<State1>().to::<State1>();
	
	fsm.new_unit_state::<State2>();

	fsm.new_unit_event::<Event2>();
	fsm.on_event::<Event2>()
	   .transition_internal::<State1>()
	   .action(|ev, ctx, state1| {
		   state1.internal_action += 1;
	   });

	fsm.new_unit_event::<Event3>();
	fsm.on_event::<Event3>()
		.transition_internal::<State1>()
		.action(|ev, ctx, state1| {
			ctx.queue.enqueue_event(FsmOneEvents::Event2(Event2));
		});		

	#[derive(Clone, PartialEq, Default, Debug, Serialize)]
	pub struct MagicEvent(u32);
	fsm.new_event::<MagicEvent>();

	fsm.on_event::<MagicEvent>()
		.transition_from::<State1>().to::<State2>()
		.guard(|ev, ctx, states| {
			ev.0 == 42
		});
}

#[cfg(test)]
#[test]
fn test_machine1() {

	let mut fsm1 = FsmOne::new(Default::default()).unwrap();
	fsm1.execute_queue_pre = true;
	fsm1.execute_queue_post = false;
	
	assert_eq!(fsm1.get_current_state(), FsmOneStates::Initial);
	{
		let initial: &Initial = fsm1.get_state();
		assert_eq!(initial.entry, 0);
		assert_eq!(initial.exit, 0);
	}

	fsm1.start();

	assert_eq!(fsm1.get_current_state(), FsmOneStates::State1);

	{
		let initial: &Initial = fsm1.get_state();
		assert_eq!(initial.entry, 1);
		assert_eq!(initial.exit, 1);

		let state1: &State1 = fsm1.get_state();
		assert_eq!(state1.entry, 1);
	}	
	
	fsm1.process_event(FsmOneEvents::Event1(Event1)).unwrap();

	{
		let state1: &State1 = fsm1.get_state();
		assert_eq!(state1.exit, 1);
		assert_eq!(state1.entry, 2);
	}

	fsm1.process_event(FsmOneEvents::Event2(Event2)).unwrap();

	{
		let state1: &State1 = fsm1.get_state();
		assert_eq!(state1.exit, 1);
		assert_eq!(state1.entry, 2);

		assert_eq!(state1.internal_action, 1);
	}

	// event queueing, implicit and explicit execution
	fsm1.process_event(Event3).unwrap();
	
	{
		let state1: &State1 = fsm1.get_state();
		assert_eq!(state1.internal_action, 1);
	}	

	fsm1.process_event(Event3).unwrap();

	{
		let state1: &State1 = fsm1.get_state();
		assert_eq!(state1.internal_action, 2);
	}

	fsm1.execute_queued_events();

	{
		let state1: &State1 = fsm1.get_state();
		assert_eq!(state1.internal_action, 3);
	}

	// event guards
	assert_eq!(Err(FsmError::NoTransition), fsm1.process_event(FsmOneEvents::MagicEvent(MagicEvent(1))));
	assert_eq!(FsmOneStates::State1, fsm1.get_current_state());

	fsm1.process_event(FsmOneEvents::MagicEvent(MagicEvent(42))).unwrap();
	assert_eq!(FsmOneStates::State2, fsm1.get_current_state());


}