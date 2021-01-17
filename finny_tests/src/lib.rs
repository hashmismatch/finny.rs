//! An example Finny use case that showcases the generated documentation.

use finny::{decl::{BuiltFsm, FsmBuilder}, finny_fsm};

extern crate finny;

#[derive(Debug, Default)]
pub struct StateMachineContext {
    count: usize,
    total_time: usize
}

#[derive(Default)]
pub struct StateA {
    enter: usize,
    exit: usize
}
#[derive(Default)]
pub struct StateB {
    counter: usize
}

#[derive(Clone)]
pub struct EventClick { time: usize }
#[derive(Clone)]
pub struct EventEnter;

#[finny_fsm]
fn build_fsm(mut fsm: FsmBuilder<StateMachine, StateMachineContext>) -> BuiltFsm {
    fsm.initial_state::<StateA>();

    fsm.state::<StateA>()
        .on_entry(|state_a, ctx| {
            ctx.context.count += 1;
            state_a.enter += 1;
        })
        .on_exit(|state_a, ctx| {
            ctx.context.count += 1;
            state_a.exit += 1;
        })
        .on_event::<EventClick>()
        .transition_to::<StateB>()
        .guard(|ev, _ctx, _states| {
            ev.time > 100
        })
        .action(|ev, ctx, _state_from, _state_to| {
            ctx.context.total_time += ev.time;
        });

    fsm.state::<StateB>()
        .on_entry(|state_b, _ctx| {
            state_b.counter += 1;
        });
        
    fsm.build()
}
