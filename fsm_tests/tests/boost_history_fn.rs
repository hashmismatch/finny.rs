#![feature(proc_macro)]

extern crate fsm;
#[macro_use]
extern crate fsm_codegen;


extern crate serde;
#[macro_use]
extern crate serde_derive;


use fsm::*;
use fsm::declaration::*;
use fsm_codegen::fsm_fn;

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

#[fsm_fn]
fn fsm_player() {
    let fsm = FsmDecl::new_fsm::<Player>()
        .context_ty::<PlayerContext>()
        .initial_state::<Empty>();
    
    fsm.new_unit_event::<Play>();
    fsm.new_unit_event::<EndPause>();
    fsm.new_unit_event::<Stop>();
    fsm.new_unit_event::<Pause>();
    fsm.new_unit_event::<OpenClose>();

    fsm.on_event::<EndPause>()
        .shallow_history::<Playing>();
    
    #[derive(Clone, PartialEq, Debug, Serialize)]
    pub struct CdDetected { name: String }
    fsm.new_event::<CdDetected>();

    fsm.new_unit_state::<Empty>()
        .on_entry(|state, ctx| {
            ctx.context.action_empty_entry_counter += 1;
        })
        .on_exit(|state, ctx| {
            ctx.context.action_empty_exit_counter += 1;
        });

    fsm.new_unit_state::<Open>()
        .on_entry(|state, ctx| {
            ctx.context.action_open_entry_counter += 1;
        })
        .on_exit(|state, ctx| {
            ctx.context.action_open_exit_counter += 1;
        });

    fsm.new_unit_state::<Stopped>()
        .on_entry(|state, ctx| {
            ctx.context.action_stopped_entry_counter += 1;
        })
        .on_exit(|state, ctx| {
            ctx.context.action_stopped_exit_counter += 1;
        });

    fsm.new_unit_state::<Paused>()
        .on_entry(|state, ctx| {
            ctx.context.action_paused_entry_counter += 1;
        })
        .on_exit(|state, ctx| {
            ctx.context.action_paused_exit_counter += 1;
        });

    fsm.add_sub_machine::<Playing>()
        .on_entry(|state, ctx| {
            ctx.context.playing_fsm_entry_counter += 1;
        })
        .on_exit(|state, ctx| {
            ctx.context.playing_fsm_exit_counter += 1;
        });

    fsm.on_event::<Play>()
        .transition_from::<Stopped>()
        .to::<Playing>()
        .action(|event, ctx, stopped, playing| {
            ctx.context.start_playback_counter += 1;
        });

    fsm.on_event::<OpenClose>()
        .transition_from::<Stopped>()
        .to::<Open>();

    fsm.on_event::<Stop>()
        .transition_self::<Stopped>();

    
    fsm.on_event::<OpenClose>()
        .transition_from::<Open>()
        .to::<Empty>();

    fsm.on_event::<OpenClose>()
        .transition_from::<Empty>()
        .to::<Open>();                

    fsm.on_event::<CdDetected>()
        .transition_from::<Empty>()
        .to::<Stopped>();


    // playing transitions
    fsm.on_event::<Stop>()
        .transition_from::<Playing>()
        .to::<Stopped>();
    fsm.on_event::<Pause>()
        .transition_from::<Playing>()
        .to::<Paused>();
    fsm.on_event::<OpenClose>()
        .transition_from::<Playing>()
        .to::<Open>();

    fsm.on_event::<EndPause>()
        .transition_from::<Paused>()
        .to::<Playing>();
    fsm.on_event::<Stop>()
        .transition_from::<Paused>()
        .to::<Stopped>();
    fsm.on_event::<OpenClose>()
        .transition_from::<Paused>()
        .to::<Open>();
}


#[derive(Default, Debug, Serialize)]
pub struct PlayingContext {
    song1_entry_counter: usize,
    song1_exit_counter: usize,

    song2_entry_counter: usize,
    song2_exit_counter: usize,
}

#[fsm_fn]
fn fsm_playing() {
    let fsm = FsmDecl::new_fsm::<Playing>()
        .context_ty::<PlayingContext>()
        .initial_state::<Song1>();

    fsm.new_unit_event::<NextSong>();
    fsm.new_unit_event::<PreviousSong>();

    fsm.new_unit_state::<Song1>()
        .on_entry(|state, ctx| {
            ctx.context.song1_entry_counter += 1;
        })
        .on_exit(|state, ctx| {
            ctx.context.song1_exit_counter += 1;
        });

    fsm.new_unit_state::<Song2>()
        .on_entry(|state, ctx| {
            ctx.context.song2_entry_counter += 1;
        })
        .on_exit(|state, ctx| {
            ctx.context.song2_exit_counter += 1;
        });

    fsm.new_unit_state::<Song3>();

    fsm.on_event::<NextSong>()
        .transition_from::<Song1>()
        .to::<Song2>();

    fsm.on_event::<NextSong>()
        .transition_from::<Song2>()
        .to::<Song3>();

    fsm.on_event::<PreviousSong>()
        .transition_from::<Song3>()
        .to::<Song2>();
    
    fsm.on_event::<PreviousSong>()
        .transition_from::<Song2>()
        .to::<Song1>();    
}



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
