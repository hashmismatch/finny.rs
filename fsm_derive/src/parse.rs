use proc_macro2::TokenStream;
use syn::{parse::{self, Parse, ParseStream}, spanned::Spanned};

use crate::fsm_fn;


pub struct FsmFnInput {
    pub context_ty: syn::Type,
    pub fsm_ty: syn::Type
}

impl FsmFnInput {
    pub fn parse(attr: TokenStream, item: TokenStream) -> syn::Result<Self> {

        let input_fn: syn::ItemFn = syn::parse2(item)?;

        // generics check
        if input_fn.sig.generics.params.len() > 0 {
            return Err(syn::Error::new(input_fn.sig.generics.span(), "Generics aren't supported!"));
        }

        // input type check
        // let (context_ty, fsm_ty) = {
        let mut context_ty = None;
        let mut fsm_ty = None;

        {
            if input_fn.sig.inputs.len() != 1 {
                return Err(syn::Error::new(input_fn.sig.inputs.span(), "Only a single input parameter is supported!"));
            }

            if let Some(p) = input_fn.sig.inputs.first() {
                match *p {
                    syn::FnArg::Receiver(ref r) => {
                        
                    }
                    syn::FnArg::Typed(ref r) => {

                        match *r.pat {
                            syn::Pat::Ident(ref p) => {
                                if p.ident != "fsm" {
                                    return Err(syn::Error::new(p.ident.span(), "The name of the input arg has to be 'fsm'."));
                                }

                                match *r.ty {
                                    syn::Type::Path(ref p) => {
                                        if p.path.segments.len() != 1 {
                                            panic!("seg 1");
                                        } else {
                                            let s = p.path.segments.first().unwrap();
                                            match s.arguments {
                                                syn::PathArguments::AngleBracketed(ref args) => {
                                                    if args.args.len() != 2 {
                                                        panic!("nop");
                                                    }

                                                    for (i, a) in args.args.iter().enumerate() {
                                                        match a {
                                                            syn::GenericArgument::Type(ty) => {
                                                                if i == 0 {
                                                                    fsm_ty = Some(ty.clone());
                                                                } else if i == 1 {
                                                                    context_ty = Some(ty.clone());
                                                                }
                                                            }
                                                            _ => panic!("3")
                                                        }
                                                    }
                                                },
                                                _ => panic!("nop")
                                            }
                                        }
                                    }
                                    _ => {
                                        return Err(syn::Error::new(p.ident.span(), "The name of the input arg has to be 'fsm'."));
                                    }
                                }
                            }
                            _ => {
                                return Err(syn::Error::new(r.pat.span(), "The name of the input arg has to be 'fsm'."));
                            }
                        }
                    }
                }
            } else {
                panic!("foo");
                //return Err(syn::Error::new(r.pat.span(), "The name of the input arg has to be 'fsm'."));
            }
        };


        // return type check
        {
            let mut ok = false;

            match input_fn.sig.output {
                syn::ReturnType::Type(_, ref ty) => {
                    match *ty.clone() {
                        syn::Type::Path(tp) => {
                            if let Some(ident) = tp.path.get_ident() {
                                if ident == "BuiltFsm" {
                                    ok = true;
                                }
                            }
                        },
                        _ => ()
                    }
                },
                _ => ()
            }

            if !ok {
                return Err(syn::Error::new(input_fn.sig.output.span(), "The return type has to be 'BuiltFsm'!"));
            }
        }

        
        Ok(FsmFnInput {
            context_ty: context_ty.expect("Failed to get context Ty"),
            fsm_ty: fsm_ty.expect("Failed to get FSM ty")
         })
    }
}
