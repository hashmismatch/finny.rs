extern crate finny;

use finny::{FsmCurrentState, FsmError, FsmEvent, FsmFrontend, FsmEventQueue, FsmResult, FsmFactory, decl::{BuiltFsm, FsmBuilder}, finny_fsm};

#[derive(Default)]
pub struct StateA {
    value: usize
}

#[derive(Debug)]
pub struct Event;

#[finny_fsm]
fn build_fsm(mut fsm: FsmBuilder<StateMachine, ()>) -> BuiltFsm {
    fsm.initial_state::<StateA>();
    fsm.state::<StateA>()
        
        //.on_event::<Event>().transition_to::<SubStateMachine>()
    ;

    //fsm.sub_machine::<SubStateMachine>();

    fsm.build()
}

#[derive(Default)]
pub struct SubStateA {
    value: usize
}
#[derive(Default)]
pub struct SubStateB {
    value: usize
}
#[derive(Debug)]
pub struct SubEvent;

#[finny_fsm]
fn build_sub_fsm(mut fsm: FsmBuilder<SubStateMachine, ()>) -> BuiltFsm {
    fsm.initial_state::<SubStateA>();
    fsm.state::<SubStateA>();
    fsm.build()
}


#[test]
fn test_sub() -> FsmResult<()> {
    let mut fsm = StateMachine::new(())?;
    
    fsm.start()?;
    assert_eq!(FsmCurrentState::State(StateMachineCurrentState::StateA), fsm.get_current_states()[0]);

    //fsm.dispatch(Event)?;

    /*
    

    assert_eq!(FsmCurrentState::State(StateMachineCurrentState::StateB), fsm.get_current_states()[0]);
    let state_b: &StateB = fsm.get_state();
    assert_eq!(1, state_b.value);
    */
    
    Ok(())
}