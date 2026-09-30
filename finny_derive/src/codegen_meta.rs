//! Generates the runtime description of the FSM, `finny::meta::FsmMeta`.

use proc_macro2::TokenStream;
use quote::quote;

use crate::{parse::{FsmFnInput, FsmState, FsmStateKind, FsmTransitionEvent, FsmTransitionState, FsmTransitionType}, utils::{strip_generics, tokens_to_string}};

fn ty_to_string(ty: &syn::Type) -> String {
    tokens_to_string(&strip_generics(ty.clone()))
}

fn vec_of(items: Vec<TokenStream>) -> TokenStream {
    quote! { finny::bundled::alloc::vec::Vec::from([ #(#items),* ]) }
}

fn state_ref(s: &FsmTransitionState) -> TokenStream {
    match s {
        FsmTransitionState::None => quote! { None },
        FsmTransitionState::State(s) => {
            let id = ty_to_string(&s.ty);
            quote! { Some(#id.into()) }
        }
    }
}

fn state_id(s: &FsmTransitionState) -> String {
    match s {
        FsmTransitionState::None => "Stopped".into(),
        FsmTransitionState::State(s) => ty_to_string(&s.ty)
    }
}

fn state_info(state: &FsmState, fsm: &FsmFnInput) -> TokenStream {
    let (_, fsm_generics_type, _) = fsm.base.fsm_generics.split_for_impl();
    let ty = &state.ty;
    let id = ty_to_string(ty);
    let storage_field = state.state_storage_field.to_string();

    let timers = vec_of(state.timers.iter().map(|t| {
        let timer_ty = t.get_ty(&fsm.base);
        let timer_id = ty_to_string(&timer_ty);
        quote! {
            finny::meta::TimerInfo {
                id: #timer_id.into(),
                type_name: core::any::type_name::< #timer_ty #fsm_generics_type >().into()
            }
        }
    }).collect());

    let sub_machine = match state.kind {
        FsmStateKind::Normal => quote! { None },
        FsmStateKind::SubMachine(_) => quote! {
            Some(finny::bundled::alloc::boxed::Box::new(< #ty as finny::meta::FsmMeta >::fsm_info()))
        }
    };

    quote! {
        finny::meta::StateInfo {
            id: #id.into(),
            type_name: core::any::type_name::< #ty >().into(),
            storage_field: #storage_field.into(),
            timers: #timers,
            sub_machine: #sub_machine
        }
    }
}

fn generate_fsm_info(fsm: &FsmFnInput) -> TokenStream {
    let fsm_ty = &fsm.base.fsm_ty;
    let fsm_id = ty_to_string(fsm_ty);
    let context_ty = &fsm.base.context_ty;
    let (fsm_generics_impl, fsm_generics_type, fsm_generics_where) = fsm.base.fsm_generics.split_for_impl();

    let regions = vec_of(fsm.fsm.regions.iter().map(|region| {
        let region_id = region.region_id;
        let initial_state = ty_to_string(&region.initial_state);
        let states = vec_of(region.states.iter().map(|s| state_info(s, fsm)).collect());

        let transitions = vec_of(region.transitions.iter().map(|transition| {
            let transition_ty = &transition.transition_ty;
            let transition_id = ty_to_string(transition_ty);

            let (event, kind, action) = match &transition.ty {
                FsmTransitionType::InternalTransition(s) => {
                    let state = state_id(&s.state);
                    (&s.event, quote! { finny::meta::TransitionKindInfo::Internal { state: #state.into() } }, &s.action)
                },
                FsmTransitionType::SelfTransition(s) => {
                    let state = state_id(&s.state);
                    (&s.event, quote! { finny::meta::TransitionKindInfo::SelfTransition { state: #state.into() } }, &s.action)
                },
                FsmTransitionType::StateTransition(s) => {
                    let (from, to) = (state_ref(&s.state_from), state_ref(&s.state_to));
                    (&s.event, quote! { finny::meta::TransitionKindInfo::Normal { from: #from, to: #to } }, &s.action)
                }
            };

            let event = match event {
                FsmTransitionEvent::Start => quote! { finny::meta::EventInfo::Start },
                FsmTransitionEvent::Stop => quote! { finny::meta::EventInfo::Stop },
                FsmTransitionEvent::Event(ev) => {
                    let ev_ty = &ev.ty;
                    let ev_id = ty_to_string(ev_ty);
                    quote! {
                        finny::meta::EventInfo::Event {
                            id: #ev_id.into(),
                            type_name: core::any::type_name::< #ev_ty >().into()
                        }
                    }
                }
            };

            let has_guard = action.guard.is_some();
            let has_action = action.action.is_some();

            quote! {
                finny::meta::TransitionInfo {
                    id: #transition_id.into(),
                    type_name: core::any::type_name::< #transition_ty >().into(),
                    event: #event,
                    kind: #kind,
                    has_guard: #has_guard,
                    has_action: #has_action
                }
            }
        }).collect());

        quote! {
            finny::meta::RegionInfo {
                region_id: #region_id,
                initial_state: #initial_state.into(),
                states: #states,
                transitions: #transitions
            }
        }
    }).collect());

    quote! {
        impl #fsm_generics_impl finny::meta::FsmMeta for #fsm_ty #fsm_generics_type #fsm_generics_where {
            fn fsm_info() -> finny::meta::FsmInfo {
                finny::meta::FsmInfo {
                    id: #fsm_id.into(),
                    type_name: core::any::type_name::<Self>().into(),
                    context_type_name: core::any::type_name::< #context_ty >().into(),
                    regions: #regions
                }
            }
        }
    }
}

/// Writes the PlantUML diagram of the FSM into a file when running the crate's tests.
#[cfg(feature = "generate_plantuml")]
fn generate_plantuml_test(fsm: &FsmFnInput) -> TokenStream {
    let fsm_ty = &fsm.base.fsm_ty;
    // The generic FSMs can't be described without their type arguments.
    if !fsm.base.fsm_generics.params.is_empty() {
        return TokenStream::new();
    }

    let fsm_ty_name_snake = crate::utils::to_snake_case(&tokens_to_string(fsm_ty));
    let test_fn_name = crate::utils::to_field_name(&crate::utils::ty_append(fsm_ty, "_plantuml"));

    quote! {
        #[test]
        #[cfg(test)]
        fn #test_fn_name () {
            let contents = finny::meta::plantuml::to_plantuml(&< #fsm_ty as finny::meta::FsmMeta >::fsm_info());
            std::fs::write(&format!("{}.plantuml", #fsm_ty_name_snake), contents).unwrap();
        }
    }
}

pub fn generate_fsm_meta(fsm: &FsmFnInput) -> TokenStream {
    if !cfg!(feature = "meta") {
        return TokenStream::new();
    }

    let info = generate_fsm_info(fsm);

    #[cfg(feature = "generate_plantuml")]
    let plantuml = generate_plantuml_test(fsm);
    #[cfg(not(feature = "generate_plantuml"))]
    let plantuml = TokenStream::new();

    quote! {
        #info
        #plantuml
    }
}
