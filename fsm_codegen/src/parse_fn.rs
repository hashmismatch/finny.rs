extern crate quote;
extern crate syn;

use fsm_def::*;
use graph::*;

use parse_fn_visitors::{extract_method_generic_ty, extract_method_generic_ty_all};

use syn::visit::*;

#[derive(Debug, Clone)]
pub struct LetFsmDeclaration {
    pub fsm_let_ident: syn::Ident,
    pub fsm_ty: syn::Type,
    pub fsm_ctx_ty: syn::Type,
    pub fsm_initial_state_ty: syn::Type
}

fn let_fsm_local(fn_body: &syn::ItemFn) -> LetFsmDeclaration {
    #[derive(Default, Debug)]
    struct FindDeclStmnt {
        fsm_let_ident: Option<syn::Ident>,
        fsm_ty: Option<syn::Type>,
        fsm_ctx_ty: Option<syn::Type>,
        fsm_initial_state_ty: Option<syn::Type>,
        expr_call: bool,
        expr_path: bool,
        in_local: bool,
        locals: Vec<syn::Local>,
        in_fsm_decl: bool,
        in_new_fsm: bool,
        last_let_ident: Option<syn::Ident>
    }
    impl<'ast> syn::visit::Visit<'ast> for FindDeclStmnt {
        fn visit_local(&mut self, i: &'ast syn::Local) {
            self.in_local = true;
            self.locals.push(i.clone());
            visit_local(self, i);
            self.locals.pop();
            self.in_local = false;
        }

        fn visit_expr_call(&mut self, i: &'ast syn::ExprCall) {
            self.expr_call = true;
            visit_expr_call(self, i);
            self.expr_call = false;
        }

        fn visit_expr_path(&mut self, i: &'ast syn::ExprPath) {
            self.expr_path = true;
            visit_expr_path(self, i);
            self.expr_path = false;
            self.in_fsm_decl = false;
        }

        fn visit_pat_ident(&mut self, i: &'ast syn::PatIdent) {
            self.last_let_ident = Some(i.ident.clone());
            visit_pat_ident(self, i);
        }

        fn visit_ident(&mut self, i: &'ast syn::Ident) {
            if self.expr_call && self.expr_path {
                if i.as_ref() == "FsmDecl" {
                    self.in_fsm_decl = true;

                    self.fsm_let_ident = self.last_let_ident;
                }
            }
            visit_ident(self, i);
        }

        fn visit_path_segment(&mut self, i: &'ast syn::PathSegment) {
            if self.expr_call && self.expr_path && self.in_fsm_decl {
                if i.ident.as_ref() == "new_fsm" {
                    self.in_new_fsm = true;
                }
            }
            visit_path_segment(self, i);

            self.in_new_fsm = false;
        }

        fn visit_angle_bracketed_generic_arguments(&mut self, i: &'ast syn::AngleBracketedGenericArguments) {            
            
            if self.in_new_fsm {
                if let &syn::GenericArgument::Type(ref ty) = &i.args[0] {
                    if self.fsm_ty.is_none() {
                        self.fsm_ty = Some(ty.clone());
                    }
                }
            }

            visit_angle_bracketed_generic_arguments(self, i);
        }

        fn visit_expr_method_call(&mut self, i: &'ast syn::ExprMethodCall) {
            if let Some(local) = self.locals.last() {
                
                if let syn::Pat::Ident(ref pat) = *local.pat {
                    // this method gets called before the static part that detects the FsmDecl!!! TODO
                    // not quite correct... we should check against self.fsm_let_ident, which will be verified!
                    if Some(pat.ident) == self.last_let_ident {
                        if i.method.as_ref() == "initial_state" {
                            if let Some(ref turbofish) = i.turbofish {                                
                                if let syn::GenericMethodArgument::Type(ref ty) = turbofish.args[0] {
                                    self.fsm_initial_state_ty = Some(ty.clone());
                                }
                            }
                        }

                        if i.method.as_ref() == "context_ty" {
                            if let Some(ref turbofish) = i.turbofish {                                
                                if let syn::GenericMethodArgument::Type(ref ty) = turbofish.args[0] {
                                    self.fsm_ctx_ty = Some(ty.clone());
                                }
                            }
                        }
                        
                    }
                }                
            }
            
            visit_expr_method_call(self, i);
        }
    }
    
    let mut finder: FindDeclStmnt = Default::default();    
    finder.visit_item_fn(&fn_body);
    
    LetFsmDeclaration {
        fsm_let_ident: finder.fsm_let_ident.expect("Missing FSM declaration local"),
        fsm_ty: finder.fsm_ty.expect("Missing FSM type"),
        fsm_ctx_ty: finder.fsm_ctx_ty.unwrap_or(syn::parse_str("()").unwrap()),
        fsm_initial_state_ty: finder.fsm_initial_state_ty.expect("Missing FSM initial state")
    }
}


