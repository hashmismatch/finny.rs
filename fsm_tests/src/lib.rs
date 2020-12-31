//! An example Finny use case that showcases the generated documentation.

use std::marker::PhantomData;

use fsm_core::{FsmCurrentState, FsmError, FsmEvent, FsmFrontend, FsmResult, decl::fsm::{BuiltFsm, FsmBuilder}};


extern crate fsm_core;
#[macro_use]
extern crate fsm_derive;


#[derive(Debug)]
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

pub struct EventClick { time: usize }
pub struct EventEnter;

#[fsm_fn]
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
        });

    fsm.state::<StateB>()
        .on_entry(|state_b, ctx| {
            state_b.counter += 1;
        });

    fsm.on_event::<EventClick>()
        .transition_from::<StateA>()
        .to::<StateB>()
        .guard(|ev, ctx| {
            ev.time > 100
        })
        .action(|ev, ctx, state_from, state_to| {
            ctx.context.total_time += ev.time;
        });

    fsm.build()
}