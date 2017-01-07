#[macro_use]
extern crate fsm;
#[macro_use]
extern crate fsm_codegen;


use fsm::*;


// events

#[derive(Clone, PartialEq, Default, Debug)]
pub struct EventA;
impl FsmEvent for EventA {}

#[derive(Clone, PartialEq, Default, Debug)]
pub struct EventA2;
impl FsmEvent for EventA2 {}


#[derive(Clone, PartialEq, Default, Debug)]
pub struct EventB;
impl FsmEvent for EventB {}


#[derive(Clone, PartialEq, Default, Debug)]
pub struct ErrorDetected;
impl FsmEvent for ErrorDetected {}

#[derive(Clone, PartialEq, Default, Debug)]
pub struct ErrorFixed;
impl FsmEvent for ErrorFixed {}


// states

#[derive(Clone, PartialEq, Default)]
pub struct InitialA;
impl<'a> FsmState<Ortho<'a>> for InitialA { }

#[derive(Clone, PartialEq, Default)]
pub struct InitialB;
impl<'a> FsmState<Ortho<'a>> for InitialB { }


#[derive(Clone, PartialEq, Default)]
pub struct StateA;
impl<'a> FsmState<Ortho<'a>> for StateA { }

#[derive(Clone, PartialEq, Default)]
pub struct StateB;
impl<'a> FsmState<Ortho<'a>> for StateB { }

#[derive(Clone, PartialEq, Default)]
pub struct FixedC;
impl<'a> FsmState<Ortho<'a>> for FixedC { }



#[derive(Clone, PartialEq, Default)]
pub struct AllOk;
impl<'a> FsmState<Ortho<'a>> for AllOk { }

#[derive(Clone, PartialEq, Default)]
pub struct ErrorMode;
impl<'a> FsmState<Ortho<'a>> for ErrorMode { }


pub struct OrthoContext<'a> {
    id: &'a str   
}


#[derive(Fsm)]
struct OrthoDefinition<'a>(
    InitialState<Ortho<'a>, (InitialA, InitialB, FixedC, AllOk)>,
	ContextType<OrthoContext<'a>>,
    

    Transition        < Ortho<'a>,  InitialA,  EventA,   StateA,   NoAction>,
    Transition        < Ortho<'a>,  StateA,    EventA2,  InitialA, NoAction>,

    Transition        < Ortho<'a>,  InitialB,  EventB,   StateB, NoAction>,

    Transition        < Ortho<'a>,  AllOk,     ErrorDetected, ErrorMode, NoAction >,
	Transition        < Ortho<'a>,  ErrorMode, ErrorFixed,    AllOk,     NoAction >,

    // In case the current state is "ErrorMode", every other event other than "ErrorFixed" is blocked.
    InterruptState    < Ortho<'a>,  ErrorMode, ErrorFixed >
);


#[cfg(test)]
#[test]
fn test_orthogonal() {

    let id = "fsm_a";
    let ctx = OrthoContext {
        id: &id
    };
	let mut fsm = Ortho::new(ctx);

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