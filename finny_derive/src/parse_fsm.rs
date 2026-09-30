use indexmap::IndexMap;

use proc_macro2::Span;
use syn::{ExprMethodCall, ItemFn, Type, spanned::Spanned};

use crate::{parse::{EventGuardAction, FsmDeclarations, FsmEvent, FsmEventTransition, FsmFnBase, FsmMode, FsmState, FsmStateAction, FsmStateKind, FsmStateTransition, FsmSubMachineOptions, FsmTimer, FsmTransition, FsmTransitionEvent, FsmTransitionState, FsmTransitionType, ValidatedFsm}, parse_blocks::{FsmBlock, get_generics}, utils::{assert_no_generics, to_field_name, get_closure}, validation::create_regions};

#[derive(Copy, Clone, Debug)]
pub struct FsmCodegenOptions {
    pub event_debug: bool,
    /// Run the actions of the regions concurrently, async FSMs only.
    pub concurrent_regions: bool
}

impl FsmCodegenOptions {
    pub fn new() -> Self {
        Self {
            event_debug: false,
            concurrent_regions: false
        }
    }
}

/// The kind of a closure passed to the builder, to validate whether it has to be async.
#[derive(Copy, Clone, Debug, PartialEq)]
enum ClosureKind {
    /// State entry/exit actions and transition actions, async in async FSMs.
    Action,
    /// Always synchronous: guards, timer setups and sub-machine context constructors.
    Sync(&'static str)
}

fn check_closure(mode: FsmMode, kind: ClosureKind, closure: &syn::ExprClosure) -> syn::Result<()> {
    let is_async = closure.asyncness.is_some();
    match (mode, kind, is_async) {
        (FsmMode::Async, ClosureKind::Action, false) => {
            let msg = if let syn::Expr::Async(_) = *closure.body {
                "Use an async closure, `async |..| { .. }`, instead of a closure that returns an async block. The async block can't borrow the closure's arguments."
            } else {
                "The actions of an FSM built with `FsmAsyncBuilder` have to be async closures: `async |..| { .. }`."
            };
            Err(syn::Error::new(closure.span(), msg))
        },
        (FsmMode::Sync, ClosureKind::Action, true) => {
            Err(syn::Error::new(closure.span(), "Async actions require an FSM built with `FsmAsyncBuilder`."))
        },
        (_, ClosureKind::Sync(what), true) => {
            Err(syn::Error::new(closure.span(), format!("{} are always synchronous, remove the `async`.", what)))
        },
        _ => Ok(())
    }
}

pub struct FsmParser {
    initial_states: Vec<syn::Type>,
    states: IndexMap<Type, FsmState>,
    events: IndexMap<Type, FsmEvent>,
    options: FsmCodegenOptions,
    base: FsmFnBase,
    timer_id: usize
}

impl FsmParser {
    pub fn new(base: FsmFnBase) -> Self {
        FsmParser {
            initial_states: vec![],
            states: IndexMap::new(),
            events: IndexMap::new(),
            options: FsmCodegenOptions::new(),
            base,
            timer_id: 1
        }
    }

