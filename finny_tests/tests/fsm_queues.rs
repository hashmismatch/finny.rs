extern crate finny;

use finny::{FsmCurrentState, FsmError, FsmEvent, FsmFrontend, FsmEventQueue, FsmResult, FsmFactory, decl::{BuiltFsm, FsmBuilder}, finny_fsm};

#[derive(Default)]
pub struct StateA {
    value: usize
}
#[derive(Default)]
pub struct StateB {
    value: usize
}

#[derive(Clone)]
pub struct Event { n: usize }

#[finny_fsm]
fn build_fsm(mut fsm: FsmBuilder<StateMachine, ()>) -> BuiltFsm {
    fsm.initial_state::<StateA>();

    // emit events with added counts    
    fsm.state::<StateA>()
        .on_event::<Event>()
        .internal_transition()
        .guard(|ev, _| { ev.n < 100 })
        .action(|ev, ctx, _| {
            ctx.queue.enqueue(Event { n: ev.n + 100 });
        });

    // transition to state B if the events payload is more than 100
    fsm.state::<StateA>()
       .on_event::<Event>()
       .transition_to::<StateB>()
       .guard(|ev, _| { ev.n >= 100 });
        
    fsm.state::<StateB>()
        .on_entry(|state, _| {
            state.value += 1;
        });

    fsm.build()
}


#[test]
fn test_queues() -> FsmResult<()> {
    let mut fsm = StateMachine::new(())?;
    
    fsm.start()?;
    fsm.dispatch(Event { n: 42 })?;

    assert_eq!(FsmCurrentState::State(StateMachineCurrentState::StateB), fsm.get_current_states()[0]);
    let state_b: &StateB = fsm.get_state();
    assert_eq!(1, state_b.value);
    
    Ok(())
}