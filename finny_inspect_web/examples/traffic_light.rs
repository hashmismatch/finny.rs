//! A traffic light, driven by timers on the main thread, inspected in the browser.
//!
//! `cargo run -p finny_inspect_web --example traffic_light`, then open http://127.0.0.1:7878

use std::{thread::sleep, time::Duration};

use finny::{FsmEventQueueVec, FsmFactory, FsmResult, decl::{BuiltFsm, FsmBuilder}, finny_fsm, inspect::{chain::InspectChain, tracing::InspectTracing}, timers::std::TimersStd};
use finny_inspect_web::{Inspector, InspectorConfig};
use serde::Serialize;

#[derive(Default, Serialize)]
pub struct Intersection {
    cycles: u32,
    faults: u32,
    served_pedestrians: u32
}

// The states of the main region
#[derive(Default, Serialize)]
pub struct Off;
#[derive(Default, Serialize)]
pub struct Blinking { blinks: u32 }

// The pedestrian button's region
#[derive(Default, Serialize)]
pub struct ButtonIdle;
#[derive(Default, Serialize)]
pub struct ButtonPressed { presses: u32 }

#[derive(Clone, Debug, Serialize)]
pub struct Power { on: bool }
#[derive(Clone, Debug, Serialize)]
pub struct Fault { code: u16 }
#[derive(Clone, Debug, Serialize)]
pub struct Reset;
#[derive(Clone, Debug, Serialize)]
pub struct Blink;
#[derive(Clone, Debug, Serialize)]
pub struct Press;
#[derive(Clone, Debug, Serialize)]
pub struct Served;

#[finny_fsm]
fn build_traffic_light(mut fsm: FsmBuilder<TrafficLight, Intersection>) -> BuiltFsm {
    fsm.serde();
    fsm.initial_states::<(Off, ButtonIdle)>();

    fsm.state::<Off>()
        .on_event::<Power>()
        .transition_to::<Cycle>()
        .guard(|ev, _, _| ev.on);

    fsm.sub_machine::<Cycle>()
        .on_event::<Fault>()
        .transition_to::<Blinking>()
        .action(|_, ctx, _, _| { ctx.faults += 1; });

    fsm.sub_machine::<Cycle>()
        .on_event::<Power>()
        .transition_to::<Off>()
        .guard(|ev, _, _| !ev.on);

    fsm.state::<Blinking>()
        .on_entry_start_timer(|_ctx, timer| {
            timer.timeout = Duration::from_millis(400);
            timer.renew = true;
            timer.cancel_on_state_exit = true;
        }, |_ctx, _state| Some(Blink.into()))
        .with_timer_ty::<BlinkTimer>();

    fsm.state::<Blinking>()
        .on_event::<Blink>()
        .internal_transition()
        .action(|_, _, state| { state.blinks += 1; });

    fsm.state::<Blinking>()
        .on_event::<Reset>()
        .transition_to::<Cycle>()
        .guard(|_, _, states| {
            let blinking: &Blinking = states.as_ref();
            blinking.blinks >= 3
        });

    fsm.state::<ButtonIdle>()
        .on_event::<Press>()
        .transition_to::<ButtonPressed>()
        .action(|_, _, _, pressed| { pressed.presses = 1; });

    fsm.state::<ButtonPressed>()
        .on_event::<Press>()
        .internal_transition()
        .action(|_, _, pressed| { pressed.presses += 1; });

    fsm.state::<ButtonPressed>()
        .on_event::<Served>()
        .transition_to::<ButtonIdle>()
        .action(|_, ctx, _, _| { ctx.served_pedestrians += 1; });

    fsm.build()
}

#[derive(Default, Serialize)]
pub struct Lamps { switches: u32 }

#[derive(Default, Serialize)]
pub struct Red;
#[derive(Default, Serialize)]
pub struct RedAmber;
#[derive(Default, Serialize)]
pub struct Green { extended: bool }
#[derive(Default, Serialize)]
pub struct Amber;

#[derive(Clone, Debug, Serialize)]
pub struct Next;

#[finny_fsm]
fn build_cycle(mut fsm: FsmBuilder<Cycle, Lamps>) -> BuiltFsm {
    fsm.serde();
    fsm.initial_state::<Red>();

    fsm.state::<Red>()
        .on_entry_start_timer(|_ctx, timer| {
            timer.timeout = Duration::from_millis(1500);
            timer.cancel_on_state_exit = true;
        }, |_ctx, _state| Some(Next.into()))
        .with_timer_ty::<RedTimer>();
    fsm.state::<Red>().on_event::<Next>().transition_to::<RedAmber>()
        .action(|_, ctx, _, _| { ctx.switches += 1; });

    fsm.state::<RedAmber>()
        .on_entry_start_timer(|_ctx, timer| {
            timer.timeout = Duration::from_millis(500);
            timer.cancel_on_state_exit = true;
        }, |_ctx, _state| Some(Next.into()))
        .with_timer_ty::<RedAmberTimer>();
    fsm.state::<RedAmber>().on_event::<Next>().transition_to::<Green>()
        .action(|_, ctx, _, _| { ctx.switches += 1; });

    fsm.state::<Green>()
        .on_entry_start_timer(|_ctx, timer| {
            timer.timeout = Duration::from_millis(1500);
            timer.cancel_on_state_exit = true;
        }, |_ctx, _state| Some(Next.into()))
        .with_timer_ty::<GreenTimer>();
    fsm.state::<Green>().on_event::<Next>().transition_to::<Amber>()
        .action(|_, ctx, _, _| { ctx.switches += 1; });

    fsm.state::<Amber>()
        .on_entry_start_timer(|_ctx, timer| {
            timer.timeout = Duration::from_millis(700);
            timer.cancel_on_state_exit = true;
        }, |_ctx, _state| Some(Next.into()))
        .with_timer_ty::<AmberTimer>();
    fsm.state::<Amber>().on_event::<Next>().transition_to::<Red>()
        .action(|_, ctx, _, _| { ctx.switches += 1; });

    fsm.build()
}

fn main() -> FsmResult<()> {
    let inspector = Inspector::new(InspectorConfig::default());
    let addr = inspector.spawn("127.0.0.1:7878").expect("Failed to start the inspector");
    println!("The Finny inspector is running at http://{}", addr);

    let inspect = InspectChain::new_pair(inspector.attach::<TrafficLight>("traffic light"), InspectTracing::new());
    let mut fsm = TrafficLight::new_with(Intersection::default(), FsmEventQueueVec::new(), inspect, TimersStd::new())?;
    fsm.start()?;
    fsm.dispatch(Power { on: true })?;

    // A scripted day at the intersection, repeated forever.
    let mut tick: u64 = 0;
    loop {
        sleep(Duration::from_millis(100));
        tick += 1;
        fsm.dispatch_timer_events()?;

        // the failed dispatches are recorded too, they're expected here
        let _ = match tick % 200 {
            23 | 61 | 64 => fsm.dispatch(Press),
            90 => fsm.dispatch(Served),
            120 => fsm.dispatch(Fault { code: 42 }),
            // too early, the guard rejects it
            125 => fsm.dispatch(Reset),
            150 => fsm.dispatch(Reset),
            180 => fsm.dispatch(Power { on: false }),
            185 => fsm.dispatch(Power { on: true }),
            _ => Ok(())
        };

        if tick % 200 == 0 {
            fsm.cycles += 1;
        }
    }
}

