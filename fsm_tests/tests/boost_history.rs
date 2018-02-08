extern crate fsm;
#[macro_use]
extern crate fsm_codegen;


use fsm::*;

extern crate serde;
#[macro_use]
extern crate serde_derive;


// events
#[derive(Clone, PartialEq, Debug, Serialize)]
pub struct Play;
impl FsmEvent for Play { }

#[derive(Clone, PartialEq, Debug, Serialize)]
pub struct EndPause;
impl FsmEvent for EndPause {}

#[derive(Clone, PartialEq, Debug, Serialize)]
pub struct Stop;
impl FsmEvent for Stop {}

#[derive(Clone, PartialEq, Debug, Serialize)]
pub struct Pause;
impl FsmEvent for Pause {}

#[derive(Clone, PartialEq, Debug, Serialize)]
pub struct OpenClose;
impl FsmEvent for OpenClose {}

#[derive(Clone, PartialEq, Debug, Serialize)]
pub struct CdDetected { name: String }
impl FsmEvent for CdDetected {}

// states


#[derive(Clone, Default, Debug, Serialize)]
pub struct Empty;
impl FsmState<Player> for Empty {
	fn on_entry(&mut self, event_context: &mut EventContext<Player>) {
        event_context.context.action_empty_entry_counter += 1;
    }
	fn on_exit(&mut self, event_context: &mut EventContext<Player>) {
        event_context.context.action_empty_exit_counter += 1;
    }
}

#[derive(Clone, Default, Debug, Serialize)]
pub struct Open;
impl FsmState<Player> for Open {
	fn on_entry(&mut self, event_context: &mut EventContext<Player>) {
        event_context.context.action_open_entry_counter += 1;
    }
	fn on_exit(&mut self, event_context: &mut EventContext<Player>) {
        event_context.context.action_open_exit_counter += 1;
    }
}

#[derive(Clone, Default, Debug, Serialize)]
pub struct Stopped;
impl FsmState<Player> for Stopped {
    fn on_entry(&mut self, event_context: &mut EventContext<Player>) {
        event_context.context.action_stopped_entry_counter += 1;
    }
	fn on_exit(&mut self, event_context: &mut EventContext<Player>) {
        event_context.context.action_stopped_exit_counter += 1;
    }
}

#[derive(Clone, Default, Debug, Serialize)]
pub struct Paused;
impl FsmState<Player> for Paused {
    fn on_entry(&mut self, event_context: &mut EventContext<Player>) {
        event_context.context.action_paused_entry_counter += 1;
    }
	fn on_exit(&mut self, event_context: &mut EventContext<Player>) {
        event_context.context.action_paused_exit_counter += 1;
    }
}



// Submachine entry/exit
impl FsmState<Player> for Playing {
    fn on_entry(&mut self, event_context: &mut EventContext<Player>) {
        event_context.context.playing_fsm_entry_counter += 1;
    }

	fn on_exit(&mut self, event_context: &mut EventContext<Player>) {
        event_context.context.playing_fsm_exit_counter += 1;
    }    
}

impl FsmStateFactory for Playing {
    fn new_state<PlayerContext>(parent_context: &PlayerContext) -> Self {
        Playing::new_fsm(Default::default())
    }
}

// actions


pub struct StartPlayback;
impl FsmAction<Player, Stopped, Play, Playing> for StartPlayback {
	fn action(event: &Play, event_context: &mut EventContext<Player>, source_state: &mut Stopped, target_state: &mut Playing) {		
        println!("StartPlayback");
        event_context.context.start_playback_counter += 1;
	}
}

pub struct OpenDrawer;
impl FsmAction<Player, Empty, OpenClose, Open> for OpenDrawer {
	fn action(event: &OpenClose, event_context: &mut EventContext<Player>, source_state: &mut Empty, target_state: &mut Open) {		
        println!("OpenDrawer");
	}
}
impl FsmAction<Player, Stopped, OpenClose, Open> for OpenDrawer {
	fn action(event: &OpenClose, event_context: &mut EventContext<Player>, source_state: &mut Stopped, target_state: &mut Open) {		
        println!("OpenDrawer");
	}
}

