use std::ops::Add;

use finny::{FsmFactory, FsmResult, decl::{BuiltFsm, FsmBuilder}, finny_fsm};

extern crate finny;

pub struct Ctx<T> {
    val: T
}

#[derive(Debug, Default)]
pub struct StateA;

#[finny_fsm]
fn build_fsm<TT>(mut fsm: FsmBuilder<StateMachine<TT>, Ctx<TT>>) -> BuiltFsm
    //where TT: Add<usize>
{
    fsm.initial_state::<StateA>();
    fsm.state::<StateA>()
        .on_entry(|_state, ev| {
            //ev.val += 1;
        });

    fsm.build()
}

#[test]
fn test_fsm() -> FsmResult<()> {
    let ctx = Ctx { val: 0 as usize };
    
    let mut fsm = StateMachine::new(ctx)?;
    
    let current_state = fsm.get_current_state();
    let state: &StateA = fsm.get_state();
    
    fsm.start()?;
    
    Ok(())
}