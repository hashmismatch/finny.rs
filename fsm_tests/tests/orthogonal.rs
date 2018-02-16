#![feature(proc_macro)]


#[macro_use]
extern crate fsm;
#[macro_use]
extern crate fsm_codegen;

extern crate serde;
#[macro_use]
extern crate serde_derive;

use fsm::*;
use fsm_codegen::fsm_fn;


#[derive(Debug, Serialize)]
pub struct OrthoContext {
    id: &'static str   
}

#[fsm_fn]
fn ortho_fsm() {
    let fsm = FsmDecl::new_fsm::<Ortho>()
        .context_ty::<OrthoContext>()
        .initial_state::<(InitialA, InitialB, FixedC, AllOk)>();
    
    fsm.new_unit_event::<EventA>();
    fsm.new_unit_event::<EventA2>();
    fsm.new_unit_event::<EventB>();
    fsm.new_unit_event::<ErrorDetected>();
    fsm.new_unit_event::<ErrorFixed>();

    fsm.new_unit_state::<InitialA>();
    fsm.new_unit_state::<InitialB>();
    fsm.new_unit_state::<StateA>();
    fsm.new_unit_state::<StateB>();
    fsm.new_unit_state::<FixedC>();
    fsm.new_unit_state::<AllOk>();
    fsm.new_unit_state::<ErrorMode>();

    fsm.on_event::<EventA>()
       .transition_from::<InitialA>()
       .to::<StateA>();

    fsm.on_event::<EventA2>()
       .transition_from::<StateA>()
       .to::<InitialA>();

    fsm.on_event::<EventB>()
       .transition_from::<InitialB>()
       .to::<StateB>();

    fsm.on_event::<ErrorDetected>()
       .transition_from::<AllOk>()
       .to::<ErrorMode>();

    fsm.on_event::<ErrorFixed>()
       .transition_from::<ErrorMode>()
       .to::<AllOk>();

    // In case the current state is "ErrorMode", every other event other than "ErrorFixed" is blocked.
    fsm.interrupt_state::<ErrorMode>()
       .resume_on::<ErrorFixed>();
}



#[cfg(test)]
#[test]
fn test_orthogonal() {

    let id = "fsm_a";
    let ctx = OrthoContext {
        id: &id
    };
	let mut fsm = Ortho::new(ctx).unwrap();

	fsm.start();

    assert_eq!((OrthoStates::InitialA, OrthoStates::InitialB, OrthoStates::FixedC, OrthoStates::AllOk), fsm.get_current_state());

    fsm.process_event(OrthoEvents::EventA(EventA)).unwrap();
    assert_eq!((OrthoStates::StateA, OrthoStates::InitialB, OrthoStates::FixedC, OrthoStates::AllOk), fsm.get_current_state());

    fsm.process_event(OrthoEvents::EventB(EventB)).unwrap();
    assert_eq!((OrthoStates::StateA, OrthoStates::StateB, OrthoStates::FixedC, OrthoStates::AllOk), fsm.get_current_state());


    fsm.process_event(OrthoEvents::ErrorDetected(ErrorDetected)).unwrap();
    assert_eq!((OrthoStates::StateA, OrthoStates::StateB, OrthoStates::FixedC, OrthoStates::ErrorMode), fsm.get_current_state());

    assert_eq!(fsm.process_event(OrthoEvents::EventA2(EventA2)), Err(FsmError::Interrupted));

    fsm.process_event(OrthoEvents::ErrorFixed(ErrorFixed)).unwrap();
    assert_eq!((OrthoStates::StateA, OrthoStates::StateB, OrthoStates::FixedC, OrthoStates::AllOk), fsm.get_current_state());

    

}