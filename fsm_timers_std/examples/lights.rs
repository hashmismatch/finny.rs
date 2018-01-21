#[macro_use]
extern crate fsm;
#[macro_use]
extern crate fsm_codegen;

extern crate fsm_timers_std;

use fsm::*;
use fsm::console_inspect::*;

use fsm_timers_std::*;

fsm_event_unit!(LightOffElapsed);
fsm_event_unit!(LightOnElapsed);

#[derive(Clone, PartialEq, Default)]
pub struct LightOff;
impl FsmState<Lights> for LightOff { }
impl StateTimeout<Lights> for LightOff {
    fn timeout_on_entry(&self, event_context: &mut EventContext<Lights>) -> Option<TimerSettings> {
        Some(TimerSettings {
            timeout: TimerDuration::from_millis(500),
            cancel_on_state_exit: true 
        })
    }
}

#[derive(Clone, PartialEq, Default)]
pub struct LightOn;
impl FsmState<Lights> for LightOn { }
impl StateTimeout<Lights> for LightOn {
    fn timeout_on_entry(&self, event_context: &mut EventContext<Lights>) -> Option<TimerSettings> {
        Some(TimerSettings {
            timeout: TimerDuration::from_millis(100),
            cancel_on_state_exit: true
        })
    }
}

#[derive(Fsm)]
struct LightsDefinition(
	InitialState<Lights, LightOff>,

    Transition      < Lights,  LightOff,    LightOffElapsed,    LightOn,   NoAction >,
    Transition      < Lights,  LightOn,     LightOnElapsed,     LightOff,  NoAction >,

    TimerStateTimeout < Lights, LightOff, LightOffElapsed >,
    TimerStateTimeout < Lights, LightOn,  LightOnElapsed >
);



fn main() {
    use std::time::*;

    let mut lights = Lights::new_custom((), FsmInspectStdOut, FsmTimersStd::new()).unwrap();
    lights.start();

    let run_time = Duration::from_secs(3);
    let started_at = Instant::now();

    loop {        
        let events = {
            let mut timers = lights.get_timers_mut();
            timers.receive_events()
        };

        if events.len() > 0 {
            //println!("events: {:#?}", events);

            for event in events {
                lights.process_timer_event(&event);
            }
        }

        std::thread::sleep_ms(50);

        if started_at.elapsed() > run_time {
            println!("example finished.");
            return;
        }
    }
}

