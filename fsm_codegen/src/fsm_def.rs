extern crate quote;
extern crate syn;

use quote::*;
use fsm::RegionId;

use itertools::Itertools;

#[derive(Debug)]
pub struct FsmDescription {
    pub name: String,
    pub name_ident: syn::Ident,
    pub fsm_ty: syn::Type,
    pub generics: syn::Generics,

    pub runtime_generics: syn::Generics,

    pub submachines: Vec<syn::Type>,
    pub shallow_history_events: Vec<ShallowHistoryEvent>,

    pub regions: Vec<FsmRegion>,
    pub context_ty: syn::Type,
    pub inline_states: Vec<FsmInlineState>,
    pub inline_actions: Vec<FsmInlineAction>,

    pub timeout_timers: Vec<FsmTimeoutTimer>,

    pub copyable_events: bool
}

#[derive(Debug, Clone)]
pub struct FsmInlineState {
    pub ty: syn::Type,
    pub on_entry_closure: Option<syn::ExprClosure>,
    pub on_exit_closure: Option<syn::ExprClosure>
}

#[derive(Debug, Clone)]
pub struct FsmInlineAction {
    pub ty: syn::Type,
    pub action_closure: Option<syn::ExprClosure>,
    pub transition_id: u32
}


#[derive(Debug, Clone)]
pub struct FsmTimeoutTimer {
    pub id: u32,
    pub state: syn::Type,
    pub event_on_timeout: syn::Type
}

#[derive(Debug, Clone)]
pub struct FsmInterruptState {
    pub interrupt_state_ty: syn::Type,
    pub resume_event_ty: Vec<syn::Type>
}

#[derive(Debug)]
pub struct FsmRegion {
    pub submachines: Vec<syn::Type>,
    pub id: usize,
    pub transitions: Vec<TransitionEntry>,
    pub initial_state_ty: syn::Type,
    pub interrupt_states: Vec<FsmInterruptState>
}

impl FsmRegion {
    pub fn get_all_states(&self) -> Vec<syn::Type> {
        self.transitions.iter().map(|ref x| &x.source_state).chain(self.transitions.iter().map(|ref x| &x.target_state)).unique_by(|x| *x).cloned().collect()
    }
    
    pub fn get_all_internal_states(&self) -> Vec<syn::Type> {
        // warning: quadratic!
        self.get_all_states().iter().filter(|ref x| !self.is_submachine(x)).cloned().collect()
    }

    pub fn is_submachine(&self, ty: &syn::Type) -> bool {
        self.submachines.iter().find(|x| x == &ty).is_some()
    }
}


#[derive(Debug)]
pub struct ShallowHistoryEvent {
    pub event_ty: syn::Type,
    pub target_state_ty: syn::Type
}

impl ShallowHistoryEvent {
    pub fn get_field_name(&self) -> syn::Type {
        let mut t = quote::Tokens::new();
        self.target_state_ty.to_tokens(&mut t);
        
        syn::parse_str(&format!("history_{}", syn_to_string(&t))).unwrap()
    }

    pub fn get_field_ty(&self) -> syn::Type {
        let mut t = quote::Tokens::new();
        self.target_state_ty.to_tokens(&mut t);

        syn::parse_str(&format!("Option<{}>", syn_to_string(&t))).unwrap()
    }
}

impl FsmDescription {
    pub fn get_fsm_runtime_ty_inline(&self) -> syn::Type {
        syn::parse_str(&format!("{}Runtime", &self.name)).unwrap()
    }

    pub fn get_fsm_ty(&self) -> syn::Type {
        let ty = syn::parse_str(&format!("{} {}", &self.name,
        {
            let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();
            syn_to_string(&ty_generics)
        })).unwrap();
        ty
    }

    pub fn get_fsm_viz_ty(&self) -> syn::Type {
        syn::parse_str(&format!("{}{}", &self.name, "Viz")).unwrap()
    } 

    pub fn get_fsm_ty_inline(&self) -> syn::Type {        
        syn::parse_str(&self.name).unwrap()
    }

    pub fn get_impl_suffix(&self) -> quote::Tokens {
        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();
        syn_to_tokens(&impl_generics)
    }

