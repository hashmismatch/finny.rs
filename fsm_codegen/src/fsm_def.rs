extern crate quote;
extern crate syn;

use quote::*;

use itertools::Itertools;

#[derive(Debug)]
pub struct FsmDescription {
    pub name: String,
    pub name_ident: syn::Ident,
    pub lifetimes: Vec<syn::Lifetime>,

    pub submachines: Vec<syn::Ty>,
    pub shallow_history_events: Vec<ShallowHistoryEvent>,

    pub regions: Vec<FsmRegion>,
    pub context_ty: syn::Ty,
    pub inspect_ty: Option<syn::Ty>,

    pub copyable_events: bool
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
    pub fn get_fsm_ty(&self) -> syn::Ty {
        if self.lifetimes.len() > 0 {
            syn::parse_type(&format!("{}<{}>", &self.name, self.lifetimes[0].ident.as_ref())).unwrap()
        } else {
            syn::parse_type(&self.name).unwrap()
        }
    }

    pub fn get_fsm_ty_inline(&self) -> syn::Ty {        
        syn::parse_type(&self.name).unwrap()
    }

    pub fn get_impl_suffix(&self) -> quote::Tokens {
        if self.lifetimes.len() == 0 {
            quote! {}
        } else {
            let l = &self.lifetimes[0];
            quote! { <#l> }
        }
    }

    pub fn get_events_ty(&self) -> syn::Ty {
        syn::parse_type(&format!("{}Events", self.name)).unwrap()
    }

    pub fn get_states_ty(&self) -> syn::Ty {
        syn::parse_type(&format!("{}States", self.name)).unwrap()
    }

    pub fn get_current_state_ty(&self) -> syn::Ty {
        let mut q = quote::Tokens::new();
        q.append("(");
        for (i, region) in self.regions.iter().enumerate() {
            q.append(&ty_to_string(&self.get_states_ty()));
            if i < self.regions.len() - 1 {
                q.append(",");
            }
        }
        q.append(")");
        syn::parse_type(&q.as_str()).unwrap()
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

    pub fn get_inspection_ty(&self) -> syn::Ty {
        if let Some(ref ty) = self.inspect_ty {
            ty.clone()
        } else {
            let mut t = quote! {};
            self.get_fsm_ty().to_tokens(&mut t);            
            syn::parse_type(&format!("FsmInspectNull<{}>", t.as_str())).unwrap()
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

    pub fn get_all_internal_states(&self) -> Vec<syn::Ty> {
        // warning: quadratic!
        self.get_all_states().iter().filter(|ref x| !self.is_submachine(x)).cloned().collect()
    }

    pub fn to_state_field_name(state: &syn::Ty) -> syn::Ty {
        let t = ty_to_string(state).to_lowercase();
        syn::parse_type(&t).unwrap()
    }

    pub fn to_state_sub_started_field_name(state: &syn::Ty) -> syn::Ty {
        let t = &format!("{}_started", ty_to_string(state).to_lowercase());
        syn::parse_type(&t).unwrap()
    }

    pub fn has_multiple_regions(&self) -> bool {
        self.regions.len() > 1
    }
}

pub fn ty_to_string(ty: &syn::Ty) -> String {
    let mut t = quote::Tokens::new();
    ty.to_tokens(&mut t);
    t.as_str().into()
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