    pub fn parse(&mut self, _input_fn: &ItemFn, blocks: &Vec<FsmBlock>) -> syn::Result<()> {
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
                        [MethodOverviewRef { name: "events_debug", generics: [], .. }] => {
                            self.options.event_debug = true;
                        },
                        [m @ MethodOverviewRef { name: "concurrent_regions", generics: [], .. }] => {
                            if self.base.mode != FsmMode::Async {
                                return Err(syn::Error::new(m.call.span(), "Concurrent regions are only supported by FSMs built with `FsmAsyncBuilder`."));
                            }
                            self.options.concurrent_regions = true;
                        },
                        [MethodOverviewRef { name: "initial_state", generics: [ty], .. }] => {
                            assert_no_generics(ty)?;
                            if self.initial_states.len() > 0 { return Err(syn::Error::new(ty.span(), "Duplicate initial_state!")); }
                            self.initial_states.push(ty.clone());
                        },
                        [MethodOverviewRef { name: "initial_states", generics: [ty_tuple], .. }] => {

                            if self.initial_states.len() > 0 { return Err(syn::Error::new(ty_tuple.span(), "Duplicate initial_state!")); }

                            match ty_tuple {
                                Type::Tuple(tuple) => {
                                    for ty in &tuple.elems {
                                        assert_no_generics(ty)?;
                                        self.initial_states.push(ty.clone());
                                    }
                                }
                                _ => { return Err(syn::Error::new(ty_tuple.span(), "Expected a tuple of states!")); }
                            }
                        },

                        [MethodOverviewRef { name: "sub_machine", generics: [ty_sub_fsm], ..}, st @ .. ] => {

                            //assert_no_generics(ty_sub_fsm)?;
                            let field_name = to_field_name(&ty_sub_fsm);
                            
                            let state = self.states
                                .entry(ty_sub_fsm.clone())
                                .or_insert(FsmState {
                                    ty: ty_sub_fsm.clone(),
                                    state_storage_field: field_name,
                                    on_entry_closure: None,
                                    on_exit_closure: None,
                                    kind: FsmStateKind::SubMachine(FsmSubMachineOptions::default()),
                                    timers: vec![]
                                });
                            let mut sub_options = match state.kind {                                
                                FsmStateKind::SubMachine(ref sub) => sub.clone(),
                                _ => { return Err(syn::Error::new(ty_sub_fsm.span(), "Internal error with sub machines.")); }
                            };

                            match st {
                                [with_context @ MethodOverviewRef { name: "with_context", .. }, st @ .. ] => {
                                    let closure = get_closure(&with_context.call)?;
                                    check_closure(self.base.mode, ClosureKind::Sync("Sub-machine context constructors"), closure)?;
                                    if sub_options.context_constructor.is_some() {
                                        return Err(syn::Error::new(closure.span(), "Duplicate constructor for the context!"));
                                    }
                                    sub_options.context_constructor = Some(closure.clone());

                                    self.state_builder_parser(&ty_sub_fsm, st, true)?;
                                },
                                [st @ ..] => {
                                    self.state_builder_parser(&ty_sub_fsm, st, true)?;
                                },
                                _ => { return Err(syn::Error::new(ty_sub_fsm.span(), "Missing with_context?")); }
                            }                          

                            // update the options
                            self.states.entry(ty_sub_fsm.clone()).and_modify(|s| {
                                s.kind = FsmStateKind::SubMachine(sub_options);
                            });
                            
                        },

                        [MethodOverviewRef { name: "state", generics: [ty_state], .. }, st @ .. ] => {

                            self.state_builder_parser(ty_state, st, false)?;
                            
                        },

                        _ => { return Err(syn::Error::new(mc.expr_call.span(), "Unsupported method.")); }
                    }

                },
                _ => todo!("unsupported block!")
            }
        }

        Ok(())
    }

    fn parse_event_guard_action(mode: FsmMode, span: Span, event_method_calls: &[MethodOverviewRef]) -> syn::Result<EventGuardAction> {
        let mut guard_action = EventGuardAction { guard: None, action: None, type_hint: None, span };
        
        for method in event_method_calls {
            match method {
                MethodOverviewRef { name: "guard", .. } => {
                    let closure = get_closure(method.call)?;
                    check_closure(mode, ClosureKind::Sync("Guards"), closure)?;

                    if guard_action.guard.is_some() {
                        return Err(syn::Error::new(closure.span(), "Duplicate 'guard'!"));
                    }

                    guard_action.guard = Some(closure.clone());
                },
                MethodOverviewRef { name: "action", .. } => {
                    let closure = get_closure(method.call)?;
                    check_closure(mode, ClosureKind::Action, closure)?;

                    if guard_action.action.is_some() {
                        return Err(syn::Error::new(closure.span(), "Duplicate 'action'!"));
                    }

                    guard_action.action = Some(closure.clone());
                },
                MethodOverviewRef { name: "with_transition_ty", generics: [transition_ty], ..}  => {

                    if guard_action.type_hint.is_some() {
                        return Err(syn::Error::new(method.call.span(), "Duplicate 'with_transition_ty'!"));
                    }

                    guard_action.type_hint = Some(transition_ty.clone());

                },
                _ => { return Err(syn::Error::new(method.call.span(), "Unsupported method.")); }
            }
        }

        Ok(guard_action)
    }

    fn parse_state_on_event(mode: FsmMode, state: &FsmState, event: &mut FsmEvent, method_calls: &[MethodOverviewRef]) -> syn::Result<()> {
        match method_calls {
            [m @ MethodOverviewRef { name: "transition_to", generics: [ty_to], .. }, ev @ .. ] => {
                event.transitions.push(FsmEventTransition::State(state.ty.clone(), ty_to.clone(), Self::parse_event_guard_action(mode, m.call.method.span(), ev)?));
            },
            [m @ MethodOverviewRef { name: "internal_transition", generics: [], ..}, ev @ ..] => {
                event.transitions.push(FsmEventTransition::InternalTransition(state.ty.clone(), Self::parse_event_guard_action(mode, m.call.method.span(), ev)?));
            },
            [m @ MethodOverviewRef { name: "self_transition", generics: [], ..}, ev @ ..] => {
                event.transitions.push(FsmEventTransition::SelfTransition(state.ty.clone(), Self::parse_event_guard_action(mode, m.call.method.span(), ev)?));
            },
            [] => (),
            _ => { return Err(syn::Error::new(method_calls.first().map(|m| m.call.span()).unwrap_or(Span::call_site()), "Unsupported methods.")); }
        }

        Ok(())
    }

    pub fn validate(mut self, input_fn: &ItemFn) -> syn::Result<ValidatedFsm> {
        let mut transitions = vec![];

        if self.initial_states.len() == 0 {
            return Err(syn::Error::new(input_fn.span(), "Missing the initial state declaration! Use the method 'initial_state' or 'initial_states'."));
        }

        // The transitions of a state for the same event are tried in the order of their declaration.
        // Everything after a transition without a guard is unreachable.
        for (_, ev) in self.events.iter() {
            let mut unguarded: Vec<&Type> = vec![];
            for t in &ev.transitions {
                let (state, action) = match t {
                    FsmEventTransition::State(from, _, action) => (from, action),
                    FsmEventTransition::InternalTransition(state, action) | FsmEventTransition::SelfTransition(state, action) => (state, action)
                };

                if unguarded.contains(&state) {
                    return Err(syn::Error::new(action.span, "This transition can never be taken: an earlier transition of this state for the same event has no guard. The transitions are tried in the order of their declaration, add a guard to the earlier transition or remove this one."));
                }
                if action.guard.is_none() {
                    unguarded.push(state);
                }
            }
        }
        
        // build and validate the transitions table
        {
            let mut i = 0;

            fn generate_transition_ty(base: &FsmFnBase, i: &mut usize, ty_hint: &Option<syn::Type>) -> syn::Type {
                if let Some(type_hint) = ty_hint {
                    type_hint.clone()
                } else {
                    *i = *i + 1;
                    crate::utils::ty_append(&base.fsm_ty, &format!("Transition{}", i))
                }
            }

            // start transition
            for initial_state in &self.initial_states {
                let fsm_initial_state = self.states.get(initial_state).ok_or(syn::Error::new(initial_state.span(), "The initial state is not refered in the builder. Use the 'state' method on the builder."))?;

                transitions.push(FsmTransition {
                    transition_ty: generate_transition_ty(&self.base, &mut i, &None),
                    ty: FsmTransitionType::StateTransition(FsmStateTransition {
                        action: EventGuardAction::default(),
                        event: FsmTransitionEvent::Start,
                        state_from: FsmTransitionState::None,
                        state_to: FsmTransitionState::State(fsm_initial_state.clone())
                    })
                });
            }

            for (ty, ev) in self.events.iter() {
                for t in &ev.transitions {
                    match t {
                        FsmEventTransition::State(from, to, action) => {

                            let from = self.states.get(from).ok_or(syn::Error::new(from.span(), "State not found."))?;
                            let to = self.states.get(to).ok_or(syn::Error::new(to.span(), "State not found."))?;

                            transitions.push(FsmTransition {
                                transition_ty: generate_transition_ty(&self.base, &mut i, &action.type_hint),
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
                                transition_ty: generate_transition_ty(&self.base, &mut i, &action.type_hint),
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
                                transition_ty: generate_transition_ty(&self.base, &mut i, &action.type_hint),
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
            initial_states: self.initial_states,
            states: self.states,
            events: self.events,
            transitions
        };

        let regions = create_regions(dec, self.options)?;

        Ok(regions)
    }

    fn state_builder_parser(&mut self, ty_state: &syn::Type, st: &[MethodOverviewRef], is_sub_fsm: bool) -> syn::Result<()> {
        if !is_sub_fsm { assert_no_generics(ty_state)?; }
        let mode = self.base.mode;
        let field_name = to_field_name(&ty_state);
        let state = self.states
            .entry(ty_state.clone())
            .or_insert(FsmState { 
                ty: ty_state.clone(),
                on_entry_closure: None,
                on_exit_closure: None,
                state_storage_field: field_name,
                kind: FsmStateKind::Normal,
                timers: vec![]
            });

            
        let mut timer = None;

        for (i, method) in st.iter().enumerate() {
            match method {
                MethodOverviewRef { name: "on_entry", .. } => {
                    let closure = get_closure(&method.call)?;
                    check_closure(mode, ClosureKind::Action, closure)?;

                    if state.on_entry_closure.is_some() {
                        return Err(syn::Error::new(closure.span(), "Duplicate 'on_entry'!"));
                    }
                    state.on_entry_closure = Some(closure.clone());
                },
                MethodOverviewRef { name: "on_exit", .. } => {
                    let closure = get_closure(&method.call)?;
                    check_closure(mode, ClosureKind::Action, closure)?;

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
                    Self::parse_state_on_event(mode, state, event, other_method_calls)?;

                    break;
                },
                MethodOverviewRef { name: "on_entry_start_timer", generics: [], .. } => {

                    let call_args: Vec<_> = method.call.args.iter().collect();
                    match call_args.as_slice() {
                        [syn::Expr::Closure(setup), syn::Expr::Closure(trigger)] => {
                            check_closure(mode, ClosureKind::Sync("Timer setups"), setup)?;
                            check_closure(mode, ClosureKind::Sync("Timer triggers"), trigger)?;

                            if timer.is_some() { panic!("double timer bug!"); }
                            
                            timer = Some(FsmTimer {
                                setup: setup.clone(),
                                trigger: trigger.clone(),
                                id: self.timer_id,
                                type_hint: None
                            });                           
                            
                            self.timer_id += 1;
                            
                        },
                        _ => {
                            return Err(syn::Error::new(method.call.span(), "Unexpected arguments to the timer setup method."));
                        }
                    }
                },

                MethodOverviewRef { name: "with_timer_ty", generics: [timer_ty], .. } => {

                    if let Some(ref mut timer) = timer {
                        timer.type_hint = Some(timer_ty.clone());
                    } else {
                        return Err(syn::Error::new(method.call.span(), "Missing the timer to apply the type to."));
                    }

                },

                _ => { return Err(syn::Error::new(method.call.span(), format!("Unsupported method '{}'!", method.name))); }
            }
        }

        if let Some(timer) = timer {
            state.timers.push(timer);
        }

        Ok(())
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

#[derive(Clone)]
struct MethodOverviewRef<'a> {
    name: &'a str,
    generics: &'a [syn::Type],
    call: &'a ExprMethodCall
}