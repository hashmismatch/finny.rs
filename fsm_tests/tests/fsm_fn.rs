#![feature(proc_macro)]

extern crate fsm;
#[macro_use]
extern crate fsm_codegen;

extern crate serde;
#[macro_use]
extern crate serde_derive;

use fsm::*;

use fsm_codegen::fsm_fn;


#[derive(Debug, Default, Serialize)]
pub struct FsmFnCtx {
    entry: usize,
    exit: usize,
    action: usize,
    action_internal: usize,
    action_self: usize,
    state_b: usize
}

fsm_event_unit!(EventA);
fsm_event_unit!(EventInternal);
fsm_event_unit!(EventSelf);

#[fsm_fn]
fn create_it() -> () {
    //let foo = ();
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

    
    fsm.on_event::<EventA>()
       .transition_from::<StateA>()
       .to::<StateB>()
       .action(|event, event_ctx, state_a, state_b| {
           event_ctx.context.action += 1;
       })
       .guard(|event, event_ctx, states| {
           event_ctx.context.entry == 1 && event_ctx.context.exit == 0
       });
    
    fsm.on_event::<EventInternal>()
        .transition_internal::<StateB>()
        .action(|event, event_ctx, state| {
            event_ctx.context.action_internal += 1;
        });

    fsm.on_event::<EventSelf>()
        .transition_self::<StateB>()
        .action(|event, event_ctx, state| {
            event_ctx.context.action_self += 1;
        });
}

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

/*
#[derive(Clone, PartialEq, Default, Debug, Serialize)]
pub struct StaticA;
impl FsmState<FsmMinOne> for StaticA {

}

#[derive(Fsm)]
struct FsmMinOneDefinition(
	InitialState<FsmMinOne, StaticA>
);


#[cfg(test)]
#[test]
fn test_fsm_min1() {

    let mut fsm = FsmMinOne::new(()).unwrap();
    fsm.start();
    assert_eq!(FsmMinOneStates::StaticA, fsm.get_current_state());
}
*/