extern crate finny;

use std::time::Duration;

use finny::{FsmCurrentState, FsmError, FsmEvent, FsmEventQueueVec, FsmFactory, FsmResult, decl::{BuiltFsm, FsmBuilder}, finny_fsm, inspect_slog::{self, InspectSlog}};
use slog::{Drain, Logger, info, o};

#[derive(Debug)]
pub struct TimersMachineContext {
}

#[derive(Default)]
pub struct StateA {
    timers: usize
}
#[derive(Default)]
pub struct StateB {

}
#[derive(Default)]
pub struct StateC;
#[derive(Clone, Debug)]
pub struct EventClick { time: usize }
#[derive(Clone, Debug)]
pub struct EventTimer;

#[derive(Clone, Debug)]
pub struct EventEnter { shift: bool }

#[finny_fsm]
fn build_fsm(mut fsm: FsmBuilder<TimersMachine, TimersMachineContext>) -> BuiltFsm {
    fsm.events_debug();
    fsm.initial_state::<StateA>();

    fsm.state::<StateA>();

    fsm.state::<StateA>()
        .on_exit(|state, ctx| {
            state.timers = 0;
        })
        .on_event::<EventClick>()
        .transition_to::<StateB>()
        .guard(|ev, ctx, states| {
            let state: &StateA = states.as_ref();
            state.timers > 5
        });

    fsm.state::<StateA>()
        .on_event::<EventTimer>()
        .internal_transition()
        .action(|ev, ctx, state| {
            state.timers += 1;
        });

    fsm.state::<StateA>()
        .on_entry_start_timer(|_ctx, timer| {
            timer.timeout = Duration::from_millis(100);
            timer.renew = true;
        }, |ctx, state| {
            Some( EventTimer.into() )
        });

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