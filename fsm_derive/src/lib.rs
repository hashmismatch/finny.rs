extern crate proc_macro;
extern crate proc_macro2;

extern crate syn;
#[macro_use]
extern crate quote;

use parse::FsmFnInput;
use proc_macro::TokenStream;
use quote::{TokenStreamExt, quote_token_with_context};
use syn::{parse::{Parse, ParseStream}, parse_macro_input};

mod codegen;
mod parse;
mod parse_statements;

#[proc_macro_attribute]
pub fn fsm_fn(attr: TokenStream, item: TokenStream) -> TokenStream {
    let item2: proc_macro2::TokenStream = item.into();

    let parsed = match FsmFnInput::parse(attr.into(), item2.clone()) {
        Ok(p) => p,
        Err(e) => return e.to_compile_error().into()
    };

    let fsm_ty = parsed.base.fsm_ty;
    let ctx_ty = parsed.base.context_ty;
    let mut q = quote! {
        pub struct #fsm_ty {

        }

        impl crate::fsm_core::Fsm for #fsm_ty {
            type Context = #ctx_ty;
        }
    };

    // this goes in front of our definition function
    q.append_all(quote! {
        #[allow(dead_code)]
    });

    q.append_all(item2);

    q.into()
}
