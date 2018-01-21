#[macro_use]
extern crate fsm;
#[macro_use]
extern crate fsm_codegen;

extern crate fsm_timers_std;

extern crate fsm_inspect_web;

extern crate serde;
#[macro_use]
extern crate serde_derive;

use fsm::*;
//use fsm::console_inspect::*;

use fsm_timers_std::*;
use fsm_inspect_web::*;

fsm_event_unit!(LightOffElapsed);
fsm_event_unit!(LightOnElapsed);

#[derive(Debug, Clone, PartialEq, Default, Serialize)]
pub struct LightOff;
impl FsmState<Lights> for LightOff { }
impl StateTimeout<Lights> for LightOff {
    fn timeout_on_entry(&self, event_context: &mut EventContext<Lights>) -> Option<TimerSettings> {
        Some(TimerSettings {
            timeout: TimerDuration::from_millis(2000),
            cancel_on_state_exit: true 
        })
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize)]
pub struct LightOn {
    pub times_switched_on: u64
}
impl FsmState<Lights> for LightOn {
    fn on_exit(&mut self, event_context: &mut EventContext<Lights>) {
        event_context.context.times_light_turned_off += 1;
    }
}
impl StateTimeout<Lights> for LightOn {
    fn timeout_on_entry(&self, event_context: &mut EventContext<Lights>) -> Option<TimerSettings> {
        Some(TimerSettings {
            timeout: TimerDuration::from_millis(1000),
            cancel_on_state_exit: true
        })
    }
}


pub struct LightTurnedOn;
impl FsmAction<Lights, LightOff, LightOffElapsed, LightOn> for LightTurnedOn {
    fn action(event: &LightOffElapsed, event_context: &mut EventContext<Lights>, source_state: &mut LightOff, target_state: &mut LightOn) {
        target_state.times_switched_on += 1;
    }
}




fsm_event_unit!(DoorsClosedElapsed);
fsm_event_unit!(DoorsOpenElapsed);


#[derive(Debug, Clone, PartialEq, Default, Serialize)]
pub struct DoorClosed;
impl FsmState<Lights> for DoorClosed { }
impl StateTimeout<Lights> for DoorClosed {
    fn timeout_on_entry(&self, event_context: &mut EventContext<Lights>) -> Option<TimerSettings> {
        Some(TimerSettings {
            timeout: TimerDuration::from_millis(3000),
            cancel_on_state_exit: true 
        })
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize)]
pub struct DoorOpen {
    pub times_opened: u64
}
impl FsmState<Lights> for DoorOpen {
    fn on_entry(&mut self, event_context: &mut EventContext<Lights>) {
        self.times_opened += 1;
    }
}
impl StateTimeout<Lights> for DoorOpen {
    fn timeout_on_entry(&self, event_context: &mut EventContext<Lights>) -> Option<TimerSettings> {
        Some(TimerSettings {
            timeout: TimerDuration::from_millis(3000),
            cancel_on_state_exit: true 
        })
    }
}

#[derive(Debug, Default, Serialize)]
pub struct LightsContext {
    pub times_light_turned_off: usize
}

#[derive(Fsm)]
struct LightsDefinition(
	InitialState<Lights, (LightOff, DoorClosed)>,
    ContextType<LightsContext>,

    Transition      < Lights,  LightOff,    LightOffElapsed,    LightOn,   LightTurnedOn >,
    Transition      < Lights,  LightOn,     LightOnElapsed,     LightOff,  NoAction >,

    TimerStateTimeout < Lights, LightOff, LightOffElapsed >,
    TimerStateTimeout < Lights, LightOn,  LightOnElapsed >,
    

    Transition      < Lights,  DoorClosed,  DoorsClosedElapsed,    DoorOpen,   NoAction >,
    Transition      < Lights,  DoorOpen,    DoorsOpenElapsed,       DoorClosed, NoAction >,

    TimerStateTimeout < Lights, DoorClosed, DoorsClosedElapsed >,
    TimerStateTimeout < Lights, DoorOpen,   DoorsOpenElapsed >
);



fn main() {
    use std::time::*;

    let ctx: LightsContext = Default::default();
    let inspect_web = ::fsm_inspect_web::server_state::FsmInspectWebServer::new::<Lights>().unwrap();
    let mut lights = Lights::new_custom(ctx, inspect_web, FsmTimersStd::new()).unwrap();
    lights.start();

    println!("fsm started");

    loop {        
        let events = {
            let mut timers = lights.get_timers_mut();
            timers.receive_events()
        };

        if events.len() > 0 {
            for event in events {
                lights.process_timer_event(&event);
            }
        }

        std::thread::sleep_ms(50);
    }
}

