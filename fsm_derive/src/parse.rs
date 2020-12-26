use proc_macro2::TokenStream;
use syn::parse::{Parse, ParseStream};


pub struct FsmFnInput {
    
}

impl FsmFnInput {
    pub fn parse(attr: TokenStream, item: TokenStream) -> syn::Result<Self> {
        Ok(FsmFnInput { })
    }
}
