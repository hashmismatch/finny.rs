extern crate finny;

use finny::{FsmCurrentState, FsmError, FsmEvent, FsmFrontend, FsmEventQueue, FsmResult, FsmFactory, decl::{BuiltFsm, FsmBuilder}, finny_fsm};

#[derive(Default)]
pub struct MainContext {
    sub_enter: usize,
    sub_exit: usize,
    sub_action: usize
}

#[derive(Default)]
pub struct StateA {
    value: usize
}

#[derive(Debug, Clone)]
pub struct Event;
#[derive(Debug, Clone)]
pub struct EventSub { n: usize }

#[finny_fsm]
fn build_fsm(mut fsm: FsmBuilder<StateMachine, MainContext>) -> BuiltFsm {
    fsm.initial_state::<StateA>();
    fsm.state::<StateA>()        
        .on_event::<Event>().transition_to::<SubStateMachine>()
    ;

    fsm.sub_machine::<SubStateMachine>()
        .with_context(|_ctx| ())
        .on_event::<Event>()
        .transition_to::<StateA>()
        .action(|ev, ctx, from, to| {
            to.value += 1;
        });

    fsm.sub_machine::<SubStateMachine>()
        .on_event::<EventSub>()
        .self_transition()
        .guard(|ev, ctx| ev.n > 0)
        .action(|ev, ctx, state| {
            ctx.context.sub_action += 1;
        });

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
    let mut fsm = StateMachine::new(MainContext::default())?;
    
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

    let res = fsm.dispatch(EventSub { n: 0 });
    assert_eq!(Err(FsmError::NoTransition), res);
    assert_eq!(0, fsm.sub_enter);
    assert_eq!(0, fsm.sub_exit);
    assert_eq!(0, fsm.sub_action);

    fsm.dispatch(EventSub { n: 1 })?;
    assert_eq!(0, fsm.sub_enter);
    assert_eq!(0, fsm.sub_exit);
    assert_eq!(1, fsm.sub_action);

    fsm.dispatch(Event)?;
    assert_eq!(FsmCurrentState::State(StateMachineCurrentState::StateA), fsm.get_current_states()[0]);
    
    Ok(())
}