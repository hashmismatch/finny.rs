use proc_macro2::{Span, TokenStream};
use quote::{TokenStreamExt, quote};
use syn::spanned::Spanned;
use utils::remap_closure_inputs;

use crate::{parse::{EventGuardAction, FsmEvent, FsmFnInput, FsmStateAction, FsmStateTransition, FsmTransitionState, FsmTransitionType}, utils::{tokens_to_string, ty_append}};



pub fn generate_fsm_code(fsm: &FsmFnInput, attr: TokenStream, input: TokenStream) -> syn::Result<TokenStream> {
    let fsm_ty = &fsm.base.fsm_ty;
    let ctx_ty = &fsm.base.context_ty;

    let states_store_ty = ty_append(&fsm.base.fsm_ty, "States");
    let states_enum_ty = ty_append(&fsm.base.fsm_ty, "CurrentState");
    let event_enum_ty = ty_append(&fsm.base.fsm_ty, "Events");

    let (fsm_generics_impl, fsm_generics_type, fsm_generics_where) = fsm.base.fsm_generics.split_for_impl();

    let states_store = {

        let mut code_fields = TokenStream::new();
        let mut new_state_fields = TokenStream::new();
        let mut state_variants = TokenStream::new();
        let mut state_accessors = TokenStream::new();

        for (i, (_, state)) in fsm.decl.states.iter().enumerate() {
            let name = &state.state_storage_field;
            let ty = &state.ty;
            code_fields.append_all(quote! { #name: #ty, });
            new_state_fields.append_all(quote! { #name: #ty::new_state(context)?, });
            state_variants.append_all(quote!{ #ty, });

            state_accessors.append_all(quote! {
                impl core::convert::AsRef<#ty> for #states_store_ty {
                    fn as_ref(&self) -> & #ty {
                        &self. #name
                    }
                }
            });
        }

        quote! {
            pub struct #states_store_ty {
                #code_fields
            }

            impl finny::FsmStateFactory for #states_store_ty {
                fn new_state<C>(context: &C) -> finny::FsmResult<Self> {
                    let s = Self {
                        #new_state_fields
                    };
                    Ok(s)
                }
            }

            #[derive(Copy, Clone, Debug, PartialEq)]
            pub enum #states_enum_ty {
                #state_variants
            }

            impl finny::FsmStates for #states_store_ty {
                type StateKind = #states_enum_ty;
            }

            #state_accessors
        }
    };

    let events_enum = {

        let mut variants = TokenStream::new();
        for (ty, ev) in  fsm.decl.events.iter() {
            variants.append_all(quote! { #ty ( #ty ),  });
        }
        
        let evs = quote! {
            #[derive(finny::bundled::derive_more::From)]
            pub enum #event_enum_ty {
                #variants
            }
        };

        evs
    };

    let transition_types = {
        let mut t = TokenStream::new();
        
        for transition in &fsm.decl.transitions {

            let ty = &transition.transition_ty;
            let mut q = quote! {
                pub struct #ty;
            };

            match &transition.ty {
                FsmTransitionType::InternalTransition(s) | FsmTransitionType::SelfTransition(s) => {
                    
                    let state = s.state.get_fsm_state()?;
                    let event_ty = &s.event.get_event()?.ty;

                    if let Some(ref guard) = s.action.guard {
                        let remap = remap_closure_inputs(&guard.inputs, vec![
                            quote! { event }, quote! { context }
                        ].as_slice())?;

                        let body = &guard.body;

                        let g = quote! {
                            impl #fsm_generics_impl finny::FsmTransitionGuard<#fsm_ty #fsm_generics_type, #event_ty> for #ty #fsm_generics_where {
                                fn guard<'a, Q>(event: & #event_ty, context: &finny::EventContext<'a, #fsm_ty #fsm_generics_type, Q>) -> bool {
                                    #remap
                                    let result = { #body };
                                    result
                                }
                            }
                        };

                        q.append_all(g);
                    }

                    if let Some(ref action) = s.action.action {
                        let remap = remap_closure_inputs(&action.inputs, vec![
                            quote! { event }, quote! { context }, quote! { from }, quote! { to }
                        ].as_slice())?;

                        let body = &action.body;                     

                        let state_ty = &state.ty;
                        let body = &action.body;
                        let a = quote! {
                            impl #fsm_generics_impl finny::FsmAction<#fsm_ty #fsm_generics_type, #event_ty, #state_ty, > for #ty #fsm_generics_where {
                                fn action<'a, Q>(event: & #event_ty , context: &mut finny::EventContext<'a, #fsm_ty #fsm_generics_type, Q >, state: &mut #state_ty) {
                                    #remap
                                    { #body }
                                }
                            }
                        };

                        q.append_all(a);
                    }
                },
                FsmTransitionType::StateTransition(s) => {

                    if let Some(ref guard) = s.action.guard {
                        let event_ty = &s.event.get_event()?.ty;

                        let remap = remap_closure_inputs(&guard.inputs, vec![
                            quote! { event }, quote! { context }
                        ].as_slice())?;

                        let body = &guard.body;

                        let g = quote! {
                            impl #fsm_generics_impl finny::FsmTransitionGuard<#fsm_ty #fsm_generics_type, #event_ty> for #ty #fsm_generics_where {
                                fn guard<'a, Q>(event: & #event_ty, context: &finny::EventContext<'a, #fsm_ty #fsm_generics_type, Q>) -> bool {
                                    #remap
                                    let result = { #body };
                                    result
                                }
                            }
                        };

                        q.append_all(g);
                    }

                    if let Some(ref action) = s.action.action {
                        let event_ty = &s.event.get_event()?.ty;
                        let state_from = s.state_from.get_fsm_state()?;
                        let state_to = s.state_to.get_fsm_state()?;                    

                        let remap = remap_closure_inputs(&action.inputs, vec![
                            quote! { event }, quote! { context }, quote! { from }, quote! { to }
                        ].as_slice())?;

                        let body = &action.body;                     

                        let state_from_ty = &state_from.ty;
                        let state_to_ty = &state_to.ty;
                        let body = &action.body;
                        let a = quote! {
                            impl #fsm_generics_impl finny::FsmTransitionAction<#fsm_ty #fsm_generics_type, #event_ty, #state_from_ty, #state_to_ty> for #ty #fsm_generics_where {
                                fn action<'a, Q>(event: & #event_ty , context: &mut finny::EventContext<'a, #fsm_ty #fsm_generics_type, Q >, from: &mut #state_from_ty, to: &mut #state_to_ty) {
                                    #remap
                                    { #body }
                                }
                            }
                        };

                        q.append_all(a);
                    }
                }
            }
            
            t.append_all(q);
        }

        t
    };

    let dispatch = {        
        let mut transition_match = TokenStream::new();
        for transition in &fsm.decl.transitions {

            let transition_ty = &transition.transition_ty;
            
            let match_state = {
                let state_from = match &transition.ty {
                    FsmTransitionType::InternalTransition(s) | FsmTransitionType::SelfTransition(s) => {
                        &s.state
                    }
                    FsmTransitionType::StateTransition(s) => &s.state_from
                };

                match state_from {
                    FsmTransitionState::None => quote! { finny::FsmCurrentState::Stopped },
                    FsmTransitionState::State(st) => {
                        let kind = &st.ty;
                        quote! { finny::FsmCurrentState::State(#states_enum_ty :: #kind) }
                    }
                }
            };
            
            let match_event = {                
                let event = match &transition.ty {
                    FsmTransitionType::InternalTransition(s) | FsmTransitionType::SelfTransition(s) => &s.event,
                    FsmTransitionType::StateTransition(s) => &s.event
                };

                match event {
                    crate::parse::FsmTransitionEvent::Start => quote! { finny::FsmEvent::Start },
                    crate::parse::FsmTransitionEvent::Stop => quote ! { finny::FsmEvent::Stop },
                    crate::parse::FsmTransitionEvent::Event(ref ev) => {
                        let kind = &ev.ty;
                        quote! { finny::FsmEvent::Event(#event_enum_ty::#kind(ref ev)) }
                    }
                }
            };

            let state_from_action = {
                let state = match &transition.ty {                    
                    FsmTransitionType::SelfTransition(s) => Some(&s.state),
                    FsmTransitionType::StateTransition(s) => Some(&s.state_from),
                    FsmTransitionType::InternalTransition(_) => None
                };

                match state {
                    Some(FsmTransitionState::State(st)) => {
                        let field = &st.state_storage_field;
                        let ty = &st.ty;
                        quote! {
                            let state_from = &mut backend.states. #field ;
                            <#ty>::on_exit(state_from, &mut context);
                        }
                    },
                    _ => TokenStream::new()
                }
            };

            let state_to_action = {
                let state = match &transition.ty {                    
                    FsmTransitionType::SelfTransition(s) => Some(&s.state),
                    FsmTransitionType::StateTransition(s) => Some(&s.state_to),
                    FsmTransitionType::InternalTransition(_) => None
                };

                match state {
                    Some(FsmTransitionState::State(st)) => {
                        let field = &st.state_storage_field;
                        let ty = &st.ty;
                        quote! {
                            let state_to = &mut backend.states. #field ;
                            <#ty>::on_entry(state_to, &mut context);
                        }
                    },
                    _ => TokenStream::new()
                }
            };

            let current_state_update = {
                match &transition.ty {
                    FsmTransitionType::InternalTransition(_) | FsmTransitionType::SelfTransition(_) => TokenStream::new(),
                    FsmTransitionType::StateTransition(ref s) => {
                        match s.state_to {
                            FsmTransitionState::None => {
                                quote! {
                                    backend.current_state = finny::FsmCurrentState::None;
                                }
                            }
                            FsmTransitionState::State(ref st) => {
                                let kind = &st.ty;
                                quote! {
                                    backend.current_state = finny::FsmCurrentState::State(#states_enum_ty :: #kind);
                                }
                            }
                        }
                    }
                }
            };
            
            let guard = {
                let has_guard = match &transition.ty {
                    FsmTransitionType::StateTransition(s) => {
                        s.action.guard.is_some()
                    }
                    FsmTransitionType::InternalTransition(s) | FsmTransitionType::SelfTransition(s) => {
                        s.action.guard.is_some()
                    }
                };

                if has_guard {
                    quote! {
                        if <#transition_ty>::guard(ev, &context)
                    }
                } else {
                    TokenStream::new()
                }
            };

            let event_action = {
                match &transition.ty {
                    FsmTransitionType::InternalTransition(FsmStateAction { action: EventGuardAction { action: Some(_), ..}, state, .. }) |
                    FsmTransitionType::SelfTransition(FsmStateAction { action: EventGuardAction { action: Some(_), ..}, state, .. }) => {
                        panic!("todo ev action");
                    },
                    FsmTransitionType::StateTransition(FsmStateTransition { action: EventGuardAction { action: Some(_), .. }, state_from, state_to, .. }) => {
                        let state_from = state_from.get_fsm_state()?;
                        let state_to = state_to.get_fsm_state()?;

                        let state_from_field = &state_from.state_storage_field;
                        let state_to_field = &state_to.state_storage_field;

                        quote! {
                            <#transition_ty>::action(ev, &mut context,
                                &mut backend.states. #state_from_field,
                                &mut backend.states. #state_to_field
                            );
                        }
                    },
                    _ => TokenStream::new()
                }
            };
            
            let m = quote! {
                ( #match_state , #match_event ) #guard => {
                    { #state_from_action }

                    { #event_action }

                    { #state_to_action }

                    { #current_state_update }
                },
            };

            transition_match.append_all(m);
        }

        quote! {
              
            impl #fsm_generics_impl finny::FsmBackend for #fsm_ty #fsm_generics_type
                #fsm_generics_where
            {
                type Context = #ctx_ty;
                type States = #states_store_ty;
                type Events = #event_enum_ty;

                fn dispatch_event<Q>(backend: &mut finny::FsmBackendImpl<Self>, event: &finny::FsmEvent<Self::Events>, queue: &mut Q) -> finny::FsmResult<()>
                    where Q: finny::FsmEventQueue<<Self as finny::FsmBackend>::Events>
                {
                    use finny::{FsmTransitionGuard, FsmTransitionAction, FsmState};

                    let mut context = finny::EventContext::<#fsm_ty #fsm_generics_type, Q> {
                        context: &mut backend.context,
                        queue
                    };
                    
                    match (&backend.current_state, event) {
                        
                        #transition_match

                        _ => {
                            return Err(finny::FsmError::NoTransition);
                        }
                    }

                    Ok(())
                }
            }
        }
    };

    let states = {

        let mut states = TokenStream::new();
        for (ty, state) in fsm.decl.states.iter() {

            let remap_closure = |c: &Option<syn::ExprClosure>| -> syn::Result<TokenStream> {
                if let Some(c) = &c {
                    let remap = remap_closure_inputs(&c.inputs, &vec![ quote! { self }, quote! { context } ])?;
                    let b = &c.body;
                    
                    let q = quote! {                                        
                        #remap
                        { #b }
                    };
                    Ok(q)
                } else {
                    Ok(TokenStream::new())
                }
            };

            let on_entry = remap_closure(&state.on_entry_closure)?;
            let on_exit = remap_closure(&state.on_exit_closure)?;

            states.append_all(quote! {

                impl #fsm_generics_impl finny::FsmState<#fsm_ty #fsm_generics_type> for #ty #fsm_generics_where {
                    fn on_entry<'a, Q: finny::FsmEventQueue<<#fsm_ty #fsm_generics_type as finny::FsmBackend>::Events>>(&mut self, context: &mut finny::EventContext<'a, #fsm_ty #fsm_generics_type, Q>) {
                        #on_entry
                    }

                    fn on_exit<'a, Q: finny::FsmEventQueue<<#fsm_ty #fsm_generics_type as finny::FsmBackend>::Events>>(&mut self, context: &mut finny::EventContext<'a, #fsm_ty #fsm_generics_type, Q>) {
                        #on_exit
                    }
                }

            })
        }

        states
    };

    let builder = {

        quote! {

            pub struct #fsm_ty #fsm_generics_type #fsm_generics_where {
                backend: finny::FsmBackendImpl<#fsm_ty #fsm_generics_type >
            }

            impl #fsm_generics_impl finny::FsmFactory for #fsm_ty #fsm_generics_type #fsm_generics_where {
                type Fsm = #fsm_ty #fsm_generics_type;
            }
            
        }
    };

    let mut q = quote! {
        #states_store

        #states

        #events_enum

        #transition_types

        #dispatch

        #builder
    };

    // this goes in front of our definition function
    q.append_all(quote! {
        #[allow(dead_code)]
    });

    q.append_all(attr);
    q.append_all(input);

    Ok(q.into())
}