    pub fn get_fsm_where_ty(&self) -> quote::Tokens {
        let mut q = quote! {};
        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();
        append_to_tokens(&where_clause, &mut q);
        q
    }

    pub fn get_events_ty(&self) -> syn::Type {
        syn::parse_str(&format!("{}Events", self.name)).unwrap()
    }

    pub fn get_event_kind_ty(&self) -> syn::Type {
        syn::parse_str(&format!("{}EventKind", self.name)).unwrap()
    }    

    pub fn get_events_ref_ty(&self) -> syn::Type {
        syn::parse_str(&format!("{}EventsRef", self.name)).unwrap()
    }

    pub fn get_states_ty(&self) -> syn::Type {
        syn::parse_str(&format!("{}States", self.name)).unwrap()
    }

    pub fn get_current_state_ty(&self) -> syn::Type {
        let mut s = String::new();
        s.push_str("(");
        for (i, region) in self.regions.iter().enumerate() {
            s.push_str(&syn_to_string(&self.get_states_ty()));
            if i < self.regions.len() - 1 {
                s.push_str(", ");
            }
        }
        s.push_str(")");
        syn::parse_str(&s).unwrap()
    }

    pub fn get_current_region_state(&self, region_id: RegionId) -> quote::Tokens {
        if self.has_multiple_regions() {
            let region_id = syn::Index::from(region_id as usize);
            quote! { .#region_id }
        } else {
            quote! {}
        }
    }

    pub fn get_states_store_ty(&self) -> syn::Type {
        syn::parse_str(&format!("{}StatesStore", self.name)).unwrap()
    }

    pub fn get_actions_ty(&self) -> syn::Type {
        syn::parse_str(&format!("{}Actions", self.name)).unwrap()
    }

    pub fn get_history_ty(&self) -> syn::Type {
        if self.shallow_history_events.len() == 0 {
            syn::parse_str("()").unwrap()
        } else {
            syn::parse_str(&format!("{}History", self.name)).unwrap()
        }
    }

    pub fn get_build_viz_fn(&self) -> syn::Type {
        syn::parse_str(&format!("build_viz_{}", self.name)).unwrap()
    }

    pub fn get_build_viz_docs_fn(&self) -> syn::Type {
        syn::parse_str(&format!("build_viz_docs_{}", self.name)).unwrap()
    }

    pub fn get_submachine_types(&self) -> &[syn::Type] {
        &self.submachines
    }

    pub fn is_submachine(&self, ty: &syn::Type) -> bool {
        self.get_submachine_types().iter().find(|x| x == &ty).is_some()
    }

    pub fn get_all_transitions(&self) -> Vec<TransitionEntry> {
        self.regions.iter().flat_map(|x| &x.transitions).cloned().collect()
    }

    pub fn get_all_states(&self) -> Vec<syn::Type> {
        self.get_all_transitions().iter().map(|ref x| &x.source_state)
            .chain(self.get_all_transitions().iter().map(|ref x| &x.target_state))
            .chain(self.regions.iter().map(|ref x| &x.initial_state_ty))
            .unique_by(|x| *x).cloned().collect()
    }

    pub fn get_all_events(&self) -> Vec<syn::Type> {
        self.get_all_transitions().iter().map(|ref x| &x.event)            
            .unique_by(|x| *x).cloned().collect()
    }

    pub fn get_all_internal_states(&self) -> Vec<syn::Type> {
        // warning: quadratic!
        self.get_all_states().iter().filter(|ref x| !self.is_submachine(x)).cloned().collect()
    }

    pub fn to_state_field_access(&self, state: &syn::Type) -> Tokens {
        if self.is_submachine(&state) {
            let field_name = format!("fsm_sub_{}", syn_to_string(&state).to_lowercase());
            let field_name: syn::Lit = syn::parse_str(&field_name).unwrap();

            quote! {
                self.#field_name.fsm
            }

        } else {
            let f = Self::to_state_field_name(state);
            quote! {
                self.fsm.states.#f
            }
        }
    }

    pub fn to_sub_runtime(&self, state: &syn::Type) -> Tokens {
        if self.is_submachine(&state) {
            let field_name = format!("fsm_sub_{}", syn_to_string(&state).to_lowercase());
            let field_name: syn::Lit = syn::parse_str(&field_name).unwrap();

            quote! {
                self.#field_name
            }

        } else {
            panic!("not a sub?");
        }
    }    

    pub fn to_state_field_name(state: &syn::Type) -> syn::Type {
        let t = syn_to_string(state).to_lowercase();
        syn::parse_str(&t).unwrap()
    }

    pub fn to_state_sub_started_field_name(state: &syn::Type) -> syn::Type {
        let t = &format!("{}_started", syn_to_string(state).to_lowercase());
        syn::parse_str(&t).unwrap()
    }

    pub fn has_multiple_regions(&self) -> bool {
        self.regions.len() > 1
    }

    pub fn get_fsm_runtime_generics(&self, types: &[(&str, &syn::Type)]) -> syn::Type {
        let mut g = self.runtime_generics.clone();

        for &(i, r) in types {
            if let Some(idx) = g.params.iter().position(|p| {
                if let &syn::GenericParam::Type(ref p) = p {
                    p.ident == i
                } else {
                    false
                }
            }) 
            {
                let gt = &mut g.params[idx];
                if let &mut syn::GenericParam::Type(ref mut gt) = gt {
                    gt.default = Some(r.clone());
                }
            }
        }

        let t = format!("{} < {} >",
            syn_to_string(&self.get_fsm_runtime_ty_inline()),
            {
                let mut parts = vec![];
                'l: for ty in g.params {
                    for &(i, r) in types {
                        if let syn::GenericParam::Type(ref ty) = ty {
                            if ty.ident == i {
                                parts.push(syn_to_string(r));
                                continue 'l;
                            }
                        }
                    }

                    if let syn::GenericParam::Type(ref ty) = ty {
                        parts.push(syn_to_string(&ty.ident));
                    }
                }

                parts.join(", ")
            }
        );

        syn::parse_str(&t).unwrap()
    }

