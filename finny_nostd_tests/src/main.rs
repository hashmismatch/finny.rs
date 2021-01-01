#![no_std]
#![no_main]

extern crate libc;
extern crate finny;
extern crate heapless;

use finny::{FsmFactory, FsmEventQueueHeapless};
use heapless::consts::*;

#[no_mangle]
pub extern "C" fn main(_argc: isize, _argv: *const *const u8) -> isize {
    // Since we are passing a C string the final null character is mandatory
    const HELLO: &'static str = "Hello, world!\n\0";
    unsafe {
        libc::printf(HELLO.as_ptr() as *const _);
    }

    {
        let ctx = StateMachineContext::default();
        let queue = FsmEventQueueHeapless::<_, U8>::new();
        let mut fsm = StateMachine::new_with(ctx, queue).unwrap();
        fsm.start().unwrap();
    }

    0
}

#[panic_handler]
fn my_panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}


///////////////////////////////////////////////////

#[derive(Debug, Default)]
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

use finny::{finny_fsm};
use finny::decl::*;

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
