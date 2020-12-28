use proc_macro2::{Span, TokenStream};
use quote::{TokenStreamExt, quote};
use syn::spanned::Spanned;

use crate::parse::FsmFnInput;



pub fn generate_fsm_code(fsm: &FsmFnInput, attr: TokenStream, input: TokenStream) -> syn::Result<TokenStream> {
    let fsm_ty = &fsm.base.fsm_ty;
    let fsm_impl_ty = syn::Ident::new("FsmImpl", fsm_ty.span());
    let ctx_ty = &fsm.base.context_ty;
    let states_store_ty = syn::Ident::new("States", fsm_ty.span());
    let states_enum_ty = syn::Ident::new("StatesEnum", fsm_ty.span());
    let event_enum_ty = syn::Ident::new("FsmEvents", fsm_ty.span());

    let states_store = {

        let mut code_fields = TokenStream::new();
        let mut new_state_fields = TokenStream::new();
        let mut state_variants = TokenStream::new();

        for (i, (_, state)) in fsm.decl.states.iter().enumerate() {
            let name = syn::Ident::new(&format!("state_{}", i), Span::call_site());
            let ty = &state.ty;
            code_fields.append_all(quote! { #name: #ty, });
            new_state_fields.append_all(quote! { #name: #ty::new_state(context)?, });
            state_variants.append_all(quote!{ #ty, });
        }

        quote! {
            pub struct #states_store_ty {
                #code_fields
            }

            impl crate::fsm_core::FsmStateFactory for #states_store_ty {
                fn new_state<C>(context: &C) -> crate::fsm_core::FsmResult<Self> {
                    let s = Self {
                        #new_state_fields
                    };
                    Ok(s)
                }
            }

            #[derive(Debug, PartialEq)]
            pub enum #states_enum_ty {
                #state_variants
            }

            impl crate::fsm_core::FsmStates for #states_store_ty {
                type StateKind = #states_enum_ty;
            }
        }
    };

    let events_enum = {

        let mut variants = TokenStream::new();
        for (ty, ev) in  fsm.decl.events.iter() {
            variants.append_all(quote! { #ty ( #ty ),  });
        }

        quote! {
            pub enum #event_enum_ty {
                #variants
            }
        }

    };

    let dispatch = {

        quote! {

            // todo: which one is really needed?

            impl<Q> crate::fsm_core::FsmCore for #fsm_impl_ty<Q> {
                type Context = #ctx_ty;
                type States = #states_store_ty;
                type Events = #states_enum_ty;
            }

            impl crate::fsm_core::FsmCore for #fsm_ty {
                type Context = #ctx_ty;
                type States = #states_store_ty;
                type Events = #states_enum_ty;
            }

            /*
            impl<Q> crate::fsm_core::FsmCoreDispatch for crate::fsm_core::FsmCoreImpl<#ctx_ty, #states_store_ty, #states_enum_ty, Q>
                where Self: crate::fsm_core::FsmCore<Context = #ctx_ty>
            {
                fn dispatch(&mut self, event: &Self::Events) -> crate::fsm_core::FsmResult<()> {

                    let ev_ctx: crate::fsm_core::EventContext<Self> = crate::fsm_core::EventContext {
                        context: &mut self.context
                    };

                    todo!("foo")
                }
            }
             */
        }
    };

    let builder = {
        quote! {
            pub struct #fsm_impl_ty<Q> {
                fsm: crate::fsm_core::FsmCoreImpl<#ctx_ty, #states_store_ty, #states_enum_ty, Q>
            }

            pub struct #fsm_ty;

            impl #fsm_ty {
                pub fn new(context: #ctx_ty) -> crate::fsm_core::FsmResult<#fsm_impl_ty<crate::fsm_core::FsmEventQueueVec<#states_enum_ty>>>
                {
                    use crate::fsm_core::FsmStateFactory;

                    let queue = crate::fsm_core::FsmEventQueueVec::new();
                    let states = #states_store_ty::new_state(&context)?;

                    let fsm_impl = crate::fsm_core::FsmCoreImpl::new_all(context, states, queue)?;

                    let fsm = #fsm_impl_ty { fsm: fsm_impl };

                    Ok(fsm)
                }
            }
        }
    };

    let mut q = quote! {
        #states_store

        #events_enum

        #dispatch

        #builder
    };

    // this goes in front of our definition function
    q.append_all(quote! {
        #[allow(dead_code)]
    });

    q.append_all(attr);
    q.append_all(input);

    Ok(q.into())
}