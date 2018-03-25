#![feature(proc_macro)]

extern crate fsm;
#[macro_use]
extern crate fsm_codegen;

extern crate serde;
#[macro_use]
extern crate serde_derive;

use fsm::*;
use fsm::declaration::*;
use fsm_codegen::fsm_fn;


#[derive(Debug, Default, Serialize)]
pub struct FsmFnCtx {

}

#[fsm_fn]
fn create_it() -> () {
    let fsm = FsmDecl::new_fsm::<FsmTwo>()
        .context_ty::<FsmFnCtx>()
        .initial_state::<StateA>();
    
    #[derive(Debug, Default, Serialize)]
    pub struct StateA {
        entry: usize,
        exit: usize
    }
    fsm.new_state::<StateA>()
        .on_entry(|state, ctx| {
            state.entry += 1;
        })
        .on_exit(|state, ctx| {
            state.exit += 1;
        });

    fsm.new_unit_state::<StateB>();

    #[derive(Debug, Default, Serialize)]
    pub struct EventA {
        pub magic_number: usize
    }
    fsm.new_event::<EventA>();

    fsm.on_event::<EventA>()
        .transition_from::<StateA>()
        .to::<StateB>()
        .guard(|event, event_ctx, states| {
            event.magic_number == 42
        });
}

#[test]
fn test_fsm_fn2() {
    let mut fsm = FsmTwo::new(Default::default()).unwrap();
    fsm.start();

    assert_eq!(FsmTwoStates::StateA, fsm.get_current_state());
    {
        let state: &StateA = fsm.get_state();
        assert_eq!(1, state.entry);
    }

    assert_eq!(FsmError::NoTransition, fsm.process_event(EventA { magic_number: 10 }).unwrap_err());
    assert_eq!(FsmTwoStates::StateA, fsm.get_current_state());

    fsm.process_event(EventA { magic_number: 42 }).unwrap();
    assert_eq!(FsmTwoStates::StateB, fsm.get_current_state());
    {
        let state: &StateA = fsm.get_state();
        assert_eq!(1, state.exit);
    }
}
