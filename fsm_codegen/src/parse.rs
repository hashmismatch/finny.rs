extern crate quote;
extern crate syn;

use fsm_def::*;
use graph::*;

fn match_type_grab_param_data(path: &syn::Path) -> Result<syn::AngleBracketedParameterData , ()> {
    if path.segments.len() == 1 {
        let ref segment = path.segments[0];

        match segment.parameters {
            syn::PathParameters::AngleBracketed(ref angle_bracketed) => {
                return Ok(angle_bracketed.clone());
            },
            _ => ()
        }
    }

    Err(())
}

fn match_type_grab_generics(path: &syn::Path, type_name: &str) -> Result<Vec<syn::Ty>, ()> {
    if path.segments.len() == 1 {
        let ref segment = path.segments[0];

        if segment.ident == syn::Ident::new(type_name) {
            match segment.parameters {
                syn::PathParameters::AngleBracketed(ref angle_bracketed) => {
                    return Ok(angle_bracketed.types.iter().cloned().collect());
                },
                _ => ()
            }
        }
    }

    Err(())
}

fn transition_from_ty(g: &[syn::Ty], transition_type: TransitionType) -> Vec<TransitionEntry> {
    let mut ret = vec![];

    let src_states = ty_to_vec(&g[1]);

    for src_state in src_states {
        let t = match transition_type {
            TransitionType::Normal => {
                TransitionEntry {
                    source_state: src_state.clone(),
                    event: g[2].clone(),
                    target_state: g[3].clone(),
                    action: g[4].clone(),
                    transition_type: TransitionType::Normal,
                    guard: g.get(5).cloned()
                }
            },
            TransitionType::SelfTransition => {
                TransitionEntry {
                    source_state: src_state.clone(),
                    event: g[2].clone(),
                    target_state: src_state.clone(),
                    action: g[3].clone(),
                    transition_type: TransitionType::SelfTransition,
                    guard: g.get(4).cloned()
                }
            },
            TransitionType::Internal => {
                TransitionEntry {
                    source_state: src_state.clone(),
                    event: g[2].clone(),
                    target_state: src_state.clone(),
                    action: g[3].clone(),
                    transition_type: TransitionType::Internal,
                    guard: g.get(4).cloned()
                }
            }
        };
        ret.push(t);
    }

    ret
}

pub fn parse_description(ast: &syn::MacroInput) -> FsmDescription {
    
    let fsm_name = ast.ident.as_ref().replace("Definition", "");
    let fsm_name_ident = syn::Ident::new(fsm_name.clone());

    let fields: Vec<_> = match ast.body {
        syn::Body::Struct(syn::VariantData::Tuple(ref fields)) => {
            fields.iter().collect()
        },
        _ => panic!("Tuples only!"),
    };


    let mut initial_state_ty = None;
    let mut inspect_ty = None;
    let mut context_ty = syn::parse_type("()").unwrap();
    let mut transitions = Vec::new();
    let mut submachines = Vec::new();
    let mut shallow_history_events = Vec::new();
    let mut interrupt_states: Vec<FsmInterruptState> = Vec::new();
    let mut lifetimes = Vec::new();


    for field in fields {

        match field.ty {
            syn::Ty::Path(None, ref p @ syn::Path { .. }) => {

                if let Ok(g) = match_type_grab_generics(&p, "InitialState") {
                    if let Some(t) = g.get(1) {
                        initial_state_ty = Some(t.clone());
                        continue;
                    }
                } else if let Ok(g) = match_type_grab_generics(&p, "ContextType") {
                
                    if let Some(t) = g.get(0) {
                        context_ty = t.clone();

                        if let &syn::Ty::Path(_, ref t) = t {
                            if let Ok(pd) = match_type_grab_param_data(&t) {
                                for lifetime in pd.lifetimes {
                                    lifetimes.push(lifetime.clone());
                                }
                            }
                        }
                        
                        continue;
                    }

                    
                } else if let Ok(g) = match_type_grab_generics(&p, "InspectionType") {
                    if let Some(t) = g.get(1) {
                        inspect_ty = Some(t.clone());
                        continue;
                    }
                } else if let Ok(g) = match_type_grab_generics(&p, "SubMachine") {
                    if let Some(t) = g.get(0) {
                        submachines.push(t.clone());
                        continue;
                    }
                } else if let Ok(g) = match_type_grab_generics(&p, "InterruptState") {

                    let st = g[1].clone();
                    let ev = g[2].clone();

                    let mut created = false;
                    if let Some(ref mut f) = interrupt_states.iter_mut().find(|x| x.interrupt_state_ty == st) {
                        f.resume_event_ty.push(ev.clone());
                        created = true;
                    }
                    
                    if !created {
                        interrupt_states.push(FsmInterruptState {
                            interrupt_state_ty: st,
                            resume_event_ty: vec![ev]
                        });
                    }

                } else if let Ok(g) = match_type_grab_generics(&p, "ShallowHistory") {
                    match (g.get(1), g.get(2)) {
                        (Some(event), Some(target_state)) => {
                            shallow_history_events.push(ShallowHistoryEvent {
                                event_ty: event.clone(),
                                target_state_ty: target_state.clone()
                            });
                        },
                        _ => ()
                    }
                } else if let Ok(g) = match_type_grab_generics(&p, "Transition") {
                    transitions.extend_from_slice(&transition_from_ty(&g, TransitionType::Normal));
                } else if let Ok(g) = match_type_grab_generics(&p, "TransitionGuard") {
                    transitions.extend_from_slice(&transition_from_ty(&g, TransitionType::Normal));
                
                } else if let Ok(g) = match_type_grab_generics(&p, "TransitionSelf") {
                    transitions.extend_from_slice(&transition_from_ty(&g, TransitionType::SelfTransition));
                } else if let Ok(g) = match_type_grab_generics(&p, "TransitionSelfGuard") {
                    transitions.extend_from_slice(&transition_from_ty(&g, TransitionType::SelfTransition));

                } else if let Ok(g) = match_type_grab_generics(&p, "TransitionInternal") {
                    transitions.extend_from_slice(&transition_from_ty(&g, TransitionType::Internal));
                } else if let Ok(g) = match_type_grab_generics(&p, "TransitionInternalGuard") {
                    transitions.extend_from_slice(&transition_from_ty(&g, TransitionType::Internal));

                } else {
                    panic!("Unknown parameter type: {:?}", p);
                }

            },
            _ => {
                panic!("nop!");
            }
        }
        
    }

    let regions = create_regions(&transitions,
                                 &ty_to_vec(&initial_state_ty.expect("Missing Initial State")),
                                 &submachines,
                                 &interrupt_states
                                );


    FsmDescription {
        name: fsm_name,
        name_ident: fsm_name_ident,
        lifetimes: lifetimes,

        submachines: submachines,
        shallow_history_events: shallow_history_events,
        
        context_ty: context_ty,
        inspect_ty: inspect_ty,
        regions: regions
    }
}
