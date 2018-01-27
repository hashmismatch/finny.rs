#![feature(proc_macro)]

extern crate fsm;
#[macro_use]
extern crate fsm_codegen;

extern crate serde;
#[macro_use]
extern crate serde_derive;

use fsm::*;

use fsm_codegen::fsm_fn;

#[fsm_fn]
fn create_it() -> () {
    //let foo = ();
    let fsm = FsmDecl::new_fsm::<FsmMinOne>()
        .initial_state::<StaticA>();

    fsm.new_unit_state::<StaticA>()
        .on_entry(|state, ctx| {
            println!("Entering state.");
        })
        .on_exit(|state, ctx| {
            println!("Exiting state.");
        });

    //foo
}

#[test]
fn test_fsm_min1() {
    let mut fsm = FsmMinOne::new(()).unwrap();
    fsm.start();
    assert_eq!(FsmMinOneStates::StaticA, fsm.get_current_state());
}

/*
#[derive(Clone, PartialEq, Default, Debug, Serialize)]
pub struct StaticA;
impl FsmState<FsmMinOne> for StaticA {

}

#[derive(Fsm)]
struct FsmMinOneDefinition(
	InitialState<FsmMinOne, StaticA>
);


#[cfg(test)]
#[test]
fn test_fsm_min1() {

    let mut fsm = FsmMinOne::new(()).unwrap();
    fsm.start();
    assert_eq!(FsmMinOneStates::StaticA, fsm.get_current_state());
}
*/