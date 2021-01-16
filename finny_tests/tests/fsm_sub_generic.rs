extern crate finny;

use std::ops::{Add, AddAssign};

use finny::{FsmCurrentState, FsmError, FsmEventQueueVec, FsmFactory, FsmResult, decl::{BuiltFsm, FsmBuilder}, finny_fsm, inspect_slog::InspectSlog};
use slog::{Drain, o};

#[derive(Debug)]
pub struct MainContext<X>
    where X: Add<usize> + Add<usize, Output = X> + Copy
{
    field: X
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
fn build_fsm<X, Y>(mut fsm: FsmBuilder<StateMachine<X, Y>, MainContext<X>>) -> BuiltFsm
    where
        X: Add<usize> + Add<usize, Output = X> + Copy,
        Y: Add<isize> + Add<isize, Output = Y> + Copy
{
    fsm.initial_state::<StateA>();
    fsm.state::<StateA>()        
        .on_event::<Event>().transition_to::<SubStateMachine<Y>>()
    ;

    fsm.sub_machine::<SubStateMachine<Y>>()
        .with_context(|_ctx| {
            SubContext { f2: -10 }
        })
        .on_entry(|sub, ctx| {
            ctx.field = ctx.field + 1;
        })
        .on_exit(|sub, ctx| {
            ctx.field = ctx.field + 1;
        })
        .on_event::<Event>()
        .transition_to::<StateA>()
        .action(|ev, ctx, from, to| {
            to.value += 1;
        });

    fsm.sub_machine::<SubStateMachine<Y>>()
        .on_event::<EventSub>()
        .self_transition()
        .guard(|ev, ctx| ev.n > 0)
        .action(|ev, ctx, state| {
            ctx.field = ctx.field + 1;
        });

    fsm.build()
}

#[derive(Debug)]
pub struct SubContext<Y>
    where Y: Add<isize> + Add<isize, Output = Y> + Copy
{
    f2: Y
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
fn build_sub_fsm<Y>(mut fsm: FsmBuilder<SubStateMachine<Y>, SubContext<Y>>) -> BuiltFsm
    where Y: Add<isize> + Add<isize, Output = Y> + Copy
{
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
fn test_sub_generics() -> FsmResult<()> {
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::CompactFormat::new(decorator).build().fuse();
    let drain = std::sync::Mutex::new(drain).fuse();

    let logger = slog::Logger::root(drain, o!());
    
    let main_ctx = MainContext {
        field: 0usize
    };
    let mut fsm = StateMachine::new_with(main_ctx, FsmEventQueueVec::new(), InspectSlog::new(Some(logger)))?;
    
    fsm.start()?;
    assert_eq!(FsmCurrentState::State(StateMachineCurrentState::StateA), fsm.get_current_states()[0]);

    /*
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
    assert_eq!(1, fsm.sub_enter);
    assert_eq!(0, fsm.sub_exit);
    assert_eq!(0, fsm.sub_action);

    fsm.dispatch(EventSub { n: 1 })?;
    assert_eq!(2, fsm.sub_enter);
    assert_eq!(1, fsm.sub_exit);
    assert_eq!(1, fsm.sub_action);

    fsm.dispatch(Event)?;
    assert_eq!(FsmCurrentState::State(StateMachineCurrentState::StateA), fsm.get_current_states()[0]);
    */

    Ok(())
}