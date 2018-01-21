extern crate quote;
extern crate syn;

use quote::*;
use fsm::RegionId;

use itertools::Itertools;

#[derive(Debug)]
pub struct FsmDescription {
    pub name: String,
    pub name_ident: syn::Ident,
    pub generics: syn::Generics,

    pub runtime_generics: syn::Generics,

    pub submachines: Vec<syn::Ty>,
    pub shallow_history_events: Vec<ShallowHistoryEvent>,

    pub regions: Vec<FsmRegion>,
    pub context_ty: syn::Ty,

    pub timeout_timers: Vec<FsmTimeoutTimer>,

    pub copyable_events: bool
}

#[derive(Debug, Clone)]
pub struct FsmTimeoutTimer {
    pub id: u32,
    pub state: syn::Ty,
    pub event_on_timeout: syn::Ty
}

#[derive(Debug, Clone)]
pub struct FsmInterruptState {
    pub interrupt_state_ty: syn::Ty,
    pub resume_event_ty: Vec<syn::Ty>
}

#[derive(Debug)]
pub struct FsmRegion {
    pub submachines: Vec<syn::Ty>,
    pub id: usize,
    pub transitions: Vec<TransitionEntry>,
    pub initial_state_ty: syn::Ty,
    pub interrupt_states: Vec<FsmInterruptState>
}

impl FsmRegion {
    pub fn get_all_states(&self) -> Vec<syn::Ty> {
        self.transitions.iter().map(|ref x| &x.source_state).chain(self.transitions.iter().map(|ref x| &x.target_state)).unique_by(|x| *x).cloned().collect()
    }
    
    pub fn get_all_internal_states(&self) -> Vec<syn::Ty> {
        // warning: quadratic!
        self.get_all_states().iter().filter(|ref x| !self.is_submachine(x)).cloned().collect()
    }

    pub fn is_submachine(&self, ty: &syn::Ty) -> bool {
        self.submachines.iter().find(|x| x == &ty).is_some()
    }
}


#[derive(Debug)]
pub struct ShallowHistoryEvent {
    pub event_ty: syn::Ty,
    pub target_state_ty: syn::Ty
}

impl ShallowHistoryEvent {
    pub fn get_field_name(&self) -> syn::Ty {
        let mut t = quote::Tokens::new();
        self.target_state_ty.to_tokens(&mut t);
        
        syn::parse_type(&format!("history_{}", t.as_str())).unwrap()
    }

    pub fn get_field_ty(&self) -> syn::Ty {
        let mut t = quote::Tokens::new();
        self.target_state_ty.to_tokens(&mut t);

        syn::parse_type(&format!("Option<{}>", t.as_str())).unwrap()
    }
}

impl FsmDescription {
    pub fn get_fsm_runtime_ty_inline(&self) -> syn::Ty {
        syn::parse_type(&format!("{}Runtime", &self.name)).unwrap()
    }

    pub fn get_fsm_ty(&self) -> syn::Ty {
        let ty = syn::parse_type(&format!("{} {}", &self.name,
        {
            let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();
            syn_to_string(&ty_generics)
        })).unwrap();
        ty
    }

    pub fn get_fsm_viz_ty(&self) -> syn::Ty {
        syn::parse_type(&format!("{}{}", &self.name, "Viz")).unwrap()
    } 

