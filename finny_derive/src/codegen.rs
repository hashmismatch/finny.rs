use proc_macro2::{TokenStream};
use quote::{TokenStreamExt, quote};
use crate::{fsm::FsmTypes, parse::{FsmState, FsmStateKind}, utils::{remap_closure_inputs}};

use crate::{parse::{FsmFnInput, FsmStateTransition, FsmTransitionState, FsmTransitionType}, utils::ty_append};

pub fn generate_fsm_code(fsm: &FsmFnInput, attr: TokenStream, input: TokenStream) -> syn::Result<TokenStream> {
    let fsm_ty = &fsm.base.fsm_ty;
    let fsm_types = FsmTypes::new(&fsm.base.fsm_ty, &fsm.base.fsm_generics);
    //let fsm_mod = to_field_name(&ty_append(fsm_ty, "Finny"))?;
    let ctx_ty = &fsm.base.context_ty;

    let states_store_ty = ty_append(&fsm.base.fsm_ty, "States");
    let states_enum_ty = ty_append(&fsm.base.fsm_ty, "CurrentState");
    let event_enum_ty = fsm_types.get_fsm_events_ty();

    let region_count = fsm.fsm.regions.len();

    let (fsm_generics_impl, fsm_generics_type, fsm_generics_where) = fsm.base.fsm_generics.split_for_impl();
    
    let states_store = {

        let mut code_fields = TokenStream::new();
        let mut new_state_fields = TokenStream::new();
        let mut state_variants = TokenStream::new();
        let mut state_accessors = TokenStream::new();

        for (i, (_, state)) in fsm.fsm.states.iter().enumerate() {
            let name = &state.state_storage_field;
            let state_ty = FsmTypes::new(&state.ty,&fsm.base.fsm_generics);
            let ty = state_ty.get_fsm_ty();
            let ty_name = state_ty.get_fsm_no_generics_ty();

            code_fields.append_all(quote! { #name: #ty, });
            state_variants.append_all(quote!{ #ty_name, });

            let new_state_field = match state.kind {
                FsmStateKind::Normal => {
                    quote! {
                        #name: < #ty as finny::FsmStateFactory< #fsm_ty #fsm_generics_type > >::new_state(context)?,
                    }
                }
                FsmStateKind::SubMachine(ref sub) => {

                    let ctx_codegen = match &sub.context_constructor {
                        Some(c) => {
                            let remap = remap_closure_inputs(&c.inputs, &[quote!{ context }])?;
                            let body = &c.body;
                            quote! {
                                #remap
                                {
                                    #body
                                }
                            }
                        },
                        None => {
                            quote! {
                                Default::default()
                            }
                        }
                    };

                    quote! {
                        #name: {
                            use finny::{FsmFactory};
                            
                            let sub_ctx = {
                                #ctx_codegen
                            };
                            let fsm_backend = finny::FsmBackendImpl::<#ty>::new(sub_ctx)?;
                            let fsm = <#ty>::new_submachine_backend(fsm_backend)?;
                            fsm
                        },
                    }
                }
            };
            new_state_fields.append_all(new_state_field);

            state_accessors.append_all(quote! {
                impl #fsm_generics_impl core::convert::AsRef<#ty> for #states_store_ty #fsm_generics_type #fsm_generics_where {
                    fn as_ref(&self) -> & #ty {
                        &self. #name
                    }
                }

                impl #fsm_generics_impl core::convert::AsMut<#ty> for #states_store_ty #fsm_generics_type #fsm_generics_where {
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
                                    impl #fsm_generics_impl finny::FsmStateTransitionAsMut<#state_from_ty, #state_to_ty> for #states_store_ty #fsm_generics_type #fsm_generics_where {
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
            pub struct #states_store_ty #fsm_generics_type #fsm_generics_where {
                #code_fields
                _fsm: core::marker::PhantomData< #fsm_ty #fsm_generics_type >
            }
            
            impl #fsm_generics_impl finny::FsmStateFactory< #fsm_ty #fsm_generics_type > for #states_store_ty #fsm_generics_type #fsm_generics_where {
                fn new_state(context: & #ctx_ty ) -> finny::FsmResult<Self> {
                    let s = Self {
                        #new_state_fields
                        _fsm: core::marker::PhantomData::default()
                    };
                    Ok(s)
                }
            }
                        
            #[derive(Copy, Clone, Debug, PartialEq)]
            pub enum #states_enum_ty {
                #state_variants
            }

            impl #fsm_generics_impl finny::FsmStates< #fsm_ty #fsm_generics_type > for #states_store_ty #fsm_generics_type #fsm_generics_where {
                type StateKind = #states_enum_ty;
                type CurrentState = [finny::FsmCurrentState<Self::StateKind>; #region_count];
            }

            #state_accessors

            #transition_states
        }
    };
    

    let events_enum = {

        let submachines: Vec<_> = fsm.fsm.states.iter().filter_map(|(_, state)| {
            match &state.kind {
                FsmStateKind::Normal => None,
                FsmStateKind::SubMachine(ref sub) => {
                    Some((sub, state))
                }
            }
        }).collect();

        let mut variants = TokenStream::new();
        for (ty, _ev) in  fsm.fsm.events.iter() {
            variants.append_all(quote! { #ty ( #ty ),  });
        }
        for (_sub, state) in submachines {
            let sub_fsm = FsmTypes::new(&state.ty, &fsm.base.fsm_generics);
            let sub_fsm_event_ty = sub_fsm.get_fsm_events_ty();
            let sub_fsm_ty = sub_fsm.get_fsm_no_generics_ty();            

            variants.append_all(quote! {
                #sub_fsm_ty ( #sub_fsm_event_ty ),
            })
        }

        let mut derives = TokenStream::new();
        if fsm.fsm.codegen_options.event_debug {
            derives.append_all(quote! {
                #[derive(Debug)]
            });
        }
        
        let evs = quote! {
            #[derive(finny::bundled::derive_more::From, finny::bundled::strum::AsRefStr)]
            #[derive(Clone)]
            #derives
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
                    // internal or self transtion (only the current state)
                    FsmTransitionType::InternalTransition(s) | FsmTransitionType::SelfTransition(s) => {
                        
                        let state = s.state.get_fsm_state()?;
                        let event_ty = &s.event.get_event()?.ty;

                        let is_self_transition = if let FsmTransitionType::SelfTransition(_) = &transition.ty { true } else { false };

                        if let Some(ref guard) = s.action.guard {
                            let remap = remap_closure_inputs(&guard.inputs, vec![
                                quote! { event }, quote! { context }
                            ].as_slice())?;

                            let body = &guard.body;

                            let g = quote! {
                                impl #fsm_generics_impl finny::FsmTransitionGuard<#fsm_ty #fsm_generics_type, #event_ty> for #ty #fsm_generics_where {
                                    fn guard<'fsm_event, Q>(event: & #event_ty, context: &finny::EventContext<'fsm_event, #fsm_ty #fsm_generics_type, Q>) -> bool
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
                        
                        let action_body = if let Some(ref action) = s.action.action {
                            let remap = remap_closure_inputs(&action.inputs, vec![
                                quote! { event }, quote! { context }, quote! { state }
                            ].as_slice())?;

                            let body = &action.body;
                            
                            quote! { 
                                #remap
                                { #body }
                            }
                        } else {
                            TokenStream::new()
                        };

                        let state_ty = &state.ty;
                        q.append_all(quote! {
                            impl #fsm_generics_impl finny::FsmAction<#fsm_ty #fsm_generics_type, #event_ty, #state_ty > for #ty #fsm_generics_where {
                                fn action<'fsm_event, Q>(event: & #event_ty , context: &mut finny::EventContext<'fsm_event, #fsm_ty #fsm_generics_type, Q >, state: &mut #state_ty)
                                    where Q: finny::FsmEventQueue<#fsm_ty #fsm_generics_type>
                                {
                                    #action_body
                                }

                                fn should_trigger_state_actions() -> bool {
                                    #is_self_transition
                                }
                            }
                        });
                    },

                    // fsm start transition
                    FsmTransitionType::StateTransition(s @ FsmStateTransition { state_from: FsmTransitionState::None, .. }) => {
                        let initial_state_ty = &s.state_to.get_fsm_state()?.ty;

                        q.append_all(quote! {
                            impl #fsm_generics_impl finny::FsmTransitionFsmStart<#fsm_ty #fsm_generics_type, #initial_state_ty > for #ty #fsm_generics_where {

                            }
                        });

                    },

                    // normal state transition
                    FsmTransitionType::StateTransition(s) => {

                        if let Some(ref guard) = s.action.guard {
                            let event_ty = &s.event.get_event()?.ty;

                            let remap = remap_closure_inputs(&guard.inputs, vec![
                                quote! { event }, quote! { context }
                            ].as_slice())?;

                            let body = &guard.body;

                            let g = quote! {
                                impl #fsm_generics_impl finny::FsmTransitionGuard<#fsm_ty #fsm_generics_type, #event_ty> for #ty #fsm_generics_where {
                                    fn guard<'fsm_event, Q>(event: & #event_ty, context: &finny::EventContext<'fsm_event, #fsm_ty #fsm_generics_type, Q>) -> bool
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

                        let action_body = if let Some(ref action) = s.action.action {
                            let remap = remap_closure_inputs(&action.inputs, vec![
                                quote! { event }, quote! { context }, quote! { from }, quote! { to }
                            ].as_slice())?;

                            let body = &action.body;

                            quote! {
                                #remap
                                { #body }
                            }
                        } else {
                            TokenStream::new()
                        };

                        let event_ty = &s.event.get_event()?.ty;
                        let state_from = s.state_from.get_fsm_state()?;
                        let state_to = s.state_to.get_fsm_state()?;
                        
                        let state_from_ty = &state_from.ty;
                        let state_to_ty = &state_to.ty;

                        let a = quote! {
                            impl #fsm_generics_impl finny::FsmTransitionAction<#fsm_ty #fsm_generics_type, #event_ty, #state_from_ty, #state_to_ty> for #ty #fsm_generics_where {
                                fn action<'fsm_event, Q>(event: & #event_ty , context: &mut finny::EventContext<'fsm_event, #fsm_ty #fsm_generics_type, Q >, from: &mut #state_from_ty, to: &mut #state_to_ty)
                                    where Q: finny::FsmEventQueue<#fsm_ty #fsm_generics_type>
                                {
                                    #action_body
                                }
                            }
                        };

                        q.append_all(a);
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
                            let state_ty = FsmTypes::new(&st.ty, &fsm.base.fsm_generics);
                            let variant = state_ty.get_fsm_no_generics_ty();
                            quote! { finny::FsmCurrentState::State(#states_enum_ty :: #variant) }
                        }
                    }
                };
                
                let match_event = {                
                    let event = match &transition.ty {
                        FsmTransitionType::InternalTransition(s) | FsmTransitionType::SelfTransition(s) => &s.event,
                        FsmTransitionType::StateTransition(s) => &s.event
                    };

                    match event {
                        crate::parse::FsmTransitionEvent::Start => quote! { ev @ finny::FsmEvent::Start },
                        crate::parse::FsmTransitionEvent::Stop => quote ! { ev @ finny::FsmEvent::Stop },
                        crate::parse::FsmTransitionEvent::Event(ref ev) => {
                            let kind = &ev.ty;
                            quote! { finny::FsmEvent::Event(#event_enum_ty::#kind(ref ev)) }
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
                            if <#transition_ty>::execute_guard(&mut ctx, &ev, #region_id, &mut inspect_event_ctx)
                        }
                    } else {
                        TokenStream::new()
                    }
                };
                
                let fsm_sub_entry = match transition.ty {
                    FsmTransitionType::StateTransition(FsmStateTransition { state_to: FsmTransitionState::State(FsmState { kind: FsmStateKind::SubMachine(_), .. }), .. }) => {
                        quote! {
                            <#transition_ty>::execute_on_sub_entry(&mut ctx, #region_id, &mut inspect_event_ctx);
                        }
                    },
                    _ => TokenStream::new()
                };

                let m = quote! {
                    ( #match_state , #match_event ) #guard => {

                        <#transition_ty>::execute_transition(&mut ctx, &ev, #region_id, &mut inspect_event_ctx);

                        #fsm_sub_entry
                        
                    },
                };

                region_transitions.append_all(m);
            }

            // match and dispatch to submachines
            let region_submachines = {

                let mut sub_matches = TokenStream::new();

                let submachines: Vec<_> = region.transitions.iter().filter_map(|t| match &t.ty {
                    FsmTransitionType::InternalTransition(_) => None,
                    FsmTransitionType::SelfTransition(_) => None,
                    FsmTransitionType::StateTransition(FsmStateTransition { state_to: FsmTransitionState::State(s @ FsmState { kind: FsmStateKind::SubMachine(_), .. }), .. }) => {
                        Some(s)
                    },
                    _ => None
                }).collect();

                for submachine in submachines {
                    let kind = &submachine.ty;
                    let fsm_sub = FsmTypes::new(&submachine.ty, &fsm.base.fsm_generics);
                    let kind_variant = fsm_sub.get_fsm_no_generics_ty();

                    let sub = quote! {
                        ( finny::FsmCurrentState::State(#states_enum_ty :: #kind_variant), finny::FsmEvent::Event(#event_enum_ty::#kind_variant(ev))  ) => {
                            return finny::dispatch_to_submachine::<_, #kind, _, _, _>(&mut ctx, ev, &mut inspect_event_ctx);
                        },
                    };

                    sub_matches.append_all(sub);
                }

                sub_matches
            };

            regions.append_all(quote! {
                match (ctx.backend.current_states[#region_id], &event) {

                    #region_submachines
                    
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
                type States = #states_store_ty #fsm_generics_type;
                type Events = #event_enum_ty;

                fn dispatch_event<Q, I>(mut ctx: finny::DispatchContext<Self, Q, I>, event: finny::FsmEvent<Self::Events>) -> finny::FsmDispatchResult
                    where Q: finny::FsmEventQueue<Self>,
                    I: finny::Inspect
                {
                    use finny::{FsmTransitionGuard, FsmTransitionAction, FsmAction, FsmState, FsmTransitionFsmStart};

                    let mut transition_misses = 0;

                    let mut inspect_event_ctx = ctx.inspect.new_event::<Self>(&event);
                    
                    #regions

                    let result = if transition_misses == #region_count {
                        Err(finny::FsmError::NoTransition)
                    } else {
                        Ok(())
                    };

                    inspect_event_ctx.event_done();

                    result
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

            let state_ty = FsmTypes::new(&ty, &fsm.base.fsm_generics);
            let variant = state_ty.get_fsm_no_generics_ty();            

            let state = quote! {

                impl #fsm_generics_impl finny::FsmState<#fsm_ty #fsm_generics_type> for #ty #fsm_generics_where {
                    fn on_entry<'fsm_event, Q: finny::FsmEventQueue<#fsm_ty #fsm_generics_type>>(&mut self, context: &mut finny::EventContext<'fsm_event, #fsm_ty #fsm_generics_type, Q>) {
                        #on_entry
                    }

                    fn on_exit<'fsm_event, Q: finny::FsmEventQueue<#fsm_ty #fsm_generics_type>>(&mut self, context: &mut finny::EventContext<'fsm_event, #fsm_ty #fsm_generics_type, Q>) {
                        #on_exit
                    }

                    fn fsm_state() -> #states_enum_ty {
                        #states_enum_ty :: #variant
                    }
                }

            };

            states.append_all(state);

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

                fn new_submachine_backend(backend: finny::FsmBackendImpl<Self::Fsm>) -> finny::FsmResult<Self> where Self: Sized {
                    Ok(Self {
                        backend
                    })
                }
            }

            impl #fsm_generics_impl core::ops::Deref for #fsm_ty #fsm_generics_type #fsm_generics_where {
                type Target = finny::FsmBackendImpl<#fsm_ty #fsm_generics_type >;

                fn deref(&self) -> &Self::Target {
                    &self.backend
                }
            }

            impl #fsm_generics_impl core::ops::DerefMut for #fsm_ty #fsm_generics_type #fsm_generics_where {
                fn deref_mut(&mut self) -> &mut Self::Target {
                    &mut self.backend
                }
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