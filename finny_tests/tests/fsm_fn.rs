extern crate finny;

use finny::{FsmCurrentState, FsmError, FsmEvent, FsmFrontend, FsmResult, FsmFactory, decl::{BuiltFsm, FsmBuilder}, finny_fsm};

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


#[test]
fn test_fsm() -> FsmResult<()> {
    let ctx = StateMachineContext { count: 0, total_time: 0 };
    
    let mut fsm = StateMachine::new(ctx)?;
    
    let current_state = fsm.get_current_state();
    let state: &StateA = fsm.get_state();
    assert_eq!(0, state.enter);
    assert_eq!(FsmCurrentState::Stopped, current_state);
    assert_eq!(0, fsm.get_context().count);

    fsm.start()?;

    assert_eq!(FsmCurrentState::State(StateMachineCurrentState::StateA), fsm.get_current_state());
    assert_eq!(1, fsm.get_context().count);
    let state: &StateA = fsm.get_state();
    assert_eq!(1, state.enter);

    let ret = fsm.dispatch(EventClick { time: 99 });
    assert_eq!(Err(FsmError::NoTransition), ret);
    
    fsm.dispatch(EventClick { time: 123 })?;

    assert_eq!(2, fsm.get_context().count);
    assert_eq!(123, fsm.get_context().total_time);
    //assert_eq!()

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