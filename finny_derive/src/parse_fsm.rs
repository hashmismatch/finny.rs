use std::collections::HashMap;

use proc_macro2::Span;
use syn::{ExprMethodCall, ItemFn, Type, spanned::Spanned};

use crate::{parse::{EventGuardAction, FsmDeclarations, FsmEvent, FsmEventTransition, FsmFnBase, FsmState, FsmStateAction, FsmStateTransition, FsmTransition, FsmTransitionEvent, FsmTransitionState, FsmTransitionType}, parse_blocks::{FsmBlock, get_generics}, utils::{assert_no_generics, to_field_name, get_closure}};

pub struct FsmParser {
    initial_state: Option<syn::Type>,
    states: HashMap<Type, FsmState>,
    events: HashMap<Type, FsmEvent>
}

impl FsmParser {
    pub fn new() -> Self {
        FsmParser {
            initial_state: None,
            states: HashMap::new(),
            events: HashMap::new()
        }
    }

    pub fn parse(&mut self, base: &FsmFnBase, input_fn: &ItemFn, blocks: &Vec<FsmBlock>) -> syn::Result<()> {
        for block in blocks {
            match block {
                FsmBlock::MethodCall(mc) => {

                    let methods = mc.method_calls
                        .iter()
                        .map(|m| MethodOverview::parse(m))
                        .collect::<syn::Result<Vec<_>>>()?;

                    let methods: Vec<_> = methods.iter().map(|m| m.as_ref()).collect();

                    match methods.as_slice() {
                        [MethodOverviewRef { name: "build", .. } ] => {
                            
                        },
                        [MethodOverviewRef { name: "initial_state", generics: [ty], .. }] => {
                            assert_no_generics(ty)?;
                            if self.initial_state.is_some() { return Err(syn::Error::new(ty.span(), "Duplicate initial_state!")); }
                            self.initial_state = Some(ty.clone());
                        },
                        [MethodOverviewRef { name: "state", generics: [ty_state], .. }, st @ .. ] => {

                            assert_no_generics(ty_state)?;
                            let field_name = to_field_name(&ty_state)?;
                            let state = self.states
                                .entry(ty_state.clone())
                                .or_insert(FsmState { 
                                    ty: ty_state.clone(),
                                    on_entry_closure: None,
                                    on_exit_closure: None,
                                    state_storage_field: field_name
                                });

                            for (i, method) in st.iter().enumerate() {

                                match method {
                                    MethodOverviewRef { name: "on_entry", .. } => {
                                        let closure = get_closure(&method.call)?;

                                        if state.on_entry_closure.is_some() {
                                            return Err(syn::Error::new(closure.span(), "Duplicate 'on_entry'!"));
                                        }
                                        state.on_entry_closure = Some(closure.clone());
                                    },
                                    MethodOverviewRef { name: "on_exit", .. } => {
                                        let closure = get_closure(&method.call)?;

                                        if state.on_exit_closure.is_some() {
                                            return Err(syn::Error::new(closure.span(), "Duplicate 'on_exit'!"));
                                        }
                                        state.on_exit_closure = Some(closure.clone());
                                    },
                                    MethodOverviewRef { name: "on_event", generics: [ty_event], .. } => {
                                        assert_no_generics(ty_event)?;

                                        let event = self.events
                                            .entry(ty_event.clone())
                                            .or_insert(FsmEvent { ty: ty_event.clone(), transitions: vec![] });

                                        let other_method_calls = &st[(i+1)..];
                                        Self::parse_state_on_event(state, event, other_method_calls)?;

                                        break;
                                    },
                                    _ => { return Err(syn::Error::new(mc.expr_call.span(), format!("Unsupported method '{}'!", method.name))); }
                                }
                            }
                            
                        },

                        _ => { return Err(syn::Error::new(mc.expr_call.span(), "Unsupported method.")); }
                    }

                },
                _ => todo!("unsupported block!")
            }
        }

        Ok(())
    }

    fn parse_event_guard_action(event_method_calls: &[MethodOverviewRef]) -> syn::Result<EventGuardAction> {
        let mut guard_action = EventGuardAction { guard: None, action: None};
        
        for method in event_method_calls {
            match method {
                MethodOverviewRef { name: "guard", .. } => {
                    let closure = get_closure(method.call)?;

                    if guard_action.guard.is_some() {
                        return Err(syn::Error::new(closure.span(), "Duplicate 'guard'!"));
                    }

                    guard_action.guard = Some(closure.clone());
                },
                MethodOverviewRef { name: "action", .. } => {
                    let closure = get_closure(method.call)?;

                    if guard_action.action.is_some() {
                        return Err(syn::Error::new(closure.span(), "Duplicate 'action'!"));
                    }

                    guard_action.action = Some(closure.clone());
                }
                _ => { return Err(syn::Error::new(method.call.span(), "Unsupported method.")); }
            }
        }

        Ok(guard_action)
    }

