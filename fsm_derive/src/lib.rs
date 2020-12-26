extern crate proc_macro;
extern crate proc_macro2;

extern crate syn;
#[macro_use]
extern crate quote;

use parse::FsmFnInput;
use proc_macro::TokenStream;
use quote::quote_token_with_context;
use syn::{parse::{Parse, ParseStream}, parse_macro_input};

mod codegen;
mod parse;

#[proc_macro_attribute]
pub fn fsm_fn(attr: TokenStream, item: TokenStream) -> TokenStream {
    let parsed = match FsmFnInput::parse(attr.into(), item.clone().into()) {
        Ok(p) => p,
        Err(e) => return e.to_compile_error().into()
    };

    let q = quote! {
        // hello
    };

    //q.into()
    item.into()
}