struct IdentFinder {
    ident: syn::Ident,
    found: bool
}

impl<'ast> syn::visit::Visit<'ast> for IdentFinder {
    fn visit_ident(&mut self, i: &'ast syn::Ident) {
        if self.ident == i {
            self.found = true;
        }

        visit_ident(self, i);
    }
}


pub fn parse_definition_fn(fn_body: &syn::ItemFn) -> FsmDescription {
    
    let mut copyable_events = false;    
    let mut transitions = vec![];
    let mut inline_submachines = vec![];
    let mut shallow_history_events = vec![];
    let mut interrupt_states: Vec<FsmInterruptState> = vec![];
    let mut timeout_timers = vec![];
    let mut timer_id = 0;
    
    let mut transition_id = 1;


    let fsm_decl = let_fsm_local(fn_body);
        

    let fsm_name_ident = ::parse_fn_visitors::get_base_name(&fsm_decl.fsm_ty);
    let fsm_name = syn_to_string(&fsm_name_ident);
    //let fsm_name = syn_to_string(&fsm_decl.fsm_ty);
    //panic!("fsm_ty: {:?}", fsm_decl.fsm_ty);
    //panic!("fsm_name: {:?}", fsm_name);
    //let fsm_name_ident = syn::Ident::from(fsm_name.clone());



    //let inline_states = find_inline_states(fn_body, &fsm_decl);
    let mut inline_actions = vec![];
    let mut inline_guards = vec![];
    let mut inline_events = vec![];
    let mut inline_states = vec![];
    

    {        
        let method_calls = ::parse_fn_visitors::find_fsm_method_calls(fn_body, &fsm_decl);        
        for st in &method_calls {
            if let Some(first) = st.calls.get(0) {
                if first.method.as_ref() == "new_event" {
                    let event_ty = extract_method_generic_ty(first);

                    inline_events.push(FsmInlineEvent {
                        ty: event_ty,
                        unit: false
                    });
                } else if first.method.as_ref() == "add_sub_machine" {
                    let sub_ty = extract_method_generic_ty(first);
                    let mut on_entry = None;
                    let mut on_exit = None;

                    for call in &st.calls[1..] {
                        match call.method.as_ref() {
                            "on_entry" => {
                                let closure = if let syn::Expr::Closure(ref closure) = call.args[0] {
                                    closure.clone()
                                } else {
                                    panic!("missing closure?");
                                };

                                on_entry = Some(closure);
                            },
                            "on_exit" => {
                                let closure = if let syn::Expr::Closure(ref closure) = call.args[0] {
                                    closure.clone()
                                } else {
                                    panic!("missing closure?");
                                };

                                on_exit = Some(closure);
                            },
                            _ => { panic!("unsupported add_sub_machine method: {:?}", call); }
                        }
                    }

                    inline_submachines.push(FsmInlineSubMachine {
                        ty: sub_ty,
                        on_entry_closure: on_entry,
                        on_exit_closure: on_exit
                    });
                } else if first.method.as_ref() == "new_unit_event" {
                    let event_ty = extract_method_generic_ty(first);

                    inline_events.push(FsmInlineEvent {
                        ty: event_ty,
                        unit: true
                    });

                } else if first.method.as_ref() == "copyable_events" {
                    copyable_events = true;

                } else if first.method.as_ref() == "new_state_timeout" {
                    let generics = extract_method_generic_ty_all(first);
                    
                    let closure = if let syn::Expr::Closure(ref closure) = first.args[0] {
                        closure.clone()
                    } else {
                        panic!("missing timer closure?");
                    };
                    
                    timeout_timers.push(FsmTimeoutTimer {
                        id: timer_id,
                        state: generics[0].clone(),
                        event_on_timeout: generics[1].clone(),
                        timer_settings_closure: Some(closure)
                    });

                    timer_id += 1;                    

                } else if (first.method.as_ref() == "new_unit_state" || first.method.as_ref() == "new_state") {
                    let unit = first.method.as_ref() == "new_unit_state";
                    let state_ty = extract_method_generic_ty(first);

                    let mut on_entry = None;
                    let mut on_exit = None;

                    for call in &st.calls[1..] {
                        match call.method.as_ref() {
                            "on_entry" => {
                                let closure = if let syn::Expr::Closure(ref closure) = call.args[0] {
                                    closure.clone()
                                } else {
                                    panic!("missing closure?");
                                };

                                on_entry = Some(closure);
                            },
                            "on_exit" => {
                                let closure = if let syn::Expr::Closure(ref closure) = call.args[0] {
                                    closure.clone()
                                } else {
                                    panic!("missing closure?");
                                };

                                on_exit = Some(closure);
                            },
                            _ => { panic!("unsupported new_*_state method: {:?}", call); }
                        }
                    }

                    inline_states.push(FsmInlineState {
                        ty: state_ty,
                        unit: unit,
                        on_entry_closure: on_entry,
                        on_exit_closure: on_exit
                    });
                } else if first.method.as_ref() == "on_event" {
                    let event_ty = extract_method_generic_ty(first);
                    let mut transition_type = TransitionType::Normal;
                    let mut transition_from = None;
                    let mut transition_to = None;
                    let mut action = None;
                    let mut guard = None;

                    let mut transition_entry = false;

                    for call in &st.calls[1..] {
                        match call.method.as_ref() {
                            "transition_internal" => {
                                transition_type = TransitionType::Internal;
                                transition_from = Some(extract_method_generic_ty(&call));
                                transition_entry = true;
                            },
                            "transition_self" => {
                                transition_type = TransitionType::SelfTransition;
                                transition_from = Some(extract_method_generic_ty(&call));
                                transition_entry = true;
                            },
                            "transition_from" => {
                                transition_from = Some(extract_method_generic_ty(&call));
                                transition_entry = true;
                            },
                            "to" => {
                                transition_to = Some(extract_method_generic_ty(&call));
                                transition_entry = true;
                            },
                            "interrupt_state" => {
                                let state = extract_method_generic_ty(&call);
                                interrupt_states.push(FsmInterruptState {
                                    interrupt_state_ty: state,
                                    resume_event_ty: vec![event_ty.clone()]
                                });
                            },
                            "shallow_history" => {
                                let state = extract_method_generic_ty(&call);
                                shallow_history_events.push(ShallowHistoryEvent {
                                    event_ty: event_ty.clone(),
                                    target_state_ty: state
                                });
                            },
                            "action" => {
                                let action_closure = if let syn::Expr::Closure(ref closure) = call.args[0] {
                                    closure.clone()
                                } else {
                                    panic!("missing closure?");
                                };

                                match transition_type {
                                    TransitionType::Normal => {
                                        let transition_action_name = format!("{}{}{}Action",
                                            syn_to_string(&event_ty),
                                            syn_to_string(&transition_from.clone().unwrap()),
                                            syn_to_string(&transition_to.clone().unwrap())
                                        );
                                                                        
                                        let ty: syn::Type = syn::parse_str(&transition_action_name).unwrap();
                                        action = Some(ty.clone());

                                        inline_actions.push(FsmInlineAction {
                                            ty: ty,
                                            action_closure: Some(action_closure),
                                            transition_id: transition_id
                                        });
                                    },
                                    TransitionType::Internal | TransitionType::SelfTransition => {
                                        let transition_action_name = format!("{}{}{}",
                                            syn_to_string(&event_ty),
                                            syn_to_string(&transition_from.clone().unwrap()),
                                            match transition_type {
                                                TransitionType::Internal => "InternalAction",
                                                TransitionType::SelfTransition => "SelfAction",
                                                _ => panic!("nope")
                                            }
                                        );
                                                                        
                                        let ty: syn::Type = syn::parse_str(&transition_action_name).unwrap();
                                        action = Some(ty.clone());

                                        inline_actions.push(FsmInlineAction {
                                            ty: ty,
                                            action_closure: Some(action_closure),
                                            transition_id: transition_id
                                        });
                                    }
                                }                                
                            },
                            "guard" => {
                                let transition_guard_name = format!("{}{}{}Guard",
                                    syn_to_string(&event_ty),
                                    syn_to_string(&transition_from.clone().unwrap()),
                                    syn_to_string(&transition_to.clone().unwrap())
                                );

                                let guard_closure = if let syn::Expr::Closure(ref closure) = call.args[0] {
                                    closure.clone()
                                } else {
                                    panic!("missing closure?");
                                };

                                let ty: syn::Type = syn::parse_str(&transition_guard_name).unwrap();
                                guard = Some(ty.clone());

                                inline_guards.push(FsmInlineGuard {
                                    ty: ty,
                                    guard_closure: Some(guard_closure),
                                    transition_id: transition_id
                                });
                            }
                            &_ => { }
                        }
                    }

                    if transition_entry {
                        let entry = TransitionEntry {
                            id: transition_id,
                            source_state: transition_from.clone().expect("Missing source state?"),
                            event: event_ty,
                            target_state: match transition_type {
                                TransitionType::Normal => { transition_to.expect("Missing target state?") },
                                _ => { transition_from.expect("Missing source state?") }
                            },
                            action: action.unwrap_or(syn::parse_str("NoAction").unwrap()),
                            transition_type: transition_type,
                            guard: guard
                        };

                        transitions.push(entry);

                        transition_id += 1;
                    }
                }
            }
        }

    }

    let submachines: Vec<_> = inline_submachines.iter().map(|s| s.ty.clone()).collect();

    let inline_structs = ::parse_fn_visitors::find_inline_structs(fn_body, &fsm_decl);

    let (generics, runtime_generics) = {
        use syn::*;

        let generics = fn_body.decl.generics.clone();
        
        let mut g = generics.clone();
        //g.params.clear();
        //g.params = syn::punctuated::Punctuated::new();

        let gt = g.clone();
        let (impl_generics, ty_generics, where_clause) = gt.split_for_impl();

        //panic!("g: {:#?}", g);

        //let fsm_ty: syn::Type = syn::parse_str(&format!("{} {}", fsm_name, syn_to_string(&ty_generics))).unwrap();
        //panic!("fsm_ty: {:#?}", fsm_ty);

        let all_fsm_types = {
            //let mut f = vec![syn_to_string(&fsm_ty)];
            let mut f = vec![syn_to_string(&fsm_decl.fsm_ty)];
            //let mut f = vec![];
            f.extend(submachines.iter().map(|sub| syn_to_string(sub)));
            f
        };
        
        g.params.push(TypeParam {
            attrs: vec![],
            ident: "FI".into(),
            bounds: {
                let mut p = syn::punctuated::Punctuated::new();
                for t in all_fsm_types.iter().map(|t| {
                    let t = syn::parse_str(&format!("::fsm::FsmInspect<{}>", t)).unwrap();
                    t
                })
                {
                    p.push(t);
                }
                p
            },
            default: None,
            colon_token: Default::default(),
            eq_token: Default::default()
        }.into());

        g.params.push(TypeParam {
            attrs: vec![],
            ident: "FT".into(),
            bounds: {
                let mut p = syn::punctuated::Punctuated::new();
                p.push(syn::parse_str(&"::fsm::FsmTimers").unwrap());
                p
            },
            default: None,
            colon_token: Default::default(),
            eq_token: Default::default()
        }.into());

        (generics, g)
    };
    


    let regions = create_regions(&transitions,
                                 &ty_to_vec(&fsm_decl.fsm_initial_state_ty),
                                 &submachines,
                                 &interrupt_states
                                );


    FsmDescription {
        fsm_ty: syn::parse_str(&fsm_name).unwrap(),
        name: fsm_name.into(),
        name_ident: fsm_name_ident,        
        generics: generics,
        runtime_generics: runtime_generics,

        timeout_timers: timeout_timers,

        inline_states: inline_states,
        inline_actions: inline_actions,
        inline_guards: inline_guards,
        inline_structs: inline_structs,
        inline_events: inline_events,
        inline_submachines: inline_submachines,

        submachines: submachines.clone(),
        shallow_history_events: shallow_history_events,
        
        context_ty: fsm_decl.fsm_ctx_ty,
        regions: regions,

        copyable_events: copyable_events
    }    
}