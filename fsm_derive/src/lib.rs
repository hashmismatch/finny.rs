extern crate proc_macro;
extern crate proc_macro2;

extern crate syn;
#[macro_use]
extern crate quote;

use codegen::generate_fsm_code;
use parse::FsmFnInput;
use proc_macro::TokenStream;
use quote::{TokenStreamExt, quote_token_with_context};
use syn::{parse::{Parse, ParseStream}, parse_macro_input};

mod codegen;
mod parse;
mod parse_blocks;
//mod parse_statements;

#[proc_macro_attribute]
pub fn fsm_fn(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr2: proc_macro2::TokenStream = attr.into();
    let item2: proc_macro2::TokenStream = item.into();

    let parsed = match FsmFnInput::parse(attr2.clone(), item2.clone()) {
        Ok(p) => p,
        Err(e) => return e.to_compile_error().into()
    };

    match generate_fsm_code(&parsed, attr2.clone(), item2.clone()) {
        Ok(t) => t.into(),
        Err(e) => e.to_compile_error().into()
    }
}
