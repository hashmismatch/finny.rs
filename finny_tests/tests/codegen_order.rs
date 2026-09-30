//! The generated names and orders follow the order of the declarations, the same in every build.

use finny::{AllVariants, FsmEventQueueVec, FsmFactory, FsmResult, FsmTimersNull, decl::{BuiltFsm, FsmBuilder}, finny_fsm};
use finny_tests::test_utils::TransitionNames;

#[derive(Default)]
pub struct S1;
#[derive(Default)]
pub struct S2;
#[derive(Default)]
pub struct S3;
#[derive(Clone, Debug)]
pub struct E1;
#[derive(Clone, Debug)]
pub struct E2;
#[derive(Clone, Debug)]
pub struct E3;

#[finny_fsm]
fn build_fsm(mut fsm: FsmBuilder<OrderFsm, ()>) -> BuiltFsm {
    fsm.initial_state::<S1>();

    fsm.state::<S1>().on_event::<E1>().transition_to::<S2>();
    fsm.state::<S2>().on_event::<E2>().transition_to::<S3>();
    fsm.state::<S3>().on_event::<E3>().transition_to::<S1>();

    fsm.state::<S1>().on_entry_start_timer(|_, _| { }, |_, _| None);
    fsm.state::<S2>().on_entry_start_timer(|_, _| { }, |_, _| None);
    fsm.state::<S3>().on_entry_start_timer(|_, _| { }, |_, _| None);

    fsm.build()
}

#[test]
fn test_transitions_are_numbered_in_declaration_order() -> FsmResult<()> {
    let inspect = TransitionNames::default();
    let mut fsm = OrderFsm::new_with((), FsmEventQueueVec::new(), inspect.clone(), FsmTimersNull)?;

    fsm.start()?;
    fsm.dispatch(E1)?;
    fsm.dispatch(E2)?;
    fsm.dispatch(E3)?;

    assert_eq!(vec!["OrderFsmTransition1", "OrderFsmTransition2", "OrderFsmTransition3", "OrderFsmTransition4"], *inspect.names.borrow());

    Ok(())
}

#[test]
fn test_timers_are_ordered_by_declaration() {
    let timers: Vec<_> = OrderFsmTimers::iter().collect();
    assert_eq!(vec![OrderFsmTimers::OrderFsmTimer1, OrderFsmTimers::OrderFsmTimer2, OrderFsmTimers::OrderFsmTimer3], timers);
    assert!(OrderFsmTimers::OrderFsmTimer1 < OrderFsmTimers::OrderFsmTimer2);
    assert!(OrderFsmTimers::OrderFsmTimer2 < OrderFsmTimers::OrderFsmTimer3);
}

#[derive(Default)]
pub struct Ctx {
    guarded: bool
}
#[derive(Default)]
pub struct Start;
#[derive(Default)]
pub struct Guarded;
#[derive(Default)]
pub struct Fallback;
#[derive(Clone, Debug)]
pub struct Go;

/// For the same state and event, the transitions are tried in the order of their declaration.
#[finny_fsm]
fn build_priority(mut fsm: FsmBuilder<PriorityFsm, Ctx>) -> BuiltFsm {
    fsm.initial_state::<Start>();

    fsm.state::<Start>()
        .on_event::<Go>()
        .transition_to::<Guarded>()
        .guard(|_, ctx, _| ctx.guarded);

    fsm.state::<Start>()
        .on_event::<Go>()
        .transition_to::<Fallback>();

    fsm.state::<Guarded>();
    fsm.state::<Fallback>();

    fsm.build()
}

#[test]
fn test_transitions_are_tried_in_declaration_order() -> FsmResult<()> {
    let mut fsm = PriorityFsm::new(Ctx { guarded: true })?;
    fsm.start()?;
    fsm.dispatch(Go)?;
    assert_eq!(finny::FsmCurrentState::State(PriorityFsmCurrentState::Guarded), fsm.get_current_states()[0]);

    let mut fsm = PriorityFsm::new(Ctx { guarded: false })?;
    fsm.start()?;
    fsm.dispatch(Go)?;
    assert_eq!(finny::FsmCurrentState::State(PriorityFsmCurrentState::Fallback), fsm.get_current_states()[0]);

    Ok(())
}
