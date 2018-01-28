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
    entry: usize,
    exit: usize
}

#[fsm_fn]
fn create_it() -> () {
    //let foo = ();
    let fsm = FsmDecl::new_fsm::<FsmMinOne>()
        .context_ty::<FsmFnCtx>()
        .initial_state::<StateA>();

    fsm.new_unit_state::<StateA>()
        .on_entry(|a, b| {
            println!("Entering state.");
            b.context.entry += 1;
        })
        .on_exit(|state, ctx| {
            println!("Exiting state.");
        });

    fsm.new_unit_state::<StateB>();

    //foo
}

#[test]
fn test_fsm_min1() {
    let mut fsm = FsmMinOne::new(Default::default()).unwrap();
    fsm.start();
    assert_eq!(FsmMinOneStates::StateA, fsm.get_current_state());
    assert_eq!(1, fsm.get_context().entry);
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