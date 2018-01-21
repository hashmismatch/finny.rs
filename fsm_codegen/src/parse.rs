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

fn transition_from_ty(id_counter: &mut u32, g: &[syn::Ty], transition_type: TransitionType) -> Vec<TransitionEntry> {
    let mut ret = vec![];

    let src_states = ty_to_vec(&g[1]);

    for src_state in src_states {
        let id = *id_counter;
        *id_counter += 1;

        let t = match transition_type {
            TransitionType::Normal => {
                TransitionEntry {
                    id: id,
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
                    id: id,
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
                    id: id,
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
    let mut copyable_events = false;
    let mut context_ty = syn::parse_type("()").unwrap();
    let mut transitions = Vec::new();
    let mut submachines = Vec::new();
    let mut shallow_history_events = Vec::new();
    let mut interrupt_states: Vec<FsmInterruptState> = Vec::new();
    let mut timeout_timers = vec![];
    let mut timer_id = 0;
    
    let mut transition_id = 1;


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

                        
                        continue;
                    }

                } else if let Ok(g) = match_type_grab_generics(&p, "CopyableEvents") {
                    copyable_events = true;                
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
                    transitions.extend_from_slice(&transition_from_ty(&mut transition_id, &g, TransitionType::Normal));
                } else if let Ok(g) = match_type_grab_generics(&p, "TransitionGuard") {
                    transitions.extend_from_slice(&transition_from_ty(&mut transition_id, &g, TransitionType::Normal));
                
                } else if let Ok(g) = match_type_grab_generics(&p, "TransitionSelf") {
                    transitions.extend_from_slice(&transition_from_ty(&mut transition_id, &g, TransitionType::SelfTransition));
                } else if let Ok(g) = match_type_grab_generics(&p, "TransitionSelfGuard") {
                    transitions.extend_from_slice(&transition_from_ty(&mut transition_id, &g, TransitionType::SelfTransition));

                } else if let Ok(g) = match_type_grab_generics(&p, "TransitionInternal") {
                    transitions.extend_from_slice(&transition_from_ty(&mut transition_id, &g, TransitionType::Internal));
                } else if let Ok(g) = match_type_grab_generics(&p, "TransitionInternalGuard") {
                    transitions.extend_from_slice(&transition_from_ty(&mut transition_id, &g, TransitionType::Internal));

                } else if let Ok(g) = match_type_grab_generics(&p, "TimerStateTimeout") {
                    timeout_timers.push(
                        FsmTimeoutTimer {
                            id: timer_id,
                            state: g.get(1).unwrap().clone(),
                            event_on_timeout: g.get(2).unwrap().clone()
                        }
                    );

                    timer_id += 1;                    
                } else {
                    panic!("Unknown parameter type: {:?}", p);
                }

            },
            _ => {
                panic!("nop!");
            }
        }        
    }


    let (generics, runtime_generics) = {
        use syn::*;

        let mut g = ast.generics.clone();
        let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

        let fsm_ty = syn::parse_type(&format!("{} {}", fsm_name, syn_to_string(&ty_generics))).unwrap();

        let all_fsm_types = {
            let mut f = vec![syn_to_string(&fsm_ty)];
            f.extend(submachines.iter().map(|sub| syn_to_string(sub)));
            f
        };
        
        g.ty_params.push(TyParam {
            attrs: vec![],
            ident: "FI".into(),
            bounds: all_fsm_types.iter().map(|t| {
                syn::parse_ty_param_bound(&format!("FsmInspect<{}>", t)).unwrap()
            }).collect(),
            default: None
        });

        g.ty_params.push(TyParam {
            attrs: vec![],
            ident: "FT".into(),
            bounds: vec![syn::parse_ty_param_bound(&"FsmTimers").unwrap()],
            default: None
        });

        //panic!("b: {:#?}", b);
        //panic!("g: {:#?}", g);

        (ast.generics.clone(), g)
    };




    let regions = create_regions(&transitions,
                                 &ty_to_vec(&initial_state_ty.expect("Missing Initial State")),
                                 &submachines,
                                 &interrupt_states
                                );


    FsmDescription {
        name: fsm_name,
        name_ident: fsm_name_ident,
        generics: generics,
        runtime_generics: runtime_generics,

        timeout_timers: timeout_timers,
        

        submachines: submachines,
        shallow_history_events: shallow_history_events,
        
        context_ty: context_ty,
        regions: regions,

        copyable_events: copyable_events
    }
}
