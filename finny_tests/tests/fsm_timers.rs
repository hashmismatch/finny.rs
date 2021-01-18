extern crate finny;

use finny::{FsmCurrentState, FsmError, FsmEvent, FsmEventQueueVec, FsmFactory, FsmResult, decl::{BuiltFsm, FsmBuilder}, finny_fsm, inspect_slog::{self, InspectSlog}};
use slog::{Drain, Logger, info, o};

#[derive(Debug)]
pub struct TimersMachineContext {
}

#[derive(Default)]
pub struct StateA {

}
#[derive(Default)]
pub struct StateB {

}
#[derive(Default)]
pub struct StateC;
#[derive(Clone, Debug)]
pub struct EventClick { time: usize }
#[derive(Clone, Debug)]
pub struct EventEnter { shift: bool }

#[finny_fsm]
fn build_fsm(mut fsm: FsmBuilder<TimersMachine, TimersMachineContext>) -> BuiltFsm {
    fsm.events_debug();
    fsm.initial_state::<StateA>();

    fsm.state::<StateA>();

    fsm.state::<StateA>()
        .on_event::<EventClick>()
        .transition_to::<StateB>();

    fsm.state::<StateB>();

    fsm.build()
}


#[test]
fn test_timers_fsm() -> FsmResult<()> {
    let plain = slog_term::PlainSyncDecorator::new(std::io::stdout());
    let logger = Logger::root(
        slog_term::FullFormat::new(plain)
        .build().fuse(), o!()
    );

    let ctx = TimersMachineContext { };
    
    let mut fsm = TimersMachine::new_with(ctx, FsmEventQueueVec::new(), InspectSlog::new(Some(logger)))?;
    
    fsm.start()?;   
    Ok(())
}