use proc_macro2::{Span, TokenStream};
use quote::{TokenStreamExt, quote};
use syn::spanned::Spanned;
use utils::remap_closure_inputs;

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
                        
            impl crate::fsm_core::FsmCore for #fsm_ty {
                type Context = #ctx_ty;
                type States = #states_store_ty;
                type Events = #states_enum_ty;
            }

            impl<Q> crate::fsm_core::FsmCore for #fsm_impl_ty<Q> {
                type Context = #ctx_ty;
                type States = #states_store_ty;
                type Events = #states_enum_ty;
            }

            impl<Q> crate::fsm_core::FsmCoreDispatch for #fsm_impl_ty<Q> {
                fn dispatch<'a>(&'a mut self, event: &Self::Events) -> crate::fsm_core::FsmResult<()> {

                    let context = crate::fsm_core::EventContext::<#fsm_ty> {
                        context: &mut self.fsm.context
                    };

                    match self.current_state {
                        Some( #states_enum_ty :: StateA ) => {
                            use crate::fsm_core::FsmState;
                            let state = &mut self.fsm.states.state_0;
                            <StateA>::on_entry(state, &context);
                        },
                        _ => {}
                    }

                    Ok(())
                }
            }
        }
    };

    let states = {

        let mut states = TokenStream::new();
        for (ty, state) in fsm.decl.states.iter() {

            
            let on_entry = if let Some(c) = &state.on_entry_closure {
                let remap = remap_closure_inputs(&c.inputs, &vec![ quote! { self }, quote! { context } ])?;
                let b = &c.body;
                
                quote! {                                        
                    #remap
                    {
                        #b
                    }
                }
            } else {
                TokenStream::new()
            };

            states.append_all(quote! {

                impl crate::fsm_core::FsmState<#fsm_ty> for #ty {
                    fn on_entry<'a>(&mut self, context: &crate::fsm_core::EventContext<'a, #fsm_ty>) {
                        #on_entry
                    }

                    fn on_exit<'a>(&mut self, context: &crate::fsm_core::EventContext<'a, #fsm_ty>) {

                    }
                }

            })
        }

        states
    };

    let builder = {

        let initial_state = &fsm.decl.initial_state;

        quote! {
            pub struct #fsm_impl_ty<Q> {
                fsm: crate::fsm_core::FsmCoreImpl<#ctx_ty, #states_store_ty, #states_enum_ty, Q>,
                current_state: Option< #states_enum_ty >
            }

            pub struct #fsm_ty;

            impl #fsm_ty {
                pub fn new(context: #ctx_ty) -> crate::fsm_core::FsmResult<#fsm_impl_ty<crate::fsm_core::FsmEventQueueVec<#states_enum_ty>>>
                {
                    use crate::fsm_core::FsmStateFactory;

                    let queue = crate::fsm_core::FsmEventQueueVec::new();
                    let states = #states_store_ty::new_state(&context)?;

                    let fsm_impl = crate::fsm_core::FsmCoreImpl::new_all(context, states, queue)?;

                    let fsm = #fsm_impl_ty {
                        fsm: fsm_impl,
                        current_state: None
                    };

                    Ok(fsm)
                }
            }

            impl<Q> #fsm_impl_ty<Q> {
                pub fn start(&mut self) {
                    self.current_state = Some( <#states_enum_ty>:: #initial_state );
                }

                pub fn stop(&mut self) {
                    self.current_state = None;
                }

                pub fn get_context(&self) -> & #ctx_ty {
                    &self.fsm.context
                }
            }
        }
    };

    let mut q = quote! {
        #states_store

        #states

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