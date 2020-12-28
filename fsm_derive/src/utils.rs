use proc_macro2::TokenStream;
use quote::quote;
use syn::{PatIdent, spanned::Spanned};


pub fn remap_closure_inputs(inputs: &syn::punctuated::Punctuated<syn::Pat, syn::token::Comma>, access: &[TokenStream]) -> syn::Result<TokenStream> {
    if inputs.len() != access.len() {
        panic!("Expected {} closure arguments, actually have {}.", access.len(), inputs.len());
    }
    
    let input_remap: syn::Result<Vec<_>> = inputs.iter().enumerate().map(|(idx, input)| {
        match input {
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

pub fn to_field_name(ty: &syn::Type) -> syn::Result<syn::Ident> {
    let s = tokens_to_string(ty);
    let snake = to_snake_case(&s);
    Ok(syn::Ident::new(&snake, ty.span()))
}

pub fn tokens_to_string<T: quote::ToTokens>(t: &T) -> String {
    let mut tokens = TokenStream::new();
    t.to_tokens(&mut tokens);
    tokens.to_string()
}

// From rustc
pub fn to_snake_case(mut str: &str) -> String {
    let mut words = vec![];
    // Preserve leading underscores
    str = str.trim_start_matches(|c: char| {
        if c == '_' {
            words.push(String::new());
            true
        } else {
            false
        }
    });
    for s in str.split('_') {
        let mut last_upper = false;
        let mut buf = String::new();
        if s.is_empty() {
            continue;
        }
        for ch in s.chars() {
            if !buf.is_empty() && buf != "'" && ch.is_uppercase() && !last_upper {
                words.push(buf);
                buf = String::new();
            }
            last_upper = ch.is_uppercase();
            buf.extend(ch.to_lowercase());
        }
        words.push(buf);
    }
    words.join("_")
}
