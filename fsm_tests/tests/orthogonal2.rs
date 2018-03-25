#![feature(proc_macro)]


#[macro_use]
extern crate fsm;
#[macro_use]
extern crate fsm_codegen;

extern crate serde;
#[macro_use]
extern crate serde_derive;

use fsm::*;
use fsm::declaration::*;
use fsm_codegen::fsm_fn;


#[fsm_fn]
fn ortho_fsm() {
    let fsm = FsmDecl::new_fsm::<Ortho>()
        .initial_state::<(StateA, StateX)>();
    
    fsm.new_unit_event::<EventBoth>();

    fsm.new_unit_state::<StateA>();
    fsm.new_unit_state::<StateB>();
    fsm.new_unit_state::<StateX>();
    fsm.new_unit_state::<StateY>();
    

    fsm.on_event::<EventBoth>()
       .transition_from::<StateA>().to::<StateB>();

    fsm.on_event::<EventBoth>()
       .transition_from::<StateX>().to::<StateY>();
}


#[cfg(test)]
#[test]
fn test_orthogonal() {
	let mut fsm = Ortho::new(()).unwrap();
	fsm.start();

    assert_eq!((OrthoStates::StateA, OrthoStates::StateX), fsm.get_current_state());

    fsm.process_event(EventBoth).unwrap();
    assert_eq!((OrthoStates::StateB, OrthoStates::StateY), fsm.get_current_state());
}
