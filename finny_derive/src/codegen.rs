use proc_macro2::{TokenStream};
use quote::{TokenStreamExt, quote};
use crate::utils::remap_closure_inputs;

use crate::{parse::{EventGuardAction, FsmFnInput, FsmStateAction, FsmStateTransition, FsmTransitionState, FsmTransitionType}, utils::{ty_append}};



pub fn generate_fsm_code(fsm: &FsmFnInput, attr: TokenStream, input: TokenStream) -> syn::Result<TokenStream> {
    let fsm_ty = &fsm.base.fsm_ty;
    let ctx_ty = &fsm.base.context_ty;

    let states_store_ty = ty_append(&fsm.base.fsm_ty, "States");
    let states_enum_ty = ty_append(&fsm.base.fsm_ty, "CurrentState");
    let event_enum_ty = ty_append(&fsm.base.fsm_ty, "Events");

    let region_count = fsm.fsm.regions.len();

    let (fsm_generics_impl, fsm_generics_type, fsm_generics_where) = fsm.base.fsm_generics.split_for_impl();

    let states_store = {

        let mut code_fields = TokenStream::new();
        let mut new_state_fields = TokenStream::new();
        let mut state_variants = TokenStream::new();
        let mut state_accessors = TokenStream::new();

        for (i, (_, state)) in fsm.fsm.states.iter().enumerate() {
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

                impl core::convert::AsMut<#ty> for #states_store_ty {
                    fn as_mut(&mut self) -> &mut #ty {
                        &mut self. #name
                    }
                }
            });
        }

        let mut transition_states = TokenStream::new();

        for region in &fsm.fsm.regions {
            for transition in &region.transitions {
                match transition.ty {
                    FsmTransitionType::StateTransition(ref s) => {
                        match (s.state_from.get_fsm_state(), s.state_to.get_fsm_state()) {
                            (Ok(state_from), Ok(state_to)) => {

                                let state_from_ty = &state_from.ty;
                                let state_to_ty = &state_to.ty;

                                let state_from_field = &state_from.state_storage_field;
                                let state_to_field = &state_to.state_storage_field;

                                transition_states.append_all(quote! {
                                    impl finny::FsmStateTransitionAsMut<#state_from_ty, #state_to_ty> for #states_store_ty {
                                        fn as_state_transition_mut(&mut self) -> (&mut #state_from_ty, &mut #state_to_ty) {
                                            (&mut self. #state_from_field, &mut self. #state_to_field)
                                        }
                                    }
                                });

                            },
                            _ => ()
                        }
                    },
                    _ => ()
                }
            }
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
                type CurrentState = [finny::FsmCurrentState<Self::StateKind>; #region_count];
            }

            #state_accessors

            #transition_states
        }
    };

    let events_enum = {

        let mut variants = TokenStream::new();
        for (ty, _ev) in  fsm.fsm.events.iter() {
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
        
        for region in &fsm.fsm.regions {
            for transition in &region.transitions {

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
                                    fn guard<'a, Q>(event: & #event_ty, context: &finny::EventContext<'a, #fsm_ty #fsm_generics_type, Q>) -> bool
                                        where Q: finny::FsmEventQueue<#fsm_ty #fsm_generics_type>
                                    {
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
                                quote! { event }, quote! { context }, quote! { state }
                            ].as_slice())?;

                            let body = &action.body;                     

                            let state_ty = &state.ty;
                            let body = &action.body;
                            let a = quote! {
                                impl #fsm_generics_impl finny::FsmAction<#fsm_ty #fsm_generics_type, #event_ty, #state_ty, > for #ty #fsm_generics_where {
                                    fn action<'a, Q>(event: & #event_ty , context: &mut finny::EventContext<'a, #fsm_ty #fsm_generics_type, Q >, state: &mut #state_ty)
                                        where Q: finny::FsmEventQueue<#fsm_ty #fsm_generics_type>
                                    {
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
                                    fn guard<'a, Q>(event: & #event_ty, context: &finny::EventContext<'a, #fsm_ty #fsm_generics_type, Q>) -> bool
                                        where Q: finny::FsmEventQueue<#fsm_ty #fsm_generics_type>
                                    {
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
                                    fn action<'a, Q>(event: & #event_ty , context: &mut finny::EventContext<'a, #fsm_ty #fsm_generics_type, Q >, from: &mut #state_from_ty, to: &mut #state_to_ty)
                                        where Q: finny::FsmEventQueue<#fsm_ty #fsm_generics_type>
                                    {
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
        }

        t
    };

    let dispatch = {        
        

        let mut regions = TokenStream::new();
        for region in &fsm.fsm.regions {
            let mut region_transitions = TokenStream::new();

            let region_id = region.region_id;
            for transition in &region.transitions {

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
                            let ty = &st.ty;
                            quote! {
                                <#ty>::execute_on_exit(frontend);
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
                            let ty = &st.ty;
                            quote! {
                                <#ty>::execute_on_entry(frontend);
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
                                        frontend.backend.current_states[#region_id] = finny::FsmCurrentState::None;
                                    }
                                }
                                FsmTransitionState::State(ref st) => {
                                    let kind = &st.ty;
                                    quote! {
                                        frontend.backend.current_states[#region_id] = finny::FsmCurrentState::State(#states_enum_ty :: #kind);
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
                            if <#transition_ty>::execute_guard(frontend, ev)
                        }
                    } else {
                        TokenStream::new()
                    }
                };

                let event_action = {
                    match &transition.ty {
                        FsmTransitionType::InternalTransition(FsmStateAction { action: EventGuardAction { action: Some(_), ..}, .. }) |
                        FsmTransitionType::SelfTransition(FsmStateAction { action: EventGuardAction { action: Some(_), ..}, .. }) => {
                            quote! {
                                <#transition_ty>::execute_action(frontend, ev);
                            }
                        },
                        FsmTransitionType::StateTransition(FsmStateTransition { action: EventGuardAction { action: Some(_), .. }, .. }) => {
                            quote! {
                                <#transition_ty>::execute_action_transition(frontend, ev);
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

                region_transitions.append_all(m);
            }            

            regions.append_all(quote! {
                match (frontend.backend.current_states[#region_id], event) {
                    
                    #region_transitions

                    _ => {
                        transition_misses += 1;
                    }
                }
            });
        }

        quote! {
              
            impl #fsm_generics_impl finny::FsmBackend for #fsm_ty #fsm_generics_type
                #fsm_generics_where
            {
                type Context = #ctx_ty;
                type States = #states_store_ty;
                type Events = #event_enum_ty;

                fn dispatch_event<Q>(frontend: &mut finny::FsmFrontend<Self, Q>, event: &finny::FsmEvent<Self::Events>) -> finny::FsmResult<()>
                    where Q: finny::FsmEventQueue<Self>
                {
                    use finny::{FsmTransitionGuard, FsmTransitionAction, FsmAction, FsmState};

                    let mut transition_misses = 0;
                    
                    #regions

                    if transition_misses == #region_count {
                        return Err(finny::FsmError::NoTransition);
                    }

                    Ok(())
                }
            }
        }
    };

    let states = {

        let mut states = TokenStream::new();
        for (ty, state) in fsm.fsm.states.iter() {

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
                    fn on_entry<'a, Q: finny::FsmEventQueue<#fsm_ty #fsm_generics_type>>(&mut self, context: &mut finny::EventContext<'a, #fsm_ty #fsm_generics_type, Q>) {
                        #on_entry
                    }

                    fn on_exit<'a, Q: finny::FsmEventQueue<#fsm_ty #fsm_generics_type>>(&mut self, context: &mut finny::EventContext<'a, #fsm_ty #fsm_generics_type, Q>) {
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