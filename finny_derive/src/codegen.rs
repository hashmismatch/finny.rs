use std::collections::HashSet;

use proc_macro2::{Span, TokenStream};
use quote::{TokenStreamExt, quote};
use crate::{codegen_meta::generate_fsm_meta, fsm::FsmTypes, parse::{FsmState, FsmStateAction, FsmStateKind}, utils::{remap_closure_inputs, to_field_name, tokens_to_string}};

use crate::{parse::{FsmFnInput, FsmMode, FsmRegion, FsmStateTransition, FsmTransition, FsmTransitionState, FsmTransitionType}, utils::ty_append};

pub fn generate_fsm_code(fsm: &FsmFnInput, _attr: TokenStream, input: TokenStream) -> syn::Result<TokenStream> {
    let fsm_ty = &fsm.base.fsm_ty;
    let fsm_types = FsmTypes::new(&fsm.base.fsm_ty, &fsm.base.fsm_generics);
    //let fsm_mod = to_field_name(&ty_append(fsm_ty, "Finny"))?;
    let is_async = fsm.base.mode == FsmMode::Async;
    // Async FSMs share their context through an `Arc`.
    let ctx_ty = {
        let ctx_ty = &fsm.base.context_ty;
        if is_async { quote! { finny::bundled::Arc< #ctx_ty > } } else { quote! { #ctx_ty } }
    };
    let async_kw = if is_async { quote! { async } } else { TokenStream::new() };

    let states_store_ty = ty_append(&fsm.base.fsm_ty, "States");
    let states_enum_ty = ty_append(&fsm.base.fsm_ty, "CurrentState");
    let timers_enum_ty = fsm_types.get_fsm_timers_ty();
    let timers_enum_iter_ty = fsm_types.get_fsm_timers_iter_ty();
    let timers_storage_ty = fsm_types.get_fsm_timers_storage_ty();
    let event_enum_ty = fsm_types.get_fsm_events_ty();

    let region_count = fsm.fsm.regions.len();
    let concurrent_regions = is_async && fsm.fsm.codegen_options.concurrent_regions && region_count > 1;

    let (fsm_generics_impl, fsm_generics_type, fsm_generics_where) = fsm.base.fsm_generics.split_for_impl();
    let de_generics = {
        let mut g = fsm.base.fsm_generics.clone();
        g.params.insert(0, syn::parse_quote!('de));
        g
    };
    let (de_generics_impl, _, _) = de_generics.split_for_impl();

    // `fsm.serde()`: the generated types are serialized along with the user's types.
    let serde = fsm.fsm.codegen_options.serde;
    let serde_derive = if serde {
        quote! {
            #[derive(finny::bundled::serde::Serialize)]
            #[serde(crate = "finny::bundled::serde")]
        }
    } else {
        TokenStream::new()
    };
    let serde_skip = if serde { quote! { #[serde(skip)] } } else { TokenStream::new() };
    // The deserialization is bounded on the user's types, so the machine can be restored only
    // when they implement `Deserialize` as well.
    let serde_bounded_derive = |tys: &[TokenStream]| -> TokenStream {
        if !serde { return TokenStream::new(); }
        let bounds = |tr: &str| tys.iter().map(|t| format!("{}: {}", t, tr)).collect::<Vec<_>>().join(", ");
        let ser = bounds("finny::bundled::serde::Serialize");
        let de = bounds("finny::bundled::serde::Deserialize<'de>");
        quote! {
            #[derive(finny::bundled::serde::Deserialize)]
            #[serde(bound(serialize = #ser, deserialize = #de))]
        }
    };

    let states_store = {

        let mut code_fields = TokenStream::new();
        let mut new_state_fields = TokenStream::new();
        let mut state_variants = TokenStream::new();
        let mut state_accessors = TokenStream::new();
        let mut state_field_tys = vec![];


        for (i, (_, state)) in fsm.fsm.states.iter().enumerate() {
            let name = &state.state_storage_field;
            let state_ty = FsmTypes::new(&state.ty, &fsm.base.fsm_generics);
            let ty = state_ty.get_fsm_ty();
            let ty_name = state_ty.get_fsm_no_generics_ty();

            for timer in &state.timers {
                let timer_ty = timer.get_ty(&fsm.base);
                let timer_field = timer.get_field(&fsm.base);

                code_fields.append_all(quote! { #timer_field: #timer_ty #fsm_generics_type, });
                new_state_fields.append_all(quote! { #timer_field: #timer_ty::default(), });

                state_accessors.append_all(quote! {
                    impl #fsm_generics_impl core::convert::AsRef<#timer_ty #fsm_generics_type> for #states_store_ty #fsm_generics_type #fsm_generics_where {
                        fn as_ref(&self) -> & #timer_ty #fsm_generics_type {
                            &self. #timer_field
                        }
                    }
    
                    impl #fsm_generics_impl core::convert::AsMut<#timer_ty #fsm_generics_type> for #states_store_ty #fsm_generics_type #fsm_generics_where {
                        fn as_mut(&mut self) -> &mut #timer_ty #fsm_generics_type {
                            &mut self. #timer_field
                        }
                    }
                });
            }

            code_fields.append_all(quote! { #name: #ty, });
            state_field_tys.push(quote! { #ty });
            state_variants.append_all(quote!{ #ty_name, });

            let new_state_field = match state.kind {
                FsmStateKind::Normal => {
                    quote! {
                        #name: < #ty as finny::FsmStateFactory< #fsm_ty #fsm_generics_type > >::new_state(context)?,
                    }
                }
                FsmStateKind::SubMachine(ref sub) => {

                    let sub_ctx = match &sub.context_constructor {
                        Some(c) => {
                            let remap = remap_closure_inputs(&c.inputs, &[quote!{ context }])?;
                            let body = &c.body;
                            let ctx_codegen = quote! {
                                #remap
                                {
                                    #body
                                }
                            };
                            if is_async {
                                // converted into the sub machine's `Arc` context
                                quote! {
                                    let sub_ctx: < #ty as finny::FsmBackend >::Context = core::convert::Into::into({ #ctx_codegen });
                                }
                            } else {
                                quote! {
                                    let sub_ctx = { #ctx_codegen };
                                }
                            }
                        },
                        None => {
                            quote! {
                                let sub_ctx = Default::default();
                            }
                        }
                    };

                    let factory = if is_async { quote! { FsmAsyncFactory } } else { quote! { FsmFactory } };

                    quote! {
                        #name: {
                            use finny::#factory;

                            #sub_ctx
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


        let mut transitions_seen = HashSet::new();
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

                                let key = (state_from_ty.clone(), state_to_ty.clone());
                                if transitions_seen.contains(&key) { continue; }
                                transitions_seen.insert(key);

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

        let region_views = if concurrent_regions {
            let mut view_generics = fsm.base.fsm_generics.clone();
            view_generics.params.insert(0, syn::parse_quote!('fsm_region));
            let (view_impl, view_type, view_where) = view_generics.split_for_impl();

            let mut code = TokenStream::new();
            let mut view_tys = vec![];
            let mut view_inits = vec![];

            for region in &fsm.fsm.regions {
                let view_ty = ty_append(&fsm.base.fsm_ty, &format!("Region{}States", region.region_id));

                let mut view_fields: Vec<(syn::Ident, TokenStream)> = vec![];
                for state in &region.states {
                    let ty = &state.ty;
                    view_fields.push((state.state_storage_field.clone(), quote! { #ty }));
                    for timer in &state.timers {
                        let timer_ty = timer.get_ty(&fsm.base);
                        view_fields.push((timer.get_field(&fsm.base), quote! { #timer_ty #fsm_generics_type }));
                    }
                }

                let mut fields = TokenStream::new();
                let mut inits = TokenStream::new();
                let mut accessors = TokenStream::new();
                for (name, ty) in &view_fields {
                    fields.append_all(quote! { #name: &'fsm_region mut #ty, });
                    inits.append_all(quote! { #name: &mut self. #name, });
                    accessors.append_all(quote! {
                        impl #view_impl core::convert::AsRef<#ty> for #view_ty #view_type #view_where {
                            fn as_ref(&self) -> & #ty {
                                &*self. #name
                            }
                        }

                        impl #view_impl core::convert::AsMut<#ty> for #view_ty #view_type #view_where {
                            fn as_mut(&mut self) -> &mut #ty {
                                &mut *self. #name
                            }
                        }
                    });
                }

                let mut seen = HashSet::new();
                for transition in &region.transitions {
                    if let FsmTransitionType::StateTransition(ref s) = transition.ty {
                        if let (Ok(from), Ok(to)) = (s.state_from.get_fsm_state(), s.state_to.get_fsm_state()) {
                            if !seen.insert((from.ty.clone(), to.ty.clone())) { continue; }
                            let (from_ty, to_ty) = (&from.ty, &to.ty);
                            let (from_field, to_field) = (&from.state_storage_field, &to.state_storage_field);
                            accessors.append_all(quote! {
                                impl #view_impl finny::FsmStateTransitionAsMut<#from_ty, #to_ty> for #view_ty #view_type #view_where {
                                    fn as_state_transition_mut(&mut self) -> (&mut #from_ty, &mut #to_ty) {
                                        (&mut *self. #from_field, &mut *self. #to_field)
                                    }
                                }
                            });
                        }
                    }
                }

                code.append_all(quote! {
                    /// The states of a single region, for executing the regions concurrently.
                    #[doc(hidden)]
                    pub struct #view_ty #view_generics #view_where {
                        #fields
                        _fsm: core::marker::PhantomData<&'fsm_region #fsm_ty #fsm_generics_type>
                    }

                    #accessors
                });

                view_tys.push(quote! { #view_ty #view_type });
                view_inits.push(quote! { #view_ty { #inits _fsm: core::marker::PhantomData } });
            }

            code.append_all(quote! {
                impl #fsm_generics_impl #states_store_ty #fsm_generics_type #fsm_generics_where {
                    /// Splits the states into disjoint views, one for each region.
                    #[doc(hidden)]
                    pub fn split_regions<'fsm_region>(&'fsm_region mut self) -> ( #(#view_tys),* ) {
                        ( #(#view_inits),* )
                    }
                }
            });

            code
        } else {
            TokenStream::new()
        };

        let states_serde_derive = serde_bounded_derive(&state_field_tys);
        let states_enum_serde_derive = if serde {
            quote! { #[derive(finny::bundled::serde::Deserialize)] }
        } else {
            TokenStream::new()
        };

        quote! {
            /// States storage struct for the state machine.
            #serde_derive
            #states_serde_derive
            pub struct #states_store_ty #fsm_generics_type #fsm_generics_where {
                #code_fields
                #serde_skip
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
            #serde_derive
            #states_enum_serde_derive
            pub enum #states_enum_ty {
                #state_variants
            }

            impl #fsm_generics_impl finny::FsmStates< #fsm_ty #fsm_generics_type > for #states_store_ty #fsm_generics_type #fsm_generics_where {
                type StateKind = #states_enum_ty;
                type CurrentState = [finny::FsmCurrentState<Self::StateKind>; #region_count];
            }

            #state_accessors

            #transition_states

            #region_views
        }
    };
    

    let events_enum = {

        let submachines: Vec<_> = fsm.fsm.states.iter().filter_map(|(_, state)| {
            match &state.kind {
                FsmStateKind::Normal => None,
                FsmStateKind::SubMachine(sub) => {
                    Some((sub, state))
                }
            }
        }).collect();

        let mut variants = TokenStream::new();
        let mut as_ref_str = TokenStream::new();
        let mut variant_tys = vec![];
        let mut i = 0;

        for (ty, _ev) in  fsm.fsm.events.iter() {
            let ty_str = crate::utils::tokens_to_string(ty);

            variants.append_all(quote! { #ty ( #ty ),  });
            variant_tys.push(quote! { #ty });
            as_ref_str.append_all(quote! { #event_enum_ty:: #ty(_) => #ty_str, });
            i += 1;
        }

        for (_sub, state) in submachines {
            let sub_fsm = FsmTypes::new(&state.ty, &fsm.base.fsm_generics);
            let sub_fsm_event_ty = sub_fsm.get_fsm_events_ty();
            let sub_fsm_ty = sub_fsm.get_fsm_no_generics_ty();            

            let sub_fsm_event_ty_str = crate::utils::tokens_to_string(&sub_fsm_event_ty);

            variants.append_all(quote! {
                #sub_fsm_ty ( #sub_fsm_event_ty ),
            });
            variant_tys.push(quote! { #sub_fsm_event_ty });
            as_ref_str.append_all(quote! {
                #event_enum_ty :: #sub_fsm_ty(_) => #sub_fsm_event_ty_str ,
            });
            i += 1;
        }

        let mut derives = TokenStream::new();
        if fsm.fsm.codegen_options.event_debug {
            derives.append_all(quote! {
                #[derive(Debug)]
            });
        }

        let as_ref_str = match i {
            0 => {
                quote! {
                    stringify!(#event_enum_ty)
                }
            },
            _ => {
                quote! {
                    match self {
                        #as_ref_str
                    }
                }
            }
        };
        
        let events_serde_derive = serde_bounded_derive(&variant_tys);

        let evs = quote! {
            #[derive(finny::bundled::derive_more::From)]
            #[derive(Clone)]
            #derives
            #serde_derive
            #events_serde_derive
            pub enum #event_enum_ty {
                #variants
            }

            impl core::convert::AsRef<str> for #event_enum_ty {
                fn as_ref(&self) -> &'static str {
                    #as_ref_str
                }
            }
        };

        evs
    };
    
    let transition_types = {
        let (fsm_action_trait, fsm_start_trait, fsm_transition_action_trait) = if is_async {
            (quote! { FsmActionAsync }, quote! { FsmTransitionFsmStartAsync }, quote! { FsmTransitionActionAsync })
        } else {
            (quote! { FsmAction }, quote! { FsmTransitionFsmStart }, quote! { FsmTransitionAction })
        };

        let mut t = TokenStream::new();
        
        for region in &fsm.fsm.regions {
            for transition in &region.transitions {

                let ty = &transition.transition_ty;

                let mut transition_doc = String::new();

                let mut q = TokenStream::new();

                match &transition.ty {
                    // internal or self transtion (only the current state)
                    FsmTransitionType::InternalTransition(s) | FsmTransitionType::SelfTransition(s) => {
                        
                        let state = s.state.get_fsm_state()?;
                        let event_ty = &s.event.get_event()?.ty;

                        let is_self_transition = if let FsmTransitionType::SelfTransition(_) = &transition.ty { true } else { false };

                        transition_doc.push_str(&format!(" {} transition within state [{}], responds to the event [{}].",
                            if is_self_transition { "A self" } else {"An internal"},
                            tokens_to_string(&state.ty),
                            tokens_to_string(event_ty)
                        ));

                        if let Some(ref guard) = s.action.guard {
                            let remap = remap_closure_inputs(&guard.inputs, vec![
                                quote! { event }, quote! { context }, quote! { states }
                            ].as_slice())?;

                            let body = &guard.body;

                            transition_doc.push_str(" Guarded.");

                            let g = quote! {
                                impl #fsm_generics_impl finny::FsmTransitionGuard<#fsm_ty #fsm_generics_type, #event_ty> for #ty #fsm_generics_where {
                                    fn guard<'fsm_event, Q>(event: & #event_ty, context: &finny::EventContext<'fsm_event, #fsm_ty #fsm_generics_type, Q>, states: & #states_store_ty #fsm_generics_type ) -> bool
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

                            transition_doc.push_str(" Executes an action.");

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
                            impl #fsm_generics_impl finny::#fsm_action_trait<#fsm_ty #fsm_generics_type, #event_ty, #state_ty > for #ty #fsm_generics_where {
                                #async_kw fn action<'fsm_event, Q>(event: & #event_ty , context: &mut finny::EventContext<'fsm_event, #fsm_ty #fsm_generics_type, Q >, state: &mut #state_ty)
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

                        transition_doc.push_str(" Start transition.");

                        q.append_all(quote! {
                            impl #fsm_generics_impl finny::#fsm_start_trait<#fsm_ty #fsm_generics_type, #initial_state_ty > for #ty #fsm_generics_where {

                            }
                        });

                    },

                    // normal state transition
                    FsmTransitionType::StateTransition(s) => {

                        let event_ty = &s.event.get_event()?.ty;
                        let state_from = s.state_from.get_fsm_state()?;
                        let state_to = s.state_to.get_fsm_state()?;

                        transition_doc.push_str(&format!(" Transition, from state [{}] to state [{}] upon the event [{}].",
                            tokens_to_string(&state_from.ty),
                            tokens_to_string(&state_to.ty),
                            tokens_to_string(&event_ty)
                        ));

                        if let Some(ref guard) = s.action.guard {
                            let event_ty = &s.event.get_event()?.ty;

                            transition_doc.push_str(" Guarded.");

                            let remap = remap_closure_inputs(&guard.inputs, vec![
                                quote! { event }, quote! { context }, quote! { states }
                            ].as_slice())?;

                            let body = &guard.body;

                            let g = quote! {
                                impl #fsm_generics_impl finny::FsmTransitionGuard<#fsm_ty #fsm_generics_type, #event_ty> for #ty #fsm_generics_where {
                                    fn guard<'fsm_event, Q>(event: & #event_ty, context: &finny::EventContext<'fsm_event, #fsm_ty #fsm_generics_type, Q>, states: & #states_store_ty #fsm_generics_type) -> bool
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
                            transition_doc.push_str(" Executes an action.");

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
                        
                        let state_from_ty = &state_from.ty;
                        let state_to_ty = &state_to.ty;

                        let a = quote! {
                            impl #fsm_generics_impl finny::#fsm_transition_action_trait<#fsm_ty #fsm_generics_type, #event_ty, #state_from_ty, #state_to_ty> for #ty #fsm_generics_where {
                                #async_kw fn action<'fsm_event, Q>(event: & #event_ty , context: &mut finny::EventContext<'fsm_event, #fsm_ty #fsm_generics_type, Q >, from: &mut #state_from_ty, to: &mut #state_to_ty)
                                    where Q: finny::FsmEventQueue<#fsm_ty #fsm_generics_type>
                                {
                                    #action_body
                                }
                            }
                        };

                        q.append_all(a);
                    }
                }

                transition_doc.push_str(&format!(" Part of [{}].", tokens_to_string(fsm_ty)));

                q.append_all(quote! {
                    #[doc = #transition_doc ]
                    pub struct #ty;
                });
                
                t.append_all(q);
            }
        }

        t
    };

    let dispatch = {

        // The pattern for the current state of the region.
        let match_state = |transition: &FsmTransition| -> TokenStream {
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

        // The pattern for the event, binds it to `ev`.
        let match_event = |transition: &FsmTransition| -> TokenStream {
            let event = match &transition.ty {
                FsmTransitionType::InternalTransition(s) | FsmTransitionType::SelfTransition(s) => &s.event,
                FsmTransitionType::StateTransition(s) => &s.event
            };

            match event {
                crate::parse::FsmTransitionEvent::Start => quote! { ev @ finny::FsmEvent::Start },
                crate::parse::FsmTransitionEvent::Stop => quote ! { ev @ finny::FsmEvent::Stop },
                crate::parse::FsmTransitionEvent::Event(ev) => {
                    let kind = &ev.ty;
                    quote! { finny::FsmEvent::Event(#event_enum_ty::#kind(ev)) }
                }
            }
        };

        let match_guard = |transition: &FsmTransition, region_id: usize| -> TokenStream {
            let has_guard = match &transition.ty {
                FsmTransitionType::StateTransition(s) => {
                    s.action.guard.is_some()
                }
                FsmTransitionType::InternalTransition(s) | FsmTransitionType::SelfTransition(s) => {
                    s.action.guard.is_some()
                }
            };

            let transition_ty = &transition.transition_ty;
            if has_guard {
                quote! {
                    if <#transition_ty>::execute_guard(&mut ctx, &ev, #region_id, &inspect_event_ctx)
                }
            } else {
                TokenStream::new()
            }
        };

        let entered_state = |transition: &'_ FsmTransition| -> Option<FsmState> {
            match &transition.ty {
                FsmTransitionType::SelfTransition(FsmStateAction { state: FsmTransitionState::State(st), .. }) => Some(st.clone()),
                FsmTransitionType::StateTransition(FsmStateTransition { state_to: FsmTransitionState::State(st), .. }) => Some(st.clone()),
                _ => None
            }
        };

        let exited_state = |transition: &'_ FsmTransition| -> Option<FsmState> {
            match &transition.ty {
                FsmTransitionType::SelfTransition(FsmStateAction { state: FsmTransitionState::State(st), .. }) => Some(st.clone()),
                FsmTransitionType::StateTransition(FsmStateTransition { state_from: FsmTransitionState::State(st), .. }) => Some(st.clone()),
                _ => None
            }
        };

        let sub_machine_state = |state: Option<FsmState>| -> Option<FsmState> {
            state.filter(|s| if let FsmStateKind::SubMachine(_) = s.kind { true } else { false })
        };

        let entered_sub_machine = |transition: &'_ FsmTransition| -> Option<FsmState> {
            sub_machine_state(entered_state(transition))
        };

        let exited_sub_machine = |transition: &'_ FsmTransition| -> Option<FsmState> {
            sub_machine_state(exited_state(transition))
        };

        // Starts or stops the timers of the state. Sync FSMs operate on the dispatch context `ctx`,
        // async FSMs on the region context `rc`.
        let state_timers = |state: Option<FsmState>, enter: bool| -> TokenStream {
            let mut code = TokenStream::new();
            for timer in state.iter().flat_map(|s| s.timers.iter()) {
                let timer_field = timer.get_field(&fsm.base);
                let timer_ty = timer.get_ty(&fsm.base);

                code.append_all(match (is_async, enter) {
                    (false, true) => quote! {
                        {
                            use finny::FsmTimer;
                            ctx.backend.states. #timer_field . execute_on_enter( #timers_enum_ty :: #timer_ty , &mut ctx.backend.context, &inspect_event_ctx, ctx.timers );
                        }
                    },
                    (false, false) => quote! {
                        {
                            use finny::FsmTimer;
                            ctx.backend.states. #timer_field . execute_on_exit( #timers_enum_ty :: #timer_ty , &inspect_event_ctx, ctx.timers );
                        }
                    },
                    (true, true) => quote! {
                        {
                            use finny::FsmTimer;
                            rc.states. #timer_field . execute_on_enter( #timers_enum_ty :: #timer_ty , &mut *rc.context, &inspect_event_ctx, &mut *rc.timers );
                        }
                    },
                    (true, false) => quote! {
                        {
                            use finny::FsmTimer;
                            rc.states. #timer_field . execute_on_exit( #timers_enum_ty :: #timer_ty , &inspect_event_ctx, &mut *rc.timers );
                        }
                    }
                });
            }
            code
        };

        // Starts or stops a sub machine. Its errors are reported to the inspector.
        let sub_machine_lifecycle = |sub: Option<FsmState>, start: bool| -> TokenStream {
            let sub = match sub {
                Some(sub) => sub,
                None => return TokenStream::new()
            };
            let sub_ty = &sub.ty;

            let (call, msg) = match (is_async, start) {
                (false, true) => (quote! { finny::start_submachine::<_, #sub_ty, _, _, _>(&mut ctx, &inspect_event_ctx) }, "Failed to start the sub-machine."),
                (false, false) => (quote! { finny::stop_submachine::<_, #sub_ty, _, _, _>(&mut ctx, &inspect_event_ctx) }, "Failed to stop the sub-machine."),
                (true, true) => (quote! { finny::start_submachine_async::<_, #sub_ty, _, _, _, _>(&mut rc, &inspect_event_ctx).await }, "Failed to start the sub-machine."),
                (true, false) => (quote! { finny::stop_submachine_async::<_, #sub_ty, _, _, _, _>(&mut rc, &inspect_event_ctx).await }, "Failed to stop the sub-machine.")
            };

            quote! {
                if let Err(ref e) = #call {
                    inspect_event_ctx.on_error(#msg, e);
                }
            }
        };

        // The body of a matched transition. A sub machine that is exited is stopped first, so
        // its states are exited before the sub machine's own state (inner before outer).
        let transition_body = |transition: &FsmTransition, region_id: usize| -> TokenStream {
            let transition_ty = &transition.transition_ty;
            let stop_sub = sub_machine_lifecycle(exited_sub_machine(transition), false);
            let timers_exit = state_timers(exited_state(transition), false);
            let start_sub = sub_machine_lifecycle(entered_sub_machine(transition), true);
            let timers_enter = state_timers(entered_state(transition), true);

            let execute = if is_async {
                quote! { <#transition_ty>::execute_transition(&mut rc, &ev, &inspect_event_ctx).await; }
            } else {
                quote! { <#transition_ty>::execute_transition(&mut ctx, &ev, #region_id, &inspect_event_ctx); }
            };

            quote! {
                #stop_sub

                #timers_exit

                #execute

                #start_sub

                #timers_enter
            }
        };

        // Stopping the machine: exits the region's current state.
        let stop_body = |state: &FsmState, region_id: usize| -> TokenStream {
            let state_ty = &state.ty;
            let stop_sub = sub_machine_lifecycle(sub_machine_state(Some(state.clone())), false);
            let timers_exit = state_timers(Some(state.clone()), false);

            if is_async {
                quote! {
                    #stop_sub
                    #timers_exit
                    <#state_ty>::execute_on_exit(&mut rc, &inspect_event_ctx).await;
                    *rc.current_state = finny::FsmCurrentState::Stopped;
                }
            } else {
                quote! {
                    #stop_sub
                    #timers_exit
                    <#state_ty>::execute_on_exit(&mut ctx, #region_id, &inspect_event_ctx);
                    ctx.backend.current_states[#region_id] = finny::FsmCurrentState::Stopped;
                }
            }
        };

        let state_variant = |state: &FsmState| -> syn::Type {
            FsmTypes::new(&state.ty, &fsm.base.fsm_generics).get_fsm_no_generics_ty().clone()
        };

        // Sub machines that are the states of this region.
        let region_submachines = |region: &FsmRegion| -> Vec<(syn::Type, syn::Type)> {
            region.states.iter()
                .filter(|s| if let FsmStateKind::SubMachine(_) = s.kind { true } else { false })
                .map(|s| {
                    let sub_ty = FsmTypes::new(&s.ty, &fsm.base.fsm_generics);
                    (s.ty.clone(), sub_ty.get_fsm_no_generics_ty().clone())
                })
                .collect()
        };

        // Forwards the event or the timer to a sub machine.
        let dispatch_to_sub = |sub: &syn::Type, ev: TokenStream| -> TokenStream {
            if is_async {
                quote! {
                    finny::dispatch_to_submachine_async::<_, #sub, _, _, _, _>(&mut rc, #ev, &inspect_event_ctx).await
                }
            } else {
                quote! {
                    finny::dispatch_to_submachine::<_, #sub, _, _, _>(&mut ctx, #ev, &inspect_event_ctx)
                }
            }
        };

        // Triggers our own timers.
        let own_timers_trigger = |region: &FsmRegion| -> Vec<(syn::Type, TokenStream)> {
            region.states.iter().flat_map(|s| s.timers.iter()).map(|timer| {
                let timer_ty = timer.get_ty(&fsm.base);
                let trigger = quote! {
                    {
                        use finny::FsmTimer;
                        < #timer_ty #fsm_generics_type > :: execute_trigger(*timer_id, &mut ctx, &inspect_event_ctx);
                    }
                };
                (timer_ty, trigger)
            }).collect()
        };

        let dispatch_body = if !concurrent_regions {
            // The regions are dispatched one after the other, each one matching, guarding and
            // executing its transition.
            let mut regions = TokenStream::new();
            for region in &fsm.fsm.regions {
                let region_id = region.region_id;
                let region_context = if is_async {
                    quote! { let mut rc = ctx.region_context(#region_id); }
                } else {
                    TokenStream::new()
                };

                let mut arms = TokenStream::new();

                for (sub, kind_variant) in region_submachines(region) {
                    // only the sub machines that can be entered
                    let entered = region.transitions.iter().any(|t| entered_sub_machine(t).map(|s| s.ty == sub).unwrap_or(false));
                    if !entered { continue; }

                    let dispatch = dispatch_to_sub(&sub, quote! { finny::FsmEvent::Event(ev.clone()) });
                    arms.append_all(quote! {
                        ( finny::FsmCurrentState::State(#states_enum_ty :: #kind_variant), finny::FsmEvent::Event(#event_enum_ty::#kind_variant(ev))  ) => {
                            #region_context
                            forwarded = Some(#dispatch);
                            break 'fsm_regions;
                        },
                    });
                }

                for transition in &region.transitions {
                    let match_state = match_state(transition);
                    let match_event = match_event(transition);
                    let guard = match_guard(transition, region_id);
                    let body = transition_body(transition, region_id);

                    arms.append_all(quote! {
                        ( #match_state , #match_event ) #guard => {
                            #region_context
                            #body
                        },
                    });
                }

                for state in &region.states {
                    let variant = state_variant(state);
                    let body = stop_body(state, region_id);
                    arms.append_all(quote! {
                        ( finny::FsmCurrentState::State(#states_enum_ty :: #variant), finny::FsmEvent::Stop ) => {
                            #region_context
                            #body
                        },
                    });
                }

                // do not dispatch timers if the machine is stopped
                arms.append_all(quote! {
                    (finny::FsmCurrentState::Stopped, finny::FsmEvent::Timer(_)) => (),
                });

                for (timer_ty, trigger) in own_timers_trigger(region) {
                    arms.append_all(quote! {
                        (_, finny::FsmEvent::Timer( timer_id @ #timers_enum_ty :: #timer_ty )) => {
                            #trigger
                        },
                    });
                }

                for (sub, sub_variant) in region_submachines(region) {
                    let dispatch = dispatch_to_sub(&sub, quote! { finny::FsmEvent::Timer(*timer_id) });
                    arms.append_all(quote! {
                        (finny::FsmCurrentState::State(#states_enum_ty :: #sub_variant), finny::FsmEvent::Timer( #timers_enum_ty :: #sub_variant (timer_id))) => {
                            #region_context
                            forwarded = Some(#dispatch);
                            break 'fsm_regions;
                        },
                        // a timer of a sub machine that isn't active anymore
                        (_, finny::FsmEvent::Timer( #timers_enum_ty :: #sub_variant (_))) => (),
                    });
                }

                regions.append_all(quote! {
                    match (ctx.backend.current_states[#region_id], &event) {
                        #arms
                        _ => {
                            transition_misses += 1;
                        }
                    }
                });
            }

            quote! {
                // The result of an event or a timer forwarded to a sub machine.
                let mut forwarded: Option<finny::FsmDispatchResult> = None;

                'fsm_regions: {
                    #regions
                }

                let result = match forwarded {
                    Some(result) => result,
                    None if transition_misses == #region_count => Err(finny::FsmError::NoTransition),
                    None => Ok(())
                };

                inspect_event_ctx.on_dispatch_result(&result);
                inspect_event_ctx.event_done(&ctx.backend);

                result
            }
        } else {
            // Concurrent regions. First all the regions select their transitions, then the
            // selected transitions are executed concurrently, each region on its own view of
            // the states, its own queue and timers buffers.
            let mut select = TokenStream::new();
            let mut region_futures = TokenStream::new();
            let mut merge = TokenStream::new();

            let region_idents = |prefix: &str| -> Vec<syn::Ident> {
                (0..region_count).map(|r| syn::Ident::new(&format!("{}_{}", prefix, r), Span::call_site())).collect()
            };
            let views = region_idents("region_states");
            let current_states = region_idents("region_current_state");
            let futures = region_idents("region_future");
            let results = region_idents("region_result");

            for region in &fsm.fsm.regions {
                let region_id = region.region_id;
                let selected = syn::Ident::new(&format!("region_selected_{}", region_id), Span::call_site());
                let view = &views[region_id];
                let current_state = &current_states[region_id];
                let future = &futures[region_id];
                let context = syn::Ident::new(&format!("region_context_{}", region_id), Span::call_site());
                let queue = syn::Ident::new(&format!("region_queue_{}", region_id), Span::call_site());
                let timers = syn::Ident::new(&format!("region_timers_{}", region_id), Span::call_site());

                let mut select_arms = TokenStream::new();
                let mut execute_arms = TokenStream::new();
                let mut arm_id = 0usize;

                for (sub, kind_variant) in region_submachines(region) {
                    let entered = region.transitions.iter().any(|t| entered_sub_machine(t).map(|s| s.ty == sub).unwrap_or(false));
                    if !entered { continue; }

                    let dispatch = dispatch_to_sub(&sub, quote! { finny::FsmEvent::Event(ev.clone()) });
                    select_arms.append_all(quote! {
                        ( finny::FsmCurrentState::State(#states_enum_ty :: #kind_variant), finny::FsmEvent::Event(#event_enum_ty::#kind_variant(ev))  ) => Some(#arm_id),
                    });
                    execute_arms.append_all(quote! {
                        Some(#arm_id) => {
                            if let finny::FsmEvent::Event(#event_enum_ty::#kind_variant(ev)) = &event {
                                region_result = #dispatch;
                            }
                        },
                    });
                    arm_id += 1;
                }

                for transition in &region.transitions {
                    let match_state = match_state(transition);
                    let match_event = match_event(transition);
                    let guard = match_guard(transition, region_id);
                    let body = transition_body(transition, region_id);

                    select_arms.append_all(quote! {
                        ( #match_state , #match_event ) #guard => Some(#arm_id),
                    });
                    execute_arms.append_all(quote! {
                        Some(#arm_id) => {
                            if let #match_event = &event {
                                #body
                            }
                        },
                    });
                    arm_id += 1;
                }

                for state in &region.states {
                    let variant = state_variant(state);
                    let body = stop_body(state, region_id);
                    select_arms.append_all(quote! {
                        ( finny::FsmCurrentState::State(#states_enum_ty :: #variant), finny::FsmEvent::Stop ) => Some(#arm_id),
                    });
                    execute_arms.append_all(quote! {
                        Some(#arm_id) => {
                            #body
                        },
                    });
                    arm_id += 1;
                }

                select_arms.append_all(quote! {
                    (finny::FsmCurrentState::Stopped, finny::FsmEvent::Timer(_)) => None,
                });

                // our own timers only enqueue their events, they are triggered right away
                for (timer_ty, trigger) in own_timers_trigger(region) {
                    select_arms.append_all(quote! {
                        (_, finny::FsmEvent::Timer( timer_id @ #timers_enum_ty :: #timer_ty )) => {
                            #trigger
                            None
                        },
                    });
                }

                for (sub, sub_variant) in region_submachines(region) {
                    let dispatch = dispatch_to_sub(&sub, quote! { finny::FsmEvent::Timer(*timer_id) });
                    select_arms.append_all(quote! {
                        (finny::FsmCurrentState::State(#states_enum_ty :: #sub_variant), finny::FsmEvent::Timer( #timers_enum_ty :: #sub_variant (timer_id))) => Some(#arm_id),
                        // a timer of a sub machine that isn't active anymore
                        (_, finny::FsmEvent::Timer( #timers_enum_ty :: #sub_variant (_))) => None,
                    });
                    execute_arms.append_all(quote! {
                        Some(#arm_id) => {
                            if let finny::FsmEvent::Timer( #timers_enum_ty :: #sub_variant (timer_id)) = &event {
                                region_result = #dispatch;
                            }
                        },
                    });
                    arm_id += 1;
                }

                select.append_all(quote! {
                    let #selected: Option<usize> = match (ctx.backend.current_states[#region_id], &event) {
                        #select_arms
                        _ => {
                            transition_misses += 1;
                            None
                        }
                    };
                });

                region_futures.append_all(quote! {
                    let mut #context = ctx.backend.context.clone();
                    let mut #queue = finny::FsmEventQueueVec::<Self>::new();
                    let mut #timers = region_timers.region();
                    let #future = async {
                        let mut region_result: finny::FsmDispatchResult = Ok(());
                        let mut rc = finny::RegionContext::<Self, _, _, _> {
                            states: &mut #view,
                            context: &mut #context,
                            current_state: #current_state,
                            queue: &mut #queue,
                            timers: &mut #timers,
                            region: #region_id
                        };

                        match #selected {
                            #execute_arms
                            _ => ()
                        }

                        region_result
                    };
                });

                merge.append_all(quote! {
                    finny::merge_region_queue(#queue, &mut *ctx.queue, &inspect_event_ctx);
                    // an action replaced the shared context
                    if !finny::bundled::Arc::ptr_eq(&#context, &original_context) {
                        ctx.backend.context = #context;
                    }
                });
            }

            quote! {
                #select

                let (#(mut #views),*) = ctx.backend.states.split_regions();
                let [#(#current_states),*] = &mut ctx.backend.current_states;
                let original_context = ctx.backend.context.clone();
                let region_timers = finny::FsmTimersShared::<Self, _>::new(&mut *ctx.timers);

                #region_futures

                let (#(#results),*) = finny::bundled::tokio::join!(#(#futures),*);
                drop(region_timers);

                // in the order of the regions
                #merge

                let mut region_error = None;
                for region_result in [#(#results),*] {
                    match region_result {
                        Ok(()) => (),
                        Err(finny::FsmError::NoTransition) => {
                            transition_misses += 1;
                        },
                        Err(e) => {
                            if region_error.is_none() {
                                region_error = Some(e);
                            }
                        }
                    }
                }

                let result = if let Some(e) = region_error {
                    Err(e)
                } else if transition_misses == #region_count {
                    Err(finny::FsmError::NoTransition)
                } else {
                    Ok(())
                };

                inspect_event_ctx.on_dispatch_result(&result);
                inspect_event_ctx.event_done(&ctx.backend);

                result
            }
        };

        let dispatch_impl = if is_async {
            quote! {
                impl #fsm_generics_impl finny::FsmAsyncDispatch for #fsm_ty #fsm_generics_type
                    #fsm_generics_where
                {
                    #[allow(unused_variables, unused_mut, unreachable_code, unused_labels)]
                    async fn dispatch_event<Q, I, T>(mut ctx: finny::DispatchContext<'_, '_, '_, Self, Q, I, T>, event: finny::FsmEvent<Self::Events, Self::Timers>) -> finny::FsmDispatchResult
                        where Q: finny::FsmEventQueue<Self>,
                        I: finny::Inspect, T: finny::FsmTimers<Self>
                    {
                        use finny::{FsmTransitionGuard, FsmTransitionActionAsync, FsmActionAsync, FsmStateAsync, FsmTransitionFsmStartAsync, Inspect};

                        let mut transition_misses = 0;

                        let inspect_event_ctx = ctx.inspect.new_event::<Self>(&event, &ctx.backend);

                        #dispatch_body
                    }
                }
            }
        } else {
            quote! {
                impl #fsm_generics_impl finny::FsmDispatch for #fsm_ty #fsm_generics_type
                    #fsm_generics_where
                {
                    #[allow(unused_labels)]
                    fn dispatch_event<Q, I, T>(mut ctx: finny::DispatchContext<Self, Q, I, T>, event: finny::FsmEvent<Self::Events, Self::Timers>) -> finny::FsmDispatchResult
                        where Q: finny::FsmEventQueue<Self>,
                        I: finny::Inspect, T: finny::FsmTimers<Self>
                    {
                        use finny::{FsmTransitionGuard, FsmTransitionAction, FsmAction, FsmState, FsmTransitionFsmStart};

                        let mut transition_misses = 0;

                        let mut inspect_event_ctx = ctx.inspect.new_event::<Self>(&event, &ctx.backend);

                        #dispatch_body
                    }
                }
            }
        };

        let serialize_fns = if serde {
            let mut valid_states = vec![];
            let mut restore_timers = TokenStream::new();
            for region in &fsm.fsm.regions {
                let region_id = region.region_id;
                for state in &region.states {
                    let state_ty = FsmTypes::new(&state.ty, &fsm.base.fsm_generics);
                    let variant = state_ty.get_fsm_no_generics_ty();
                    valid_states.push(quote! { (#region_id, #states_enum_ty :: #variant) });

                    for timer in &state.timers {
                        let timer_field = timer.get_field(&fsm.base);
                        restore_timers.append_all(quote! {
                            if let Some((id, settings)) = backend.states. #timer_field .instance.as_ref().map(|i| (i.id.clone(), i.settings)) {
                                let log = inspect.for_timer::<Self>(id.clone());
                                match timers.create(id.clone(), &settings.to_timer_settings()) {
                                    Ok(()) => {
                                        log.info("Restored the timer.");
                                        log.on_timer::<Self>(&id, &finny::InspectTimerEvent::Started { settings, restored: true });
                                    },
                                    Err(ref e) => {
                                        log.on_error("Failed to restore the timer", e);
                                        log.on_timer::<Self>(&id, &finny::InspectTimerEvent::Failed);
                                        backend.states. #timer_field .instance = None;
                                    }
                                }
                            }
                        });
                    }

                    if let FsmStateKind::SubMachine(_) = state.kind {
                        let sub_ty = state_ty.get_fsm_ty();
                        let field = &state.state_storage_field;
                        restore_timers.append_all(quote! {
                            {
                                let mut sub_timers = finny::FsmTimersSub {
                                    parent: &mut *timers,
                                    _parent_fsm: core::marker::PhantomData::<Self>,
                                    _sub_fsm: core::marker::PhantomData::< #sub_ty >
                                };
                                let sub_inspect = inspect.for_sub_machine::< #sub_ty >();
                                < #sub_ty as finny::FsmBackend >::restore_timers(&mut *backend.states. #field, &sub_inspect, &mut sub_timers);
                            }
                        });
                    }
                }
            }

            quote! {
                fn is_valid_current_state(region: finny::FsmRegionId, state: &#states_enum_ty) -> bool {
                    match (region, *state) {
                        #( #valid_states )|* => true,
                        _ => false
                    }
                }

                #[allow(unused_variables)]
                fn restore_timers<I: finny::Inspect, T: finny::FsmTimers<Self>>(backend: &mut finny::FsmBackendImpl<Self>, inspect: &I, timers: &mut T) {
                    use finny::Inspect;
                    #restore_timers
                }

                fn serialize_backend(backend: &finny::FsmBackendImpl<Self>) -> Option<&dyn finny::bundled::erased_serde::Serialize> {
                    Some(backend)
                }

                fn serialize_event(event: &Self::Events) -> Option<&dyn finny::bundled::erased_serde::Serialize> {
                    Some(event)
                }
            }
        } else {
            TokenStream::new()
        };

        quote! {

            impl #fsm_generics_impl finny::FsmBackend for #fsm_ty #fsm_generics_type
                #fsm_generics_where
            {
                type Context = #ctx_ty;
                type States = #states_store_ty #fsm_generics_type;
                type Events = #event_enum_ty;
                type Timers = #timers_enum_ty;

                #serialize_fns
            }

            #dispatch_impl

            impl #fsm_generics_impl core::fmt::Debug for #fsm_ty #fsm_generics_type
                #fsm_generics_where
            {
                fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error > {
                    Ok(())
                }
            }
        }
    };

    let states = {
        let fsm_state_trait = if is_async { quote! { FsmStateAsync } } else { quote! { FsmState } };

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

                impl #fsm_generics_impl finny::#fsm_state_trait<#fsm_ty #fsm_generics_type> for #ty #fsm_generics_where {
                    #async_kw fn on_entry<'fsm_event, Q: finny::FsmEventQueue<#fsm_ty #fsm_generics_type>>(&mut self, context: &mut finny::EventContext<'fsm_event, #fsm_ty #fsm_generics_type, Q>) {
                        #on_entry
                    }

                    #async_kw fn on_exit<'fsm_event, Q: finny::FsmEventQueue<#fsm_ty #fsm_generics_type>>(&mut self, context: &mut finny::EventContext<'fsm_event, #fsm_ty #fsm_generics_type, Q>) {
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
        let factory = if is_async { quote! { FsmAsyncFactory } } else { quote! { FsmFactory } };

        // A sub-machine is serialized within its parent's states.
        let serialize_fsm = if serde {
            let mut generics = fsm.base.fsm_generics.clone();
            generics.make_where_clause().predicates.push(syn::parse_quote! {
                finny::FsmBackendImpl< #fsm_ty #fsm_generics_type >: finny::bundled::serde::Serialize
            });
            let (_, _, where_clause) = generics.split_for_impl();

            let mut de_generics = de_generics.clone();
            de_generics.make_where_clause().predicates.push(syn::parse_quote! {
                finny::FsmBackendImpl< #fsm_ty #fsm_generics_type >: finny::bundled::serde::Deserialize<'de>
            });
            let (_, _, de_where_clause) = de_generics.split_for_impl();

            quote! {
                impl #fsm_generics_impl finny::bundled::serde::Serialize for #fsm_ty #fsm_generics_type #where_clause {
                    fn serialize<S: finny::bundled::serde::Serializer>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error> {
                        finny::bundled::serde::Serialize::serialize(&self.backend, serializer)
                    }
                }

                impl #de_generics_impl finny::bundled::serde::Deserialize<'de> for #fsm_ty #fsm_generics_type #de_where_clause {
                    fn deserialize<D: finny::bundled::serde::Deserializer<'de>>(deserializer: D) -> core::result::Result<Self, D::Error> {
                        let backend = <finny::FsmBackendImpl< #fsm_ty #fsm_generics_type > as finny::bundled::serde::Deserialize>::deserialize(deserializer)?;
                        Ok(Self { backend })
                    }
                }
            }
        } else {
            TokenStream::new()
        };

        quote! {

            /// A Finny Finite State Machine.
            pub struct #fsm_ty #fsm_generics_type #fsm_generics_where {
                backend: finny::FsmBackendImpl<#fsm_ty #fsm_generics_type >
            }

            impl #fsm_generics_impl finny::#factory for #fsm_ty #fsm_generics_type #fsm_generics_where {
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

            #serialize_fsm
        }
    };

    let timers = {

        let mut code = TokenStream::new();

        let mut enum_variants = vec![];
        let mut submachines = vec![];
        let mut our_timers = vec![];

        let states = fsm.fsm.states.iter().map(|s| s.1);
        for state in states {

            if let FsmStateKind::SubMachine(_) = &state.kind {
                let sub_fsm_ty = FsmTypes::new(&state.ty, &fsm.base.fsm_generics);
                let n = sub_fsm_ty.get_fsm_no_generics_ty();
                let t = sub_fsm_ty.get_fsm_timers_ty();
                enum_variants.push(quote! { #n ( #t )  });
                submachines.push(sub_fsm_ty.clone());

                code.append_all(quote! {

                    impl From<#t> for #timers_enum_ty {
                        fn from(t: #t) -> Self {
                            #timers_enum_ty :: #n ( t )
                        }
                    }

                });
            }

            for timer in &state.timers {
                let state_ty = &state.ty;
                let timer_ty = timer.get_ty(&fsm.base);

                enum_variants.push(quote! { #timer_ty });
                our_timers.push(timer_ty.clone());

                let setup = remap_closure_inputs(&timer.setup.inputs, &[quote! { ctx }, quote! { settings }])?;
                let setup_body = &timer.setup.body;

                let trigger = remap_closure_inputs(&timer.trigger.inputs, &[quote! { ctx }, quote! { state }])?;
                let trigger_body = &timer.trigger.body;

                let timer_doc = format!("A timer in the state [{}] of FSM [{}].", tokens_to_string(state_ty), tokens_to_string(fsm_ty));

                // A running timer is serialized as its settings, it is re-created on restore.
                let timer_serde = if serde {
                    quote! {
                        impl #fsm_generics_impl finny::bundled::serde::Serialize for #timer_ty #fsm_generics_type #fsm_generics_where {
                            fn serialize<S: finny::bundled::serde::Serializer>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error> {
                                finny::bundled::serde::Serialize::serialize(&self.instance.as_ref().map(|i| &i.settings), serializer)
                            }
                        }

                        impl #de_generics_impl finny::bundled::serde::Deserialize<'de> for #timer_ty #fsm_generics_type #fsm_generics_where {
                            fn deserialize<D: finny::bundled::serde::Deserializer<'de>>(deserializer: D) -> core::result::Result<Self, D::Error> {
                                let settings: Option<finny::TimerFsmSettings> = finny::bundled::serde::Deserialize::deserialize(deserializer)?;
                                Ok(Self {
                                    instance: settings.map(|settings| finny::TimerInstance { id: #timers_enum_ty :: #timer_ty, settings })
                                })
                            }
                        }
                    }
                } else {
                    TokenStream::new()
                };
                
                code.append_all(quote! {

                    #[doc = #timer_doc ]
                                        
                    pub struct #timer_ty #fsm_generics_type #fsm_generics_where {
                        instance: Option<finny::TimerInstance < #fsm_ty #fsm_generics_type > >
                    }

                    impl #fsm_generics_impl core::default::Default for #timer_ty #fsm_generics_type #fsm_generics_where {
                        fn default() -> Self {
                            Self {
                                instance: None
                            }
                        }
                    }

                    #timer_serde

                    impl #fsm_generics_impl finny::FsmTimer< #fsm_ty #fsm_generics_type , #state_ty > for #timer_ty #fsm_generics_type #fsm_generics_where {
                        fn setup(ctx: &mut #ctx_ty, settings: &mut finny::TimerFsmSettings) {
                            #setup
                            {
                                #setup_body
                            }
                        }

                        fn trigger(ctx: & #ctx_ty, state: & #state_ty ) -> Option< #event_enum_ty > {
                            #trigger
                            let ret = {
                                #trigger_body
                            };
                            ret
                        }

                        fn get_instance(&self) -> &Option<finny::TimerInstance < #fsm_ty #fsm_generics_type > > {
                            &self.instance
                        }

                        fn get_instance_mut(&mut self) -> &mut Option<finny::TimerInstance < #fsm_ty #fsm_generics_type > > {
                            &mut self.instance
                        }
                    }

                });
            }
        }


        let variants = {
            let mut t = TokenStream::new();
            t.append_separated(&enum_variants, quote! { , });
            t
        };

        code.append_all(quote! {
            #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
            pub enum #timers_enum_ty {
                #variants
            }
        });

        let submachine_iters: Vec<_> = submachines.iter().map(|s| {
            let ty = s.get_fsm_timers_iter_ty();
            let field = to_field_name(&ty);
            (ty, field)
        }).collect();

        let enum_iter_matches_variants = our_timers.iter().enumerate().map(|(i, variant)| quote! {
            #i => { self.position += 1; Some(#timers_enum_ty :: #variant) }
        });
        let mut enum_iter_matches = TokenStream::new();
        enum_iter_matches.append_separated(enum_iter_matches_variants, quote! { , });
        enum_iter_matches.append_separated(submachine_iters.iter().enumerate().map(|(i, (ty, field))| {
            let i = our_timers.len() + i;
            quote! {
                #i if self.#field.is_some() => { 
                    if let Some(ref mut iter) = self.#field {
                        let r = iter.next();
                        if let Some(r) = r {
                            let r = r.into();
                            return Some(r);
                        } else {
                            self.#field = None;
                            self.position += 1;
                            return self.next();
                        }
                    } else { None }
                }
            }
        }), quote! { , });
        
        let mut submachine_iter_struct = TokenStream::new();
        submachine_iter_struct.append_separated(submachine_iters.iter().map(|(ty, field)| quote! {
            #field : Option< #ty >
        }), quote! { , });

        let mut submachine_iter_new = TokenStream::new();
        submachine_iter_new.append_separated(submachine_iters.iter().map(|(ty, field)| quote! {
            #field : Some ( <#ty> :: new() )
        }), quote! { , });

        // timers iterator
        code.append_all(quote! {
            impl finny::AllVariants for #timers_enum_ty {
                type Iter = #timers_enum_iter_ty;

                fn iter() -> #timers_enum_iter_ty {
                    #timers_enum_iter_ty::new()
                }
            }

            pub struct #timers_enum_iter_ty {
                position: usize,
                #submachine_iter_struct
            }

            impl #timers_enum_iter_ty {
                pub fn new() -> Self {
                    Self {
                        position: 0,
                        #submachine_iter_new
                    }
                }
            }

            impl core::iter::Iterator for #timers_enum_iter_ty {
                type Item = #timers_enum_ty;

                fn next(&mut self) -> Option<Self::Item> {
                    match self.position {
                        #enum_iter_matches
                        _ => None
                    }
                }
            }
        });

        // timers storage
        let our_timers_storage: Vec<_> = our_timers.iter().map(|t| {
            let field = to_field_name(t);
            (field, t.clone())
        }).collect();
        
        let mut timers_storage_struct_fields = Vec::new();
        timers_storage_struct_fields.extend(our_timers_storage.iter().map(|(field, ty)| {
            quote! {
                #field: Option < TTimerStorage >
            }
        }));
        timers_storage_struct_fields.extend(submachines.iter().map(|s| {
            let ty = s.get_fsm_timers_storage_ty();
            let field = to_field_name(&ty);
            quote! {
                #field: #ty < TTimerStorage >
            }
        }));
        let mut fields = TokenStream::new();
        fields.append_separated(timers_storage_struct_fields, quote! { , });

        let mut new_fields_vec = vec![];
        new_fields_vec.extend(our_timers_storage.iter().map(|(field, ty)| quote! {
            #field: None
        }));
        new_fields_vec.extend(submachines.iter().map(|s| {
            let ty = s.get_fsm_timers_storage_ty();
            let field = to_field_name(&ty);
            quote! {
                #field: #ty :: default()
            }
        }));
        let mut new_fields = TokenStream::new();
        new_fields.append_separated(new_fields_vec, quote! { , });

        
        let mut timers_storage_matches = vec![];
        timers_storage_matches.extend(our_timers_storage.iter().map(|(field, ty)| {
            quote! {
                #timers_enum_ty :: #ty => &mut self. #field
            }
        }));
        timers_storage_matches.extend(submachines.iter().map(|s| {
            let ty = s.get_fsm_timers_storage_ty();
            //let t = s.get_fsm_timers_ty();
            let t = s.get_fsm_no_generics_ty();
            let field = to_field_name(&ty);
            quote! {
                #timers_enum_ty :: #t (ref sub) => {
                    self. #field .get_timer_storage_mut(sub)
                }
            }
        }));

        let matches = if timers_storage_matches.len() == 0 {
            quote! {
                panic!("Not supported in this FSM.");
            }
        } else {
            let mut m = TokenStream::new();
            m.append_separated(timers_storage_matches, quote! { , });

            quote! {
                match *id {
                    #m
                }
            }
        };

        code.append_all(quote! {

            pub struct #timers_storage_ty<TTimerStorage> {
                _storage: core::marker::PhantomData<TTimerStorage>,
                #fields
            }

            impl<TTimerStorage> core::default::Default for #timers_storage_ty<TTimerStorage> {
                fn default() -> Self {
                    Self {
                        _storage: core::marker::PhantomData::default(),
                        #new_fields
                    }
                }
            }

            impl<TTimerStorage> finny::TimersStorage<#timers_enum_ty , TTimerStorage> for #timers_storage_ty<TTimerStorage>
            {
                fn get_timer_storage_mut(&mut self, id: & #timers_enum_ty ) -> &mut Option<TTimerStorage> {
                    #matches
                }
            }

        });



        code
    };

    let fsm_meta = generate_fsm_meta(&fsm);

    let mut q = quote! {
        #states_store

        #states

        #events_enum

        #transition_types

        #dispatch

        #builder

        #timers

        #fsm_meta
    };

    // Re-emit the builder definition function as-is. It is never called, but keeping its
    // original tokens (and spans) in the output lets IDEs such as rust-analyzer resolve the
    // builder API calls for completions, hovers and go-to-definition.
    q.append_all(quote! {
        #[allow(dead_code, unused)]
        #input
    });

    Ok(q.into())
}