use std::collections::{HashMap, HashSet};

use proc_macro2::{Span, TokenStream};
use syn::{Error, Expr, ExprMethodCall, GenericArgument, ItemFn, parse::{self, Parse, ParseStream}, spanned::Spanned};

use crate::{parse_blocks::{FsmBlock, decode_blocks, get_generics, get_method_receiver_ident}, utils::{assert_no_generics, get_closure, to_field_name}};


pub struct FsmFnInput {
    pub base: FsmFnBase,
    pub decl: FsmDeclarations
}

#[derive(Debug)]
pub struct FsmFnBase {
    pub context_ty: syn::Type,
    pub fsm_ty: syn::Type,
    pub builder_ident: proc_macro2::Ident,
    pub fsm_generics: syn::Generics
}


impl FsmFnInput {
    pub fn parse(attr: TokenStream, item: TokenStream) -> syn::Result<Self> {
        let input_fn: syn::ItemFn = syn::parse2(item)?;

        // builder name/generics
        let (builder_ident, fsm_ty, context_ty) = {
            let input_fsm_builder = match (input_fn.sig.inputs.len(), input_fn.sig.inputs.first()) {
                (1, Some(p)) => {
                    Ok(p)
                },
                (_, _) => {
                    Err(Error::new(input_fn.sig.inputs.span(), "Only a single input parameter is supported!"))
                }
            }?;

            let builder_input = match input_fsm_builder {                
                syn::FnArg::Typed(pt) => Ok(pt),
                _ => Err(Error::new(input_fsm_builder.span(), "Only a typed input is supported!"))
            }?;

            let builder_input_pat_ident = match *builder_input.pat {
                syn::Pat::Ident(ref pi) => Ok(pi),
                _ => Err(Error::new(builder_input.pat.span(), "Only a type ascripted input arg is supported!"))
            }?;
            
            let builder_input_type = match *builder_input.ty {
                syn::Type::Path(ref type_path) => Ok(type_path),
                _ => Err(Error::new(builder_input.ty.span(), "The builder's type is incorrect!"))
            }?;

            let path_segment = match (builder_input_type.path.segments.len(), builder_input_type.path.segments.first()) {
                (1, Some(s)) => Ok(s),
                (_, _) => Err(Error::new(builder_input_type.path.segments.span(), "Only one segment is supported!"))
            }?;

            let generic_arguments = match &path_segment.arguments {
                syn::PathArguments::AngleBracketed(g) => Ok(g),
                _ => Err(Error::new(path_segment.arguments.span(), "Only one segment is supported!"))
            }?;


            let generic_tys: Vec<_> = generic_arguments.args.iter().collect();

            let (fsm_ty, context_ty) = match (generic_tys.get(0), generic_tys.get(1)) {
                (Some(GenericArgument::Type(fsm_ty)), Some(GenericArgument::Type(context_ty))) => {
                    Ok((fsm_ty, context_ty))
                },
                _ => Err(Error::new(generic_arguments.args.span(), "Expected a pair of generic arguments!"))
            }?;

            // remove the generics
            let fsm_ty = {
                let mut fsm_ty = fsm_ty.clone();
                match fsm_ty {
                    syn::Type::Path(ref mut tp) => {
                        let seg = tp.path.segments.first_mut().unwrap();
                        seg.arguments = syn::PathArguments::None;
                    },
                    _ => { return Err(syn::Error::new(fsm_ty.span(), "Unsupported FSM type.")); }
                }

                fsm_ty
            };

            (builder_input_pat_ident.ident.clone(), fsm_ty, context_ty.clone())
        };


        // return type check
        {
            let output_ty = match input_fn.sig.output {
                syn::ReturnType::Type(_, ref ty) => Ok(ty),
                _ => Err(syn::Error::new(input_fn.sig.output.span(), "The return type has to be 'BuiltFsm'!"))
            }?;

            let tp = match **output_ty {
                syn::Type::Path(ref tp) => Ok(tp),
                _ => Err(syn::Error::new(output_ty.span(), "The return type has to be 'BuiltFsm'!"))
            }?;

            match tp.path.get_ident() {
                Some(ident) if ident == "BuiltFsm" => Ok(()),
                _ => Err(syn::Error::new(tp.path.span(), "The return type has to be 'BuiltFsm'!"))
            }?
        }

        let base = FsmFnBase {
            builder_ident,
            context_ty,
            fsm_ty,
            fsm_generics: input_fn.sig.generics.clone()
        };

        let blocks = decode_blocks(&base, &input_fn)?;

        let fsm_declarations = FsmDeclarations::parse(&base, &input_fn, &blocks)?;

        Ok(FsmFnInput {
            base,
            decl: fsm_declarations
        })
    }
}

#[derive(Debug)]
pub struct FsmDeclarations {
    pub initial_state: syn::Type,
    pub states: HashMap<syn::Type, FsmState>,
    pub events: HashMap<syn::Type, FsmEvent>,
    pub transitions: Vec<FsmTransition>
}

#[derive(Debug)]
pub enum FsmTransitionState {
    None,
    State(FsmState)
}

