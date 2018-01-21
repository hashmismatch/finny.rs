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

#[derive(Fsm)]
struct FsmMinOneDefinition<G: SomeTrait + std::fmt::Debug + serde::Serialize>(
    ContextType<Context<G>>,
	InitialState<FsmMinOne<G>, StaticA>
);


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