    pub fn get_fsm_ty_inline(&self) -> syn::Ty {        
        syn::parse_type(&self.name).unwrap()
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

    pub fn get_events_ty(&self) -> syn::Ty {
        syn::parse_type(&format!("{}Events", self.name)).unwrap()
    }

    pub fn get_event_kind_ty(&self) -> syn::Ty {
        syn::parse_type(&format!("{}EventKind", self.name)).unwrap()
    }    

    pub fn get_events_ref_ty(&self) -> syn::Ty {
        syn::parse_type(&format!("{}EventsRef", self.name)).unwrap()
    }

    pub fn get_states_ty(&self) -> syn::Ty {
        syn::parse_type(&format!("{}States", self.name)).unwrap()
    }

    pub fn get_current_state_ty(&self) -> syn::Ty {
        let mut q = quote::Tokens::new();
        q.append("(");
        for (i, region) in self.regions.iter().enumerate() {
            q.append(&syn_to_string(&self.get_states_ty()));
            if i < self.regions.len() - 1 {
                q.append(",");
            }
        }
        q.append(")");
        syn::parse_type(&q.as_str()).unwrap()
    }

    pub fn get_current_region_state(&self, region_id: RegionId) -> quote::Tokens {
        let mut q = quote!{};
        if self.has_multiple_regions() {
            q.append(".");
            q.append(&(region_id as usize).to_string());
        }
        q
    }

    pub fn get_states_store_ty(&self) -> syn::Ty {
        syn::parse_type(&format!("{}StatesStore", self.name)).unwrap()
    }

    pub fn get_actions_ty(&self) -> syn::Ty {
        syn::parse_type(&format!("{}Actions", self.name)).unwrap()
    }

    pub fn get_history_ty(&self) -> syn::Ty {
        if self.shallow_history_events.len() == 0 {
            syn::parse_type("()").unwrap()
        } else {
            syn::parse_type(&format!("{}History", self.name)).unwrap()
        }
    }

    pub fn get_build_viz_fn(&self) -> syn::Ty {
        syn::parse_type(&format!("build_viz_{}", self.name)).unwrap()
    }

    pub fn get_build_viz_docs_fn(&self) -> syn::Ty {
        syn::parse_type(&format!("build_viz_docs_{}", self.name)).unwrap()
    }

    pub fn get_submachine_types(&self) -> &[syn::Ty] {
        &self.submachines
    }

    pub fn is_submachine(&self, ty: &syn::Ty) -> bool {
        self.get_submachine_types().iter().find(|x| x == &ty).is_some()
    }

    pub fn get_all_transitions(&self) -> Vec<TransitionEntry> {
        self.regions.iter().flat_map(|x| &x.transitions).cloned().collect()
    }

    pub fn get_all_states(&self) -> Vec<syn::Ty> {
        self.get_all_transitions().iter().map(|ref x| &x.source_state)
            .chain(self.get_all_transitions().iter().map(|ref x| &x.target_state))
            .chain(self.regions.iter().map(|ref x| &x.initial_state_ty))
            .unique_by(|x| *x).cloned().collect()
    }

    pub fn get_all_events(&self) -> Vec<syn::Ty> {
        self.get_all_transitions().iter().map(|ref x| &x.event)            
            .unique_by(|x| *x).cloned().collect()
    }

    pub fn get_all_internal_states(&self) -> Vec<syn::Ty> {
        // warning: quadratic!
        self.get_all_states().iter().filter(|ref x| !self.is_submachine(x)).cloned().collect()
    }

    pub fn to_state_field_access(&self, state: &syn::Ty) -> Tokens {
        if self.is_submachine(&state) {
            let field_name = format!("fsm_sub_{}", syn_to_string(&state).to_lowercase());
            let field_name = syn::parse_type(&field_name).unwrap();

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

    pub fn to_sub_runtime(&self, state: &syn::Ty) -> Tokens {
        if self.is_submachine(&state) {
            let field_name = format!("fsm_sub_{}", syn_to_string(&state).to_lowercase());
            let field_name = syn::parse_type(&field_name).unwrap();

            quote! {
                self.#field_name
            }

        } else {
            panic!("not a sub?");
        }
    }    

    pub fn to_state_field_name(state: &syn::Ty) -> syn::Ty {
        let t = syn_to_string(state).to_lowercase();
        syn::parse_type(&t).unwrap()
    }

    pub fn to_state_sub_started_field_name(state: &syn::Ty) -> syn::Ty {
        let t = &format!("{}_started", syn_to_string(state).to_lowercase());
        syn::parse_type(&t).unwrap()
    }

    pub fn has_multiple_regions(&self) -> bool {
        self.regions.len() > 1
    }

    pub fn get_fsm_runtime_generics(&self, types: &[(&str, &syn::Ty)]) -> syn::Ty {
        let mut g = self.runtime_generics.clone();

        for &(i, r) in types {
            if let Some(idx) = g.ty_params.iter().position(|p| p.ident == i) {
                let ref mut gt = &mut g.ty_params[idx];
                gt.default = Some(r.clone());
            }
        }

        let t = format!("{} < {} >",
            syn_to_string(&self.get_fsm_runtime_ty_inline()),
            {
                let mut parts = vec![];
                'l: for ty in g.ty_params {
                    for &(i, r) in types {
                        if ty.ident == i {
                            parts.push(syn_to_string(r));
                            continue 'l;
                        }
                    }

                    parts.push(syn_to_string(&ty.ident));
                }

                parts.join(", ")
            }
        );

        syn::parse_type(&t).unwrap()
    }

    pub fn has_timers(&self) -> bool {
        self.timeout_timers.len() > 0
    }
}

pub fn syn_to_string<T: ToTokens>(thing: &T) -> String {
    syn_to_tokens(thing).as_str().into()
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
pub fn ty_to_vec(ty: &syn::Ty) -> Vec<syn::Ty> {
    match ty {
        &syn::Ty::Tup(ref t) => t.clone(),
        t @ _ => vec![t.clone()]
    }
}



#[derive(Debug, Clone)]
pub struct TransitionEntry {
    pub id: u32,
    pub source_state: syn::Ty,
    pub event: syn::Ty,
    pub target_state: syn::Ty,
    pub action: syn::Ty,
    pub transition_type: TransitionType,
    pub guard: Option<syn::Ty>
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
        self.event == syn::parse_type("NoEvent").unwrap()
    }
}



