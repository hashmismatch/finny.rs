use std::marker::PhantomData;

use fsm_core::{FsmError, FsmResult, decl::fsm::{BuiltFsm, FsmBuilder}};


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

    fsm.state::<StateA>();
    fsm.state::<StateB>();

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


#[test]
fn test_fsm() -> FsmResult<()> {
    let ctx = StateMachineContext { count: 0, total_time: 0 };
    let mut fsm = StateMachine::new(ctx)?;

    assert_eq!(0, fsm.get_context().count);

    fsm.start().unwrap();

    assert_eq!(1, fsm.get_context().count);

    let ret = fsm.dispatch(&FsmEvents::EventClick(EventClick { time: 99 }));
    assert_eq!(Err(FsmError::NoTransition), ret);

    fsm.dispatch(&FsmEvents::EventClick(EventClick { time: 123 }))?;

    assert_eq!(2, fsm.get_context().count);
    assert_eq!(123, fsm.get_context().total_time);

    Ok(())
}



/*
#[fsm_fn]
fn create_it() -> () {
    let fsm = FsmDecl::new_fsm::<FsmMinOne>()
        .context_ty::<FsmFnCtx>()
        .initial_state::<StateA>();

    fsm.new_unit_state::<StateA>()
        .on_entry(|a, b| {
            println!("Entering state.");
            b.context.entry += 1;
        })
        .on_exit(|state, ctx| {
            println!("Exiting state.");
            ctx.context.exit += 1;
        });

    fsm.new_unit_state::<StateB>()
        .on_entry(|state, ctx| {
            ctx.context.state_b += 1;
        })
        .on_exit(|state, ctx| {
            ctx.context.state_b += 1;
        });

    fsm.new_unit_event::<EventA>();
    fsm.on_event::<EventA>()
       .transition_from::<StateA>()
       .to::<StateB>()
       .action(|event, event_ctx, state_a, state_b| {
           event_ctx.context.action += 1;
       })
       .guard(|event, event_ctx, states| {
           event_ctx.context.entry == 1 && event_ctx.context.exit == 0
       });
    
    fsm.new_unit_event::<EventInternal>();
    fsm.on_event::<EventInternal>()
        .transition_internal::<StateB>()
        .action(|event, event_ctx, state| {
            event_ctx.context.action_internal += 1;
        });

    fsm.new_unit_event::<EventSelf>();
    fsm.on_event::<EventSelf>()
        .transition_self::<StateB>()
        .action(|event, event_ctx, state| {
            event_ctx.context.action_self += 1;
        });
}
*/

/*
#[test]
fn test_fsm_min1() {
    let mut fsm = FsmMinOne::new(Default::default()).unwrap();
    fsm.start();
    assert_eq!(FsmMinOneStates::StateA, fsm.get_current_state());
    assert_eq!(1, fsm.get_context().entry);

    fsm.process_event(EventA).unwrap();
    assert_eq!(FsmMinOneStates::StateB, fsm.get_current_state());
    assert_eq!(1, fsm.get_context().exit);
    assert_eq!(1, fsm.get_context().action);

    assert_eq!(1, fsm.get_context().state_b);
    fsm.process_event(EventInternal).unwrap();
    assert_eq!(1, fsm.get_context().action_internal);
    assert_eq!(1, fsm.get_context().state_b);
        
    fsm.process_event(EventSelf).unwrap();
    assert_eq!(1, fsm.get_context().action_self);
    assert_eq!(3, fsm.get_context().state_b);
}
*/