pub struct CloseDrawer;
impl FsmAction<Player, Open, OpenClose, Empty> for CloseDrawer {
	fn action(event: &OpenClose, event_context: &mut EventContext<Player>, source_state: &mut Open, target_state: &mut Empty) {		
        println!("CloseDrawer");
	}
}

pub struct StoreCdInfo;
impl FsmAction<Player, Empty, CdDetected, Stopped> for StoreCdInfo {
    fn action(event: &CdDetected, event_context: &mut EventContext<Player>, source_state: &mut Empty, target_state: &mut Stopped) {		
        println!("StoreCdInfo: name = {}", event.name);
	}
}

pub struct StopPlayback;
impl FsmAction<Player, Playing, Stop, Stopped> for StopPlayback {
	fn action(event: &Stop, event_context: &mut EventContext<Player>, source_state: &mut Playing, target_state: &mut Stopped) {		
        println!("StopPlayback");
	}
}
impl FsmAction<Player, Paused, Stop, Stopped> for StopPlayback {
	fn action(event: &Stop, event_context: &mut EventContext<Player>, source_state: &mut Paused, target_state: &mut Stopped) {
        println!("StopPlayback");
	}
}

pub struct PausePlayback;
impl FsmAction<Player, Playing, Pause, Paused> for PausePlayback {
	fn action(event: &Pause, event_context: &mut EventContext<Player>, source_state: &mut Playing, target_state: &mut Paused) {		
        println!("PausePlayback");
	}
}

pub struct ResumePlayback;
impl FsmAction<Player, Paused, EndPause, Playing> for ResumePlayback {
	fn action(event: &EndPause, event_context: &mut EventContext<Player>, source_state: &mut Paused, target_state: &mut Playing) {
        println!("ResumePlayback");
	}
}

pub struct StopAndOpen;
impl FsmAction<Player, Playing, OpenClose, Open> for StopAndOpen {
	fn action(event: &OpenClose, event_context: &mut EventContext<Player>, source_state: &mut Playing, target_state: &mut Open) {		
        println!("StopAndOpen");
	}
}
impl FsmAction<Player, Paused, OpenClose, Open> for StopAndOpen {
	fn action(event: &OpenClose, event_context: &mut EventContext<Player>, source_state: &mut Paused, target_state: &mut Open) {		
        println!("StopAndOpen");
	}
}

pub struct StoppedAgain;
impl FsmActionSelf<Player, Stopped, Stop> for StoppedAgain {
	fn action(event: &Stop, event_context: &mut EventContext<Player>, state: &mut Stopped) {
        println!("StoppedAgain");
	}
}


#[derive(Default, Debug, Serialize)]
pub struct PlayerContext {
    action_empty_entry_counter: usize,
    action_empty_exit_counter: usize,

    action_open_entry_counter: usize,
    action_open_exit_counter: usize,

    action_stopped_entry_counter: usize,
    action_stopped_exit_counter: usize,

    action_paused_entry_counter: usize,
    action_paused_exit_counter: usize,

    start_playback_counter: usize,

    playing_fsm_entry_counter: usize,
    playing_fsm_exit_counter: usize
}


#[derive(Fsm)]
struct PlayerDefinition(
	InitialState<Player, Empty>,
	ContextType<PlayerContext>,

    SubMachine<Playing>,
    ShallowHistory<Player, EndPause, Playing>,
    
    Transition<Player, Stopped,     Play,       Playing,    StartPlayback>,
    Transition<Player, Stopped,     OpenClose,  Open,       OpenDrawer>,
    TransitionSelf<Player, Stopped,     Stop,               StoppedAgain>,

    Transition<Player, Open,        OpenClose,  Empty,      CloseDrawer>,

    Transition<Player, Empty,       OpenClose,  Open,       OpenDrawer>,
    Transition<Player, Empty,       CdDetected, Stopped,    StoreCdInfo>,

    // playing transitions
    Transition<Player,  Playing,    Stop,       Stopped,    StopPlayback>,
    Transition<Player,  Playing,    Pause,      Paused,     PausePlayback>,
    Transition<Player,  Playing,    OpenClose,  Open,       StopAndOpen>,

    Transition<Player, Paused,      EndPause,   Playing,    ResumePlayback>,
    Transition<Player, Paused,      Stop,       Stopped,    StopPlayback>,
    Transition<Player, Paused,      OpenClose,  Open,       StopAndOpen>,
);


