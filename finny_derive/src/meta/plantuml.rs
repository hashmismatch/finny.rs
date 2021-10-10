use proc_macro2:: TokenStream;
use quote::{quote, TokenStreamExt};
use syn::{PathSegment, TypePath, parse::Parse};

use super::FinnyFsm;
use std::fmt::Write;


pub fn to_plant_uml(fsm: &FinnyFsm) -> Result<(String, TokenStream), std::fmt::Error> {
    let mut output = String::new();
    let mut subs = TokenStream::new();

    for region in fsm.regions.values() {
        for state in region.states.values() {
            match state {
                super::FinnyStateKind::Stopped => {

                }
                super::FinnyStateKind::State(state) => {

                    writeln!(&mut output, "state {} {{", state.state_id)?;
                    writeln!(&mut output, "}}")?;

                    for timer in &state.timers {
                        writeln!(&mut output, "state {} : Timer {}", state.state_id, timer.timer_id)?;
                    }
                },
                super::FinnyStateKind::SubMachine(sub_id) => {
                    
                    let p = syn::parse_str::<syn::Type>(&format!("{}Info", sub_id)).unwrap();
                    

                    subs.append_all(quote! {
                        writeln!(&mut output, "state {} {{", #sub_id);
                        writeln!(&mut output, "{}", < #p > :: plantuml_inner() );
                        writeln!(&mut output, "}}");
                    });

                }
            }
        }

        for transition in region.transitions.values() {

            let event = match transition.event {
                super::FinnyEvent::Start => "Start".to_string(),
                super::FinnyEvent::Stop => "Stop".to_string(),
                super::FinnyEvent::Event(ref ev) => ev.clone()
            };

            match &transition.transition {
                super::FinnyTransitionKind::SelfTransition { state_id } => {
                    writeln!(&mut output, "{state} --> {state} : {event} (Self)", state = state_id, event = event)?;
                    writeln!(&mut output, "note on link: {}", transition.transition_id)?;
                }
                super::FinnyTransitionKind::InternalTransition { state_id } => {
                    writeln!(&mut output, "{state} --> {state} : {event} (Internal)", state = state_id, event = event)?;
                    writeln!(&mut output, "note on link: {}", transition.transition_id)?;
                }
                super::FinnyTransitionKind::NormalTransition(t) => {
                    let state_from = match t.from_state.as_str() {
                        "Stopped" => "[*]",
                        _ => &t.from_state
                    };

                    writeln!(&mut output, "{state_from} --> {state_to} : {event}", state_from = state_from, state_to = t.to_state, event = event)?;
                    writeln!(&mut output, "note on link: {}", transition.transition_id)?;
                }
            }
        }
    }

    Ok((output, subs))
}