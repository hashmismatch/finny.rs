use finny::{FsmFactory, FsmResult, decl::{BuiltFsm, FsmBuilder}, finny_fsm};
use std::{fmt::Debug, ops::AddAssign};

extern crate finny;

pub struct Ctx<T> {
    val: T
}

#[derive(Debug, Default)]
pub struct StateA;

#[derive(Debug, Default)]
pub struct StateB;

pub struct Event;

#[finny_fsm]
fn build_fsm<TT>(mut fsm: FsmBuilder<StateMachine<TT>, Ctx<TT>>) -> BuiltFsm
    where TT: Debug + AddAssign<usize> + PartialOrd<usize>
{
    fsm.initial_state::<StateA>();
    fsm.state::<StateA>()
        .on_entry(|_state, ctx| {
            ctx.context.val += 1;
            println!("Val: {:?}", ctx.context.val);
        })
        .on_event::<Event>()
        .transition_to::<StateB>()
        .guard(|_ev, ctx| {
            ctx.context.val > 100
        })
        .action(|_ev, ctx, _state_from, _state_to| {
            ctx.context.val += 100;
        });

    fsm.state::<StateB>();

    fsm.build()
}

#[test]
fn test_generic_ctx() -> FsmResult<()> {
    let ctx = Ctx { val: 123 };
    
    let mut fsm = StateMachine::new(ctx)?;
    assert_eq!(123, fsm.val);
    
    fsm.start()?;

    assert_eq!(124, fsm.val);

    fsm.dispatch(Event)?;

    assert_eq!(224, fsm.val);
    
    Ok(())
}