// Playing FSM

// events
#[derive(Clone, PartialEq, Debug, Serialize)]
pub struct NextSong;
impl FsmEvent for NextSong { }

#[derive(Clone, PartialEq, Debug, Serialize)]
pub struct PreviousSong;
impl FsmEvent for PreviousSong { }


// states

#[derive(Clone, PartialEq, Default, Debug, Serialize)]
pub struct Song1;
impl FsmState<Playing> for Song1 {
    fn on_entry(&mut self, event_context: &mut EventContext<Playing>) {
        println!("Starting Song 1");
        event_context.context.song1_entry_counter += 1;
    }
	fn on_exit(&mut self, event_context: &mut EventContext<Playing>) {
        println!("Finishing Song 1");
        event_context.context.song1_exit_counter += 1;
    }
}

#[derive(Clone, PartialEq, Default, Debug, Serialize)]
pub struct Song2;
impl FsmState<Playing> for Song2 {
    fn on_entry(&mut self, event_context: &mut EventContext<Playing>) {
        println!("Starting Song 2");
        event_context.context.song2_entry_counter += 1;
    }
	fn on_exit(&mut self, event_context: &mut EventContext<Playing>) {
        println!("Finishing Song 2");
        event_context.context.song2_exit_counter += 1;
    }
}

#[derive(Clone, PartialEq, Default, Debug, Serialize)]
pub struct Song3;
impl FsmState<Playing> for Song3 {
    fn on_entry(&mut self, event_context: &mut EventContext<Playing>) {
        println!("Starting Song 3");
    }
	fn on_exit(&mut self, event_context: &mut EventContext<Playing>) {
        println!("Finishing Song 3");
    }
}



// Actions
pub struct StartNextSong;
impl FsmAction<Playing, Song1, NextSong, Song2> for StartNextSong {
	fn action(event: &NextSong, event_context: &mut EventContext<Playing>, source_state: &mut Song1, target_state: &mut Song2) {		
        println!("Playing::StartNextSong");
	}
}
impl FsmAction<Playing, Song2, NextSong, Song3> for StartNextSong {
	fn action(event: &NextSong, event_context: &mut EventContext<Playing>, source_state: &mut Song2, target_state: &mut Song3) {
        println!("Playing::StartNextSong");
	}
}

pub struct StartPrevSong;
impl FsmAction<Playing, Song2, PreviousSong, Song1> for StartPrevSong {
	fn action(event: &PreviousSong, event_context: &mut EventContext<Playing>, source_state: &mut Song2, target_state: &mut Song1) {		
        println!("Playing::StartPrevSong");
	}
}
impl FsmAction<Playing, Song3, PreviousSong, Song2> for StartPrevSong {
	fn action(event: &PreviousSong, event_context: &mut EventContext<Playing>, source_state: &mut Song3, target_state: &mut Song2) {
        println!("Playing::StartPrevSong");
	}
}



#[derive(Default, Debug, Serialize)]
pub struct PlayingContext {
    song1_entry_counter: usize,
    song1_exit_counter: usize,

    song2_entry_counter: usize,
    song2_exit_counter: usize,
}

#[derive(Fsm)]
struct PlayingDefinition(
	InitialState<Playing, Song1>,
    ContextType<PlayingContext>,
        
    Transition<Playing, Song1,  NextSong,       Song2,  StartNextSong>,
    Transition<Playing, Song2,  PreviousSong,   Song1,  StartPrevSong>,

    Transition<Playing, Song2,  NextSong,       Song3,  StartNextSong>,
    Transition<Playing, Song3,  PreviousSong,   Song2,  StartPrevSong>
);


#[cfg(test)]
#[test]
fn test_player() {

    let mut p = Player::new(Default::default()).unwrap();

	p.start();
    assert_eq!(1, p.get_context().action_empty_entry_counter);

    p.process_event(PlayerEvents::OpenClose(OpenClose)).unwrap();
    assert_eq!(PlayerStates::Open, p.get_current_state());
    assert_eq!(1, p.get_context().action_empty_exit_counter);
    assert_eq!(1, p.get_context().action_open_entry_counter);

    
    p.process_event(PlayerEvents::OpenClose(OpenClose)).unwrap();
    assert_eq!(PlayerStates::Empty, p.get_current_state());
    assert_eq!(2, p.get_context().action_empty_entry_counter);
    assert_eq!(1, p.get_context().action_open_exit_counter);

    p.process_event(PlayerEvents::CdDetected(CdDetected { name: "louie, louie".to_string() })).unwrap();
    assert_eq!(PlayerStates::Stopped, p.get_current_state());
    assert_eq!(1, p.get_context().action_stopped_entry_counter);    
    assert_eq!(2, p.get_context().action_empty_exit_counter);
    
    
    p.process_event(PlayerEvents::Play(Play)).unwrap();
    assert_eq!(PlayerStates::Playing, p.get_current_state());
    
    {
        let sub: &Playing = p.get_state();
        assert_eq!(PlayingStates::Song1, sub.get_current_state());
        assert_eq!(1, sub.get_context().song1_entry_counter);
    }

    assert_eq!(1, p.get_context().action_stopped_exit_counter);
    assert_eq!(1, p.get_context().playing_fsm_entry_counter);
    assert_eq!(1, p.get_context().start_playback_counter);

    
    p.process_event(PlayingEvents::NextSong(NextSong)).unwrap();    
    {
        let sub: &Playing = p.get_state();
        assert_eq!(PlayingStates::Song2, sub.get_current_state());
        assert_eq!(1, sub.get_context().song1_exit_counter);
        assert_eq!(1, sub.get_context().song2_entry_counter);
        assert_eq!(0, sub.get_context().song2_exit_counter);
    }
    assert_eq!(PlayerStates::Playing, p.get_current_state());
    assert_eq!(0, p.get_context().playing_fsm_exit_counter);


    p.process_event(PlayerEvents::Pause(Pause)).unwrap();
    assert_eq!(PlayerStates::Paused, p.get_current_state());
    assert_eq!(1, p.get_context().action_paused_entry_counter);
    assert_eq!(1, p.get_context().playing_fsm_exit_counter);
    {
        let sub: &Playing = p.get_state();
        assert_eq!(1, sub.get_context().song2_entry_counter);
        assert_eq!(1, sub.get_context().song2_exit_counter);
    }

    p.process_event(PlayerEvents::EndPause(EndPause)).unwrap();
    {
        let sub: &Playing = p.get_state();
        assert_eq!(PlayingStates::Song2, sub.get_current_state());
        assert_eq!(2, sub.get_context().song2_entry_counter);
    }
    assert_eq!(PlayerStates::Playing, p.get_current_state());
    assert_eq!(1, p.get_context().action_paused_exit_counter);
    assert_eq!(2, p.get_context().playing_fsm_entry_counter);

    p.process_event(PlayerEvents::Pause(Pause)).unwrap();
    assert_eq!(PlayerStates::Paused, p.get_current_state());
    assert_eq!(2, p.get_context().playing_fsm_exit_counter);
    assert_eq!(2, p.get_context().action_paused_entry_counter);

    p.process_event(PlayerEvents::Stop(Stop)).unwrap();
    assert_eq!(PlayerStates::Stopped, p.get_current_state());
    assert_eq!(2, p.get_context().action_paused_exit_counter);
    assert_eq!(2, p.get_context().action_stopped_entry_counter);

    p.process_event(PlayerEvents::Stop(Stop)).unwrap();
    assert_eq!(PlayerStates::Stopped, p.get_current_state());
    assert_eq!(2, p.get_context().action_stopped_exit_counter);
    assert_eq!(3, p.get_context().action_stopped_entry_counter);

    
    p.process_event(PlayerEvents::Play(Play)).unwrap();
    assert_eq!(PlayerStates::Playing, p.get_current_state());
    {
        let sub: &Playing = p.get_state();
        assert_eq!(PlayingStates::Song1, sub.get_current_state());
        assert_eq!(2, sub.get_context().song1_entry_counter);
    }
    
}