    pub fn has_timers(&self) -> bool {
        self.timeout_timers.len() > 0
    }

    pub fn find_transition(&self, transition_id: u32) -> Option<&TransitionEntry> {
        for region in &self.regions {
            if let Some(t) = region.transitions.iter().find(|t| t.id == transition_id) {
                return Some(t);
            }
        }

        None
    }
}

pub fn syn_to_string<T: ToTokens>(thing: &T) -> String {
    format!("{}", syn_to_tokens(thing))
}

pub fn append_to_tokens<T: ToTokens>(thing: &T, tokens: &mut quote::Tokens) {
    thing.to_tokens(tokens)
}

pub fn syn_to_tokens<T: ToTokens>(thing: &T) -> quote::Tokens {
    let mut t = quote::Tokens::new();
    append_to_tokens(thing, &mut t);
    t
}

/// Deconstruct a potential tuple into a vector of types,
/// otherwise return just the type wrapped into a vector.
pub fn ty_to_vec(ty: &syn::Type) -> Vec<syn::Type> {
    match ty {
        &syn::Type::Tuple(ref t) => t.elems.iter().cloned().collect(),
        t @ _ => vec![t.clone()]
    }
}



#[derive(Debug, Clone)]
pub struct TransitionEntry {
    pub id: u32,
    pub source_state: syn::Type,
    pub event: syn::Type,
    pub target_state: syn::Type,
    pub action: syn::Type,
    pub transition_type: TransitionType,
    pub guard: Option<syn::Type>
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum TransitionType {
    Normal,
    SelfTransition,
    Internal
}

use std::fmt;

impl fmt::Display for TransitionType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            TransitionType::Normal => write!(f, "Normal"),
            TransitionType::SelfTransition => write!(f, "SelfTransition"),
            TransitionType::Internal => write!(f, "Internal")
        }
    }
}

impl TransitionEntry {
    pub fn has_same_states(&self) -> bool {
        self.source_state == self.target_state
    }

    pub fn is_anonymous_transition(&self) -> bool {
        self.event == syn::parse_str("NoEvent").unwrap()
    }
}



