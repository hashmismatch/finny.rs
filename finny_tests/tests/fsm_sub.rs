extern crate finny;

use finny::{FsmCurrentState, FsmError, FsmEvent, FsmFrontend, FsmEventQueue, FsmResult, FsmFactory, decl::{BuiltFsm, FsmBuilder}, finny_fsm};

#[derive(Default)]
pub struct StateA {
    value: usize
}

#[derive(Debug, Clone)]
pub struct Event;

#[finny_fsm]
fn build_fsm(mut fsm: FsmBuilder<StateMachine, ()>) -> BuiltFsm {
    fsm.initial_state::<StateA>();
    fsm.state::<StateA>()        
        .on_event::<Event>().transition_to::<SubStateMachine>()
    ;

    fsm.sub_machine::<SubStateMachine>()
        .with_context(|_ctx| ())
        .on_event::<Event>()
        .transition_to::<StateA>()
        ;

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
#[derive(Debug, Clone)]
pub struct SubEvent;

#[finny_fsm]
fn build_sub_fsm(mut fsm: FsmBuilder<SubStateMachine, ()>) -> BuiltFsm {
    fsm.initial_state::<SubStateA>();
    fsm.state::<SubStateA>()
        .on_entry(|state, _ctx| {
            state.value += 1;
        }).on_event::<SubEvent>()
        .transition_to::<SubStateB>()
        .action(|ev, ctx, state_a, state_b| {
            state_a.value += 1;
        });

    fsm.state::<SubStateB>();
    fsm.build()
}


#[test]
fn test_sub() -> FsmResult<()> {
    let mut fsm = StateMachine::new(())?;
    
    fsm.start()?;
    assert_eq!(FsmCurrentState::State(StateMachineCurrentState::StateA), fsm.get_current_states()[0]);

    fsm.dispatch(Event)?;
    
    assert_eq!(FsmCurrentState::State(StateMachineCurrentState::SubStateMachine), fsm.get_current_states()[0]);
    let sub: &SubStateMachine = fsm.get_state();
    assert_eq!(FsmCurrentState::State(SubStateMachineCurrentState::SubStateA), sub.get_current_states()[0]);
    let state: &SubStateA = sub.get_state();
    assert_eq!(1, state.value);
    
    let ev: SubStateMachineEvents = SubEvent.into();
    fsm.dispatch(ev)?;
    
    assert_eq!(FsmCurrentState::State(StateMachineCurrentState::SubStateMachine), fsm.get_current_states()[0]);
    let sub: &SubStateMachine = fsm.get_state();
    assert_eq!(FsmCurrentState::State(SubStateMachineCurrentState::SubStateB), sub.get_current_states()[0]);
    let state: &SubStateA = sub.get_state();
    assert_eq!(2, state.value);

    fsm.dispatch(Event)?;
    assert_eq!(FsmCurrentState::State(StateMachineCurrentState::StateA), fsm.get_current_states()[0]);
    
    Ok(())
}