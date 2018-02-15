extern crate syn;

use fsm_def::*;

use quote::*;
use proc_macro2::Span;

pub fn build_fsm_info(fsm: &FsmDescription) -> Tokens {
    let span = Span::call_site();
    let fsm_ty = fsm.get_fsm_ty();
    let fsm_name = &fsm.name;
    let impl_suffix = fsm.get_impl_suffix();
    let fsm_where_ty = fsm.get_fsm_where_ty();

    let mut regions = vec![];
    for region in &fsm.regions {
        let mut states = vec![];
        let mut state_types = vec![];
        for state in &region.get_all_internal_states() {
            let state_name = syn::LitStr::new(&syn_to_string(state), ::quote::__rt::Span::def_site());
            let is_initial_state = state == &region.initial_state_ty;
            let is_interrupt_state = region.interrupt_states.iter().any(|x| &x.interrupt_state_ty == state);

            states.push(quote_spanned! { span =>
                ::fsm::FsmInfoState {
                    state_name: #state_name,
                    is_initial_state: #is_initial_state,
                    is_interrupt_state: #is_interrupt_state
                }
            });
            state_types.push(state.clone());
        }
        let n = states.len();
        

        let mut transitions = vec![];
        for transition in &region.transitions {
            let id = transition.id;
            let s_from = syn_to_string(&transition.source_state);
            let s_to = syn_to_string(&transition.target_state);
            let event = syn_to_string(&transition.event);
            let action = syn_to_string(&transition.action);
            let guard = transition.guard.clone().map(|g| syn_to_string(&g)).unwrap_or("".to_string());
            let transition_type: syn::Path = syn::parse_str(match transition.transition_type {
                TransitionType::Normal => "FsmInfoTransitionType::Normal",
                TransitionType::SelfTransition => "FsmInfoTransitionType::SelfTransition",
                TransitionType::Internal => "FsmInfoTransitionType::Internal"
            }).expect("Error parsing transition type");

            let is_shallow_history = fsm.shallow_history_events.iter().find(|ref x| &x.event_ty == &transition.event && &x.target_state_ty == &transition.target_state).is_some();
            let is_resume_event = region.interrupt_states.iter().any(|x| &x.interrupt_state_ty == &transition.source_state && x.resume_event_ty.iter().any(|y| y == &transition.event));
            let is_internal = transition.transition_type == TransitionType::Internal;
            let is_anonymous = transition.is_anonymous_transition();

            let state_from_is_submachine = fsm.is_submachine(&transition.source_state);
            let state_to_is_submachine = fsm.is_submachine(&transition.target_state);
            //let state_from_idx: usize = state_types.iter().position(|t| t == &transition.source_state).expect("State from not found");
            //let state_to_idx: usize = state_types.iter().position(|t| t == &transition.target_state).expect("State to not found");

            transitions.push(quote_spanned! { span =>
                ::fsm::FsmInfoTransition {
                    transition_id: ::fsm::TransitionId::Table(#id),

                    state_from: #s_from,
                    state_from_is_submachine: #state_from_is_submachine,
                    state_to: #s_to,
                    state_to_is_submachine: #state_to_is_submachine,
                    
                    event: #event,
                    action: #action,
                    guard: #guard,
                    transition_type: #transition_type,

                    is_shallow_history: #is_shallow_history,
                    is_resume_event: #is_resume_event,
                    is_internal: #is_internal,
                    is_anonymous: #is_anonymous
                }
            });
        }
        
        let region_name: syn::LitStr = syn::LitStr::new(&region.id.to_string(), ::quote::__rt::Span::def_site());
        let initial_state = syn::LitStr::new(&syn_to_string(&region.initial_state_ty), ::quote::__rt::Span::def_site());

        regions.push(quote_spanned! { span =>
            ::fsm::FsmInfoRegion {
                region_name: #region_name,
                initial_state: #initial_state,
                states: vec![#(#states),*],
                transitions: vec![#(#transitions),*]
            }
        });
    }
    
    
    quote_spanned! { span =>
        impl #impl_suffix ::fsm::FsmInfo for #fsm_ty #fsm_where_ty {
            fn fsm_info_regions() -> Vec<::fsm::FsmInfoRegion> {
                vec![ #(#regions),* ]
            }

            fn fsm_name() -> &'static str {
                 (#fsm_name)
            }
        }
    }
}
