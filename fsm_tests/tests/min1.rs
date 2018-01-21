extern crate fsm;
#[macro_use]
extern crate fsm_codegen;

use fsm::*;

#[derive(Clone, PartialEq, Default)]
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