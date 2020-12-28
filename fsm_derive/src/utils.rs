use proc_macro2::TokenStream;
use quote::quote;
use syn::{PatIdent, spanned::Spanned};


pub fn remap_closure_inputs(inputs: &syn::punctuated::Punctuated<syn::Pat, syn::token::Comma>, access: &[TokenStream]) -> syn::Result<TokenStream> {
    if inputs.len() != access.len() {
        panic!("Expected {} closure arguments, actually have {}.", access.len(), inputs.len());
    }
    
    let input_remap: syn::Result<Vec<_>> = inputs.iter().enumerate().map(|(idx, input)| {
        
        match input {
            
            
            // syn::FnArg::Inferred(ref pat) => {
            //     let arg = pat;
            //     if let Some(rep) = p.get(idx) {
            //         quote! { let #arg = #rep; }
            //     } else {
            //         panic!("unsupported number of args");
            //     }
            // },
            // _ => { panic!("unsupported closure arg"); }
            
            
            syn::Pat::Ident(PatIdent { ident, .. }) => {
                let rep = &access[idx];
                let q = quote! {
                    let #ident = #rep;
                };
                Ok(q)
            }
            _ => { Err(syn::Error::new(input.span(), "Unsupported closure input.")) }
        }
    }).collect();
    let input_remap = input_remap?;
    
    let q = quote! {
        #(#input_remap)*
    };
    Ok(q)
}