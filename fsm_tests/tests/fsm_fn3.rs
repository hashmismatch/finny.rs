#![feature(proc_macro)]

extern crate fsm;
#[macro_use]
extern crate fsm_codegen;

extern crate serde;
#[macro_use]
extern crate serde_derive;

use fsm::*;

use fsm_codegen::fsm_fn;


#[derive(Debug, Default, Serialize)]
pub struct FsmFnCtx {
}


#[fsm_fn]
fn fsm() -> () {
    let fsm = FsmDecl::new_fsm::<FsmThree>()
        .context_ty::<FsmFnCtx>()
        .initial_state::<StateA>();
    
    fsm.add_sub_machine::<FsmSub>();

    fsm.new_unit_state::<StateA>();
    fsm.new_unit_event::<EventA>();
    
    fsm.on_event::<EventA>()
        .transition_from::<StateA>()
        .to::<FsmSub>();
        
}

#[fsm_fn]
fn fsm_sub() -> () {
    let fsm = FsmDecl::new_fsm::<FsmSub>()
        .context_ty::<FsmFnCtx>()
        .initial_state::<SubA>();

    fsm.new_unit_state::<SubA>();
    fsm.new_unit_state::<SubB>();
}

#[test]
fn test_fsm_fn3() {
    let mut fsm = FsmThree::new(Default::default()).unwrap();
    fsm.start();

    fsm.process_event(EventA).unwrap();
    assert_eq!(FsmThreeStates::FsmSub, fsm.get_current_state());
}
