use finny::{FsmFactory, FsmResult, decl::{BuiltFsm, FsmBuilder}, finny_fsm};
use std::{fmt::Debug, ops::AddAssign};

extern crate finny;

pub struct Ctx<'a, T> {
    ref_str: &'a str,
    val: T
}

#[derive(Debug, Default)]
pub struct StateA;

#[derive(Debug, Default)]
pub struct StateB;
#[derive(Clone)]
pub struct Event;

#[finny_fsm]
fn build_fsm<'a, TT>(mut fsm: FsmBuilder<StateMachine<'a, TT>, Ctx<'a, TT>>) -> BuiltFsm
    where TT: Debug + AddAssign<usize> + PartialOrd<usize>
{
    fsm.initial_state::<StateA>();
    fsm.state::<StateA>()
        .on_entry(|_, ctx| {
            ctx.context.val += 1;
        })
        .on_event::<Event>()
        .transition_to::<StateB>()
        .guard(|_ev, ctx| {
            ctx.context.val > 100
        })
        .action(|_ev, ctx, _, _| {
            ctx.context.val += 100;
        });

    fsm.state::<StateB>();

    fsm.build()
}

#[test]
fn test_generic_ctx() -> FsmResult<()> {
    let some_str = "hello!";
    let ctx = Ctx { val: 123, ref_str: &some_str };
    
    let mut fsm = StateMachine::new(ctx)?;
    assert_eq!(123, fsm.val);
    
    fsm.start()?;

    assert_eq!(124, fsm.val);

    fsm.dispatch(Event)?;

    assert_eq!(224, fsm.val);
    
    Ok(())
}