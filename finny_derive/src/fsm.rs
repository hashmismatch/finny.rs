use syn::{WhereClause, spanned::Spanned};

use crate::utils::{get_ty_ident, strip_generics, ty_append};


pub struct FsmTypes {
    fsm: syn::Type,
    fsm_no_generics: syn::Type,
    generics: syn::Generics
}

impl FsmTypes {
    pub fn new(ty: &syn::Type, generics: &syn::Generics) -> Self {
        Self {
            fsm: ty.clone(),
            fsm_no_generics: strip_generics(ty.clone()),
            generics: generics.clone()
        }
    }

    pub fn get_fsm_ty(&self) -> &syn::Type {
        &self.fsm
    }

    pub fn get_fsm_no_generics_ty(&self) -> &syn::Type {
        &self.fsm_no_generics
    }

    pub fn get_fsm_events_ty(&self) -> syn::Type {
        ty_append(&self.fsm_no_generics, "Events")
    }

    /// Extract the relevant generics from self.generics
    pub fn get_generics_for(&self, included_types: &[syn::Type]) -> syn::Result<syn::Generics> {
        let mut generics = syn::Generics::default();

        let mut generic_idents = vec![];
        
        for included_type in included_types {
            match included_type {
                syn::Type::Path(tp) => {
                    for seg in &tp.path.segments {
                        match &seg.arguments {
                            syn::PathArguments::AngleBracketed(g) => {
                                for arg in &g.args {
                                    match arg {
                                        syn::GenericArgument::Lifetime(_) => {}
                                        syn::GenericArgument::Type(ty) => {
                                            let ident = get_ty_ident(ty)?;
                                            generic_idents.push(ident.clone());
                                        }
                                        syn::GenericArgument::Binding(_) => {}
                                        syn::GenericArgument::Constraint(_) => {}
                                        syn::GenericArgument::Const(_) => {}
                                    }
                                }
                            }
                            syn::PathArguments::None => {},                            
                            syn::PathArguments::Parenthesized(_) => {}
                        }
                    }
                },
                _ => {
                    return Err(syn::Error::new(included_type.span(), "Unsupported type."));
                }
            }
        }

        for p in &self.generics.params {
            match p {
                syn::GenericParam::Type(t) => {
                    if generic_idents.contains(&t.ident) {
                        generics.params.push(p.clone());
                    }
                }
                syn::GenericParam::Lifetime(_) => {}
                syn::GenericParam::Const(_) => {}
            }
        }

        generics.lt_token = self.generics.lt_token.clone();
        generics.gt_token = self.generics.gt_token.clone();
        
        //generics.where_clause = self.generics.where_clause.clone();
        if let Some(ref w) = self.generics.where_clause {
            let mut where_clause = WhereClause {
                where_token: w.where_token,
                predicates: syn::punctuated::Punctuated::default()
            };

            for predicate in &w.predicates {
                match predicate {
                    syn::WherePredicate::Type(pt) => {
                        let ident = get_ty_ident(&pt.bounded_ty)?;
                        if generic_idents.contains(&ident) {
                            where_clause.predicates.push(predicate.clone());
                        }
                    }
                    syn::WherePredicate::Lifetime(pl) => {
                        todo!("lifetimes");
                    }
                    syn::WherePredicate::Eq(_) => {
                        todo!("equality");
                    }
                }
            }

            generics.where_clause = Some(where_clause);
        }

        

        //panic!("generics: {:#?}", generics);

        Ok(generics)
    }
}
