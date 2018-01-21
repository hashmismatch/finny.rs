extern crate fsm;
#[macro_use]
extern crate fsm_codegen;

use fsm::*;

pub trait SomeTrait: Clone {
    fn some_stuff(&self);
}

pub struct Context<G> {
    data: G
}

#[derive(Clone, PartialEq, Default)]
pub struct StaticA;
impl<G: SomeTrait> FsmState<FsmMinOne<G>> for StaticA {

}

#[derive(Fsm)]
struct FsmMinOneDefinition<G: SomeTrait>(
    ContextType<Context<G>>,
	InitialState<FsmMinOne<G>, StaticA>
);


#[cfg(test)]
#[test]
fn test_fsm_generic() {
    #[derive(Clone)]
    pub struct SomeTraitImpl;
    impl SomeTrait for SomeTraitImpl {
        fn some_stuff(&self) { }
    }

    let ctx = Context { data: SomeTraitImpl };
    let mut fsm = FsmMinOne::new(ctx).unwrap();
    fsm.start();
    assert_eq!(FsmMinOneStates::StaticA, fsm.get_current_state());
}
