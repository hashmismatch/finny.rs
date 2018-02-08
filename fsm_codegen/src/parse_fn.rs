extern crate quote;
extern crate syn;

use fsm_def::*;
use graph::*;

use parse_fn_visitors::extract_method_generic_ty;

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
                    self.fsm_ty = Some(ty.clone());
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



fn find_inline_states(fn_body: &syn::ItemFn, fsm_decl: &LetFsmDeclaration) -> Vec<FsmInlineState> {
    #[derive(Debug)]
    struct FindInlineStates<'a> {
        fsm_decl: &'a LetFsmDeclaration,
        calls: Vec<syn::ExprMethodCall>,
        level: usize
    }

    impl<'a, 'ast> syn::visit::Visit<'ast> for FindInlineStates<'a> {
        fn visit_expr_method_call(&mut self, i: &'ast syn::ExprMethodCall) {
            let is_on_fsm = {
                let mut f = IdentFinder {
                    ident: self.fsm_decl.fsm_let_ident.clone(),
                    found: false
                };

                f.visit_expr(&i.receiver);

                f.found
            };

            if is_on_fsm && self.level == 0 {
                //panic!("us: {:#?}", i);
                self.calls.push(i.clone());
            }

            self.level += 1;
            visit_expr_method_call(self, i);
            self.level -= 1;
        }
    }

    let mut finder = FindInlineStates {
        fsm_decl: fsm_decl,
        calls: vec![],
        level: 0
    };
    finder.visit_item_fn(fn_body);

    //panic!("calls ({}): {:#?}", finder.calls.len(), finder.calls);


    #[derive(Debug, Default)]
    struct DecodeInlineState {
        inline_unit_state_ty: Option<syn::Type>,
        inline_state_ty: Option<syn::Type>,
        on_entry_closure: Option<syn::ExprClosure>,
        on_exit_closure: Option<syn::ExprClosure>
    }

    impl<'ast> syn::visit::Visit<'ast> for DecodeInlineState {
        fn visit_expr_method_call(&mut self, i: &'ast syn::ExprMethodCall) {
            if i.method.as_ref() == "new_unit_state" {
                if let Some(ref turbofish) = i.turbofish {
                    if let syn::GenericMethodArgument::Type(ref ty) = turbofish.args[0] {
                        self.inline_unit_state_ty = Some(ty.clone());
                    }
                }
            }

            if i.method.as_ref() == "new_state" {
                if let Some(ref turbofish) = i.turbofish {
                    if let syn::GenericMethodArgument::Type(ref ty) = turbofish.args[0] {
                        self.inline_state_ty = Some(ty.clone());
                    }
                }
            }

            if i.method.as_ref() == "on_entry" {
                if let syn::Expr::Closure(ref closure) = i.args[0] {
                    self.on_entry_closure = Some(closure.clone());
                }
            }

            if i.method.as_ref() == "on_exit" {
                if let syn::Expr::Closure(ref closure) = i.args[0] {
                    self.on_exit_closure = Some(closure.clone());
                }
            }

            visit_expr_method_call(self, i);
        }
    }

    let mut ret = vec![];

    for call in &finder.calls {
        let mut decoder = DecodeInlineState::default();
        decoder.visit_expr_method_call(call);
        
        if let Some(ty) = decoder.inline_unit_state_ty {
            ret.push(FsmInlineState {
                ty: ty.clone(),
                unit: true,
                on_entry_closure: decoder.on_entry_closure.clone(),
                on_exit_closure: decoder.on_exit_closure.clone(),
            });
        }
        if let Some(ty) = decoder.inline_state_ty {
            ret.push(FsmInlineState {
                ty: ty.clone(),
                unit: false,
                on_entry_closure: decoder.on_entry_closure,
                on_exit_closure: decoder.on_exit_closure,
            });
        }
    }
    
    ret
}




pub fn parse_definition_fn(fn_body: &syn::ItemFn) -> FsmDescription {
    
    let mut copyable_events = false;    
    let mut transitions = vec![];
    let mut submachines = vec![];
    let mut shallow_history_events = vec![];
    let mut interrupt_states: Vec<FsmInterruptState> = vec![];
    let mut timeout_timers = vec![];
    let mut timer_id = 0;
    
    let mut transition_id = 1;


    let fsm_decl = let_fsm_local(fn_body);
        

    let fsm_name = syn_to_string(&fsm_decl.fsm_ty);
    let fsm_name_ident = syn::Ident::from(fsm_name.clone());


    let inline_states = find_inline_states(fn_body, &fsm_decl);
    let mut inline_actions = vec![];
    let mut inline_guards = vec![];
    let mut inline_events = vec![];

    //let mut inline_states = vec![];

    /*
    inline_states.push(FsmInlineState {
        ty: syn::parse_str("StaticA").unwrap()
    });
    */

    {        
        let method_calls = ::parse_fn_visitors::find_fsm_method_calls(fn_body, &fsm_decl);        
        for st in &method_calls {
            //panic!("calls: {:#?}", st.calls.iter().map(|m| m.method).collect::<Vec<_>>());

            if let Some(first) = st.calls.get(0) {
                if first.method.as_ref() == "new_event" {
                    let event_ty = extract_method_generic_ty(first);

                    inline_events.push(FsmInlineEvent {
                        ty: event_ty
                    });
                } else if first.method.as_ref() == "copyable_events" {
                    copyable_events = true;
                } else if first.method.as_ref() == "on_event" {
                    let event_ty = extract_method_generic_ty(first);
                    let mut transition_type = TransitionType::Normal;
                    let mut transition_from = None;
                    let mut transition_to = None;
                    let mut action = None;
                    let mut guard = None;

                    for call in &st.calls[1..] {
                        match call.method.as_ref() {
                            "transition_internal" => {
                                transition_type = TransitionType::Internal;
                                transition_from = Some(extract_method_generic_ty(&call));
                            },
                            "transition_self" => {
                                transition_type = TransitionType::SelfTransition;
                                transition_from = Some(extract_method_generic_ty(&call));
                            },
                            "transition_from" => {
                                transition_from = Some(extract_method_generic_ty(&call));
                            },
                            "to" => {
                                transition_to = Some(extract_method_generic_ty(&call));
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

    let inline_structs = ::parse_fn_visitors::find_inline_structs(fn_body, &fsm_decl);

    let (generics, runtime_generics) = {
        use syn::*;

        let generics: syn::Generics = Default::default();

        //let mut g: syn::Generics = ast.generics.clone();
        let mut g = generics.clone();
        let gt = g.clone();
        let (impl_generics, ty_generics, where_clause) = gt.split_for_impl();

        let fsm_ty: syn::Type = syn::parse_str(&format!("{} {}", fsm_name, syn_to_string(&ty_generics))).unwrap();

        let all_fsm_types = {
            let mut f = vec![syn_to_string(&fsm_ty)];
            f.extend(submachines.iter().map(|sub| syn_to_string(sub)));
            f
        };
        
        g.params.push(TypeParam {
            attrs: vec![],
            ident: "FI".into(),
            bounds: {
                let mut p = syn::punctuated::Punctuated::new();
                for t in all_fsm_types.iter().map(|t| {
                    let t = syn::parse_str(&format!("FsmInspect<{}>", t)).unwrap();
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
                p.push(syn::parse_str(&"FsmTimers").unwrap());
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

        submachines: submachines,
        shallow_history_events: shallow_history_events,
        
        context_ty: fsm_decl.fsm_ctx_ty,
        regions: regions,

        copyable_events: copyable_events
    }    
}