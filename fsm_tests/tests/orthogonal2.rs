#[macro_use]
extern crate fsm;
#[macro_use]
extern crate fsm_codegen;


use fsm::*;

// events
fsm_event_unit!(EventBoth);

// states
#[derive(Clone, PartialEq, Default)]
pub struct StateA;
impl FsmState<Ortho> for StateA { }

#[derive(Clone, PartialEq, Default)]
pub struct StateB;
impl FsmState<Ortho> for StateB { }

#[derive(Clone, PartialEq, Default)]
pub struct StateX;
impl FsmState<Ortho> for StateX { }

#[derive(Clone, PartialEq, Default)]
pub struct StateY;
impl FsmState<Ortho> for StateY { }




#[derive(Fsm)]
struct OrthoDefinition(
    InitialState<Ortho, (StateA, StateX)>,

    Transition        < Ortho, StateA, EventBoth, StateB, NoAction >,
    Transition        < Ortho, StateX, EventBoth, StateY, NoAction >
);


#[cfg(test)]
#[test]
fn test_orthogonal() {
	let mut fsm = Ortho::new(()).unwrap();
	fsm.start();

    assert_eq!((OrthoStates::StateA, OrthoStates::StateX), fsm.get_current_state());

    fsm.process_event(EventBoth).unwrap();
    assert_eq!((OrthoStates::StateB, OrthoStates::StateY), fsm.get_current_state());
}