#[derive(Debug)]
pub enum FsmTransitionEvent {
    Stop,
    Start,
    Event(FsmEvent)
}

#[derive(Debug)]
pub struct FsmTransition {
    pub transition_ty: syn::Type,
    pub from: FsmTransitionState,
    pub to: FsmTransitionState,
    pub event: FsmTransitionEvent
}

#[derive(Debug, Clone)]
pub struct FsmState {
    pub ty: syn::Type,
    pub state_storage_field: syn::Ident,
    pub on_entry_closure: Option<syn::ExprClosure>,
    pub on_exit_closure: Option<syn::ExprClosure>
}
#[derive(Debug, Clone)]
pub struct FsmEvent {
    pub ty: syn::Type,
    pub transitions: Vec<FsmEventTransition>,
    pub guard: Option<syn::ExprClosure>,
    pub action: Option<syn::ExprClosure>
}

#[derive(Debug, Clone)]
pub enum FsmEventTransition {
    State(syn::Type, syn::Type)
}

impl FsmDeclarations {
    pub fn parse(base: &FsmFnBase, input_fn: &ItemFn, blocks: &Vec<FsmBlock>) -> syn::Result<Self> {
        let mut initial_state = None;
        let mut states = HashMap::new();
        let mut events = HashMap::new();
        

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
                            initial_state = Some(ty.clone());
                        },
                        [MethodOverviewRef { name: "state", generics: [ty_state], .. }, st @ .. ] => {

                            assert_no_generics(ty_state)?;
                            let field_name = to_field_name(&ty_state)?;
                            let state = states
                                .entry(ty_state.clone())
                                .or_insert(FsmState { 
                                    ty: ty_state.clone(),
                                    on_entry_closure: None,
                                    on_exit_closure: None,
                                    state_storage_field: field_name
                                });

                            for method in st {

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
                                    _ => { return Err(syn::Error::new(mc.expr_call.span(), format!("Unsupported method '{}'!", method.name))); }
                                }
                            }
                            
                        },
                        [MethodOverviewRef { name: "on_event", generics: [ty_event], .. }, ev @ .. ] => {

                            assert_no_generics(ty_event)?;
                            let event = events
                                .entry(ty_event.clone())
                                .or_insert(FsmEvent { ty: ty_event.clone(), transitions: vec![], guard: None, action: None });

                            match ev {
                                [MethodOverviewRef { name: "transition_from", generics: [ty_from], .. }, MethodOverviewRef { name: "to", generics: [ty_to], .. }, ev @ .. ] => {

                                    event.transitions.push(FsmEventTransition::State(ty_from.clone(), ty_to.clone()));

                                    for method in ev {
                                        match method {
                                            MethodOverviewRef { name: "guard", .. } => {
                                                let closure = get_closure(method.call)?;

                                                if event.guard.is_some() {
                                                    return Err(syn::Error::new(closure.span(), "Duplicate 'guard'!"));
                                                }

                                                event.guard = Some(closure.clone());
                                            },
                                            MethodOverviewRef { name: "action", .. } => {
                                                let closure = get_closure(method.call)?;

                                                if event.action.is_some() {
                                                    return Err(syn::Error::new(closure.span(), "Duplicate 'action'!"));
                                                }

                                                event.action = Some(closure.clone());
                                            }
                                            _ => { return Err(syn::Error::new(mc.expr_call.span(), "Unsupported methods.")); }
                                        }
                                    }

                                },
                                _ => { return Err(syn::Error::new(mc.expr_call.span(), "Unsupported methods.")); }
                            }
                                                        
                        }
                        _ => { return Err(syn::Error::new(mc.expr_call.span(), "Unsupported method.")); }
                    }

                },
                _ => todo!("unsupported block!")
            }
        }

        let mut transitions = vec![];

        let initial_state = initial_state.ok_or(syn::Error::new(input_fn.span(), "Missing the initial state declaration! Use the method 'initial_state'."))?;
        let fsm_initial_state = states.get(&initial_state).ok_or(syn::Error::new(initial_state.span(), "The initial state is not refered in the builder. Use the 'state' method on the builder."))?;

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

            transitions.push(FsmTransition {
                transition_ty: generate_transition_ty(&mut i),
                from: FsmTransitionState::None,
                to: FsmTransitionState::State(fsm_initial_state.clone()),
                event: FsmTransitionEvent::Start
            });

            for (ty, ev) in events.iter() {
                for t in &ev.transitions {
                    match t {
                        FsmEventTransition::State(from, to) => {

                            let from = states.get(from).ok_or(syn::Error::new(from.span(), "State not found."))?;
                            let to = states.get(to).ok_or(syn::Error::new(to.span(), "State not found."))?;

                            transitions.push(FsmTransition {
                                transition_ty: generate_transition_ty(&mut i),
                                from: FsmTransitionState::State(from.clone()),
                                to: FsmTransitionState::State(to.clone()),
                                event: FsmTransitionEvent::Event(ev.clone())
                            });
                        }
                    }
                }
            }
        }
        
        
        let dec = FsmDeclarations {
            initial_state,
            states,
            events,
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