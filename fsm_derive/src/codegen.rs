use proc_macro2::{Span, TokenStream};
use quote::TokenStreamExt;

use crate::parse::FsmFnInput;



pub fn generate_fsm_code(fsm: &FsmFnInput, attr: TokenStream, input: TokenStream) -> syn::Result<TokenStream> {
    let fsm_ty = &fsm.base.fsm_ty;
    let ctx_ty = &fsm.base.context_ty;
    let states_store_ty = syn::Ident::new("States", Span::call_site());

    let states_store = {

        let mut code_fields = TokenStream::new();
        let mut new_state_fields = TokenStream::new();

        for (i, (_, state)) in fsm.decl.states.iter().enumerate() {
            let name = syn::Ident::new(&format!("state_{}", i), Span::call_site());
            let ty = &state.ty;
            code_fields.append_all(quote! { #name: #ty, });
            new_state_fields.append_all(quote! { #name: #ty::new_state(context), });
        }

        quote! {
            pub struct #states_store_ty {
                #code_fields
            }

            impl crate::fsm_core::FsmStateFactory for #states_store_ty {
                fn new_state<C>(context: &C) -> Self {
                    Self {
                        #new_state_fields
                    }
                }
            }
        }
    };

    //panic!("s: {}", states_store);

    let mut q = quote! {
        pub struct #fsm_ty {
            states: #states_store_ty
        }

        #states_store

        impl crate::fsm_core::Fsm for #fsm_ty {
            type Context = #ctx_ty;
            type States = #states_store_ty;

            fn get_states(&self) -> &Self::States {
                &self.states
            }

            fn get_states_mut(&mut self) -> &mut Self::States {
                &mut self.states
            }
        }
    };

    // this goes in front of our definition function
    q.append_all(quote! {
        #[allow(dead_code)]
    });

    q.append_all(attr);
    q.append_all(input);

    Ok(q.into())
}