    fn parse_state_on_event(state: &FsmState, event: &mut FsmEvent, method_calls: &[MethodOverviewRef]) -> syn::Result<()> {
        match method_calls {
            [MethodOverviewRef { name: "transition_to", generics: [ty_to], .. }, ev @ .. ] => {
                event.transitions.push(FsmEventTransition::State(state.ty.clone(), ty_to.clone(), Self::parse_event_guard_action(ev)?));                
            },
            [MethodOverviewRef { name: "internal_transition", generics: [], ..}, ev @ ..] => {
                event.transitions.push(FsmEventTransition::InternalTransition(state.ty.clone(), Self::parse_event_guard_action(ev)?));
            },
            _ => { return Err(syn::Error::new(method_calls.first().map(|m| m.call.span()).unwrap_or(Span::call_site()), "Unsupported methods.")); }
        }

        Ok(())
    }

    pub fn validate(self, input_fn: &ItemFn) -> syn::Result<FsmDeclarations> {
        let mut transitions = vec![];

        let initial_state = self.initial_state.ok_or(syn::Error::new(input_fn.span(), "Missing the initial state declaration! Use the method 'initial_state'."))?;
        let fsm_initial_state = self.states.get(&initial_state).ok_or(syn::Error::new(initial_state.span(), "The initial state is not refered in the builder. Use the 'state' method on the builder."))?;

        // build and validate the transitions table
        {
            let mut i = 1;

            fn generate_transition_ty(i: &mut usize) -> syn::Type {
                let ident = syn::Ident::new(&format!("Transition{}", i), Span::call_site());
                *i = *i + 1;
                let mut p = syn::punctuated::Punctuated::new();
                p.push(syn::PathSegment {ident, arguments: syn::PathArguments::None });
                syn::Type::Path(syn::TypePath { qself: None, path: syn::Path { leading_colon: None, segments: p }})
            }

            // start transition
            transitions.push(FsmTransition {
                transition_ty: generate_transition_ty(&mut i),
                ty: FsmTransitionType::StateTransition(FsmStateTransition {
                    action: EventGuardAction::default(),
                    event: FsmTransitionEvent::Start,
                    state_from: FsmTransitionState::None,
                    state_to: FsmTransitionState::State(fsm_initial_state.clone())
                })
            });

            for (ty, ev) in self.events.iter() {
                for t in &ev.transitions {
                    match t {
                        FsmEventTransition::State(from, to, action) => {

                            let from = self.states.get(from).ok_or(syn::Error::new(from.span(), "State not found."))?;
                            let to = self.states.get(to).ok_or(syn::Error::new(to.span(), "State not found."))?;

                            transitions.push(FsmTransition {
                                transition_ty: generate_transition_ty(&mut i),
                                ty: FsmTransitionType::StateTransition(FsmStateTransition {
                                    action: action.clone(),
                                    state_from: FsmTransitionState::State(from.clone()),
                                    state_to: FsmTransitionState::State(to.clone()),
                                    event: FsmTransitionEvent::Event(ev.clone())
                                })
                            });
                        }
                        FsmEventTransition::InternalTransition(state, action) => {
                            // todo: code duplication!
                            let state = self.states.get(state).ok_or(syn::Error::new(state.span(), "State not found."))?;
                            transitions.push(FsmTransition {
                                transition_ty: generate_transition_ty(&mut i),
                                ty: FsmTransitionType::InternalTransition(FsmStateAction {
                                    state: FsmTransitionState::State(state.clone()),
                                    action: action.clone(),
                                    event: FsmTransitionEvent::Event(ev.clone())
                                })
                            });
                        }
                        FsmEventTransition::SelfTransition(state, action) => {
                            // todo: code duplication!
                            let state = self.states.get(state).ok_or(syn::Error::new(state.span(), "State not found."))?;
                            transitions.push(FsmTransition {
                                transition_ty: generate_transition_ty(&mut i),
                                ty: FsmTransitionType::SelfTransition(FsmStateAction {
                                    state: FsmTransitionState::State(state.clone()),
                                    action: action.clone(),
                                    event: FsmTransitionEvent::Event(ev.clone())
                                })
                            });
                        }
                    }
                }
            }
        }
                
        let dec = FsmDeclarations {
            initial_state,
            states: self.states,
            events: self.events,
            transitions
        };

        Ok(dec)
    }
}


struct MethodOverview {
    name: String,
    generics: Vec<syn::Type>,
    call: ExprMethodCall
}

impl MethodOverview {
    pub fn parse(m: &ExprMethodCall) -> syn::Result<Self> {
        let generics = get_generics(&m.turbofish)?;

        Ok(Self {
            name: m.method.to_string(),
            generics,
            call: m.clone()
        })
    }

    pub fn as_ref(&self) -> MethodOverviewRef {
        MethodOverviewRef {
            name: self.name.as_str(),
            generics: self.generics.as_slice(),
            call: &self.call
        }
    }
}

struct MethodOverviewRef<'a> {
    name: &'a str,
    generics: &'a [syn::Type],
    call: &'a ExprMethodCall
}