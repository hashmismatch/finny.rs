#![feature(proc_macro)]

extern crate fsm;
#[macro_use]
extern crate fsm_codegen;

extern crate serde;
#[macro_use]
extern crate serde_derive;

use fsm::*;

pub trait SomeTrait: Clone {
    fn some_stuff(&self);
}

#[derive(Debug, Serialize)]
pub struct Context<G> where G: std::fmt::Debug + serde::Serialize {
    data: G
}

#[derive(Clone, PartialEq, Default, Debug, Serialize)]
pub struct StaticA;
impl<G: SomeTrait + std::fmt::Debug + serde::Serialize> FsmState<FsmMinOne<G>> for StaticA {

}

use fsm_codegen::fsm_fn;
#[fsm_fn]
fn fsm_create_it<G: SomeTrait + std::fmt::Debug + serde::Serialize>() -> () {
    let fsm = FsmDecl::new_fsm::<FsmMinOne<G>>()
        .context_ty::<Context<G>>()
        .initial_state::<StaticA>();
}

#[cfg(test)]
#[test]
fn test_fsm_generic() {
    #[derive(Clone, Debug, Serialize)]
    pub struct SomeTraitImpl;
    impl SomeTrait for SomeTraitImpl {
        fn some_stuff(&self) { }
    }

    let ctx = Context { data: SomeTraitImpl };
    let mut fsm = FsmMinOne::new(ctx).unwrap();
    fsm.start();
    assert_eq!(FsmMinOneStates::StaticA, fsm.get_current_state());
}
