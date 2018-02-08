#![recursion_limit = "256"]
#![feature(proc_macro)]

extern crate proc_macro;
extern crate fsm;

extern crate petgraph;

#[macro_use]
extern crate itertools;

use itertools::Itertools;

use fsm::*;
use proc_macro::TokenStream;
use quote::ToTokens;

extern crate syn;

#[macro_use]
extern crate quote;

use std::ops::*;


mod codegen;
mod codegen_info;
mod fsm_def;
mod parse;
mod parse_fn;
mod parse_fn_visitors;
mod viz;
mod graph;

use codegen::*;
use parse::*;
use parse_fn::*;
use fsm_def::*;
use viz::*;



#[proc_macro_derive(Fsm)]
pub fn derive_fsm(input: TokenStream) -> TokenStream {    
    let ast: syn::DeriveInput = syn::parse(input).expect("failed to parse input");

    let desc = parse_description(&ast);
    
    let enums = build_enums(&desc);        
    let main = build_main_struct(&desc);
    let state_store = build_state_store(&desc);

    let viz_test = build_test_viz_build(&desc);

    let q = quote! {
        #enums
        #state_store
        #main

        #viz_test
    };

    //panic!("q: {:?}", q.to_string());
    //panic!("main: {:?}", main);

    //let q = q.to_string().parse().unwrap();    
    //q

    q.into()

    //q.to_string().parse().unwrap()

    //quote!(#fsm).to_string().parse().unwrap()        
}

#[proc_macro_attribute]
pub fn fsm_fn(attr: TokenStream, item: TokenStream) -> TokenStream {
    let fn_body: syn::ItemFn = syn::parse(item).unwrap();
    

    let desc = parse_definition_fn(&fn_body);
    
    let inline_states = build_inline_states(&desc);
    let inline_actions = build_inline_actions(&desc);
    let inline_guards = build_inline_guards(&desc);
    let inline_structs = build_inline_structs(&desc);
    let inline_events = build_inline_events(&desc);
    let enums = build_enums(&desc);    
    let main = build_main_struct(&desc);
    let state_store = build_state_store(&desc);

    let viz_test = build_test_viz_build(&desc);

    let q = quote! {  
        #inline_structs      
        #inline_states
        #inline_actions
        #inline_guards
        #inline_events
        #enums
        #state_store
        #main

        #viz_test
    };


    q.into()

    /*
    //panic!("attr: {:?}", attr);
    let ast = syn::parse_token_trees(&item.to_string()).unwrap();
    panic!("ast: {:?}", ast);
    //panic!("item: {:?}", item);
    item
    */
}
