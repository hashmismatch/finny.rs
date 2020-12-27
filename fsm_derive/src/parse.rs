use proc_macro2::TokenStream;
use syn::{Error, GenericArgument, parse::{self, Parse, ParseStream}, spanned::Spanned};

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

        // builder name/generics
        let (builder_ident, fsm_ty, context_ty) = {
            let input_fsm_builder = match (input_fn.sig.inputs.len(), input_fn.sig.inputs.first()) {
                (1, Some(p)) => {
                    Ok(p)
                },
                (_, _) => {
                    Err(Error::new(input_fn.sig.inputs.span(), "Only a single input parameter is supported!"))
                }
            }?;

            let builder_input = match input_fsm_builder {                
                syn::FnArg::Typed(pt) => Ok(pt),
                _ => Err(Error::new(input_fsm_builder.span(), "Only a typed input is supported!"))
            }?;

            let builder_input_pat_ident = match *builder_input.pat {
                syn::Pat::Ident(ref pi) => Ok(pi),
                _ => Err(Error::new(builder_input.pat.span(), "Only a type ascripted input arg is supported!"))
            }?;
            
            let builder_input_type = match *builder_input.ty {
                syn::Type::Path(ref type_path) => Ok(type_path),
                _ => Err(Error::new(builder_input.ty.span(), "The builder's type is incorrect!"))
            }?;

            let path_segment = match (builder_input_type.path.segments.len(), builder_input_type.path.segments.first()) {
                (1, Some(s)) => Ok(s),
                (_, _) => Err(Error::new(builder_input_type.path.segments.span(), "Only one segment is supported!"))
            }?;

            let generic_arguments = match &path_segment.arguments {
                syn::PathArguments::AngleBracketed(g) => Ok(g),
                _ => Err(Error::new(path_segment.arguments.span(), "Only one segment is supported!"))
            }?;


            let generic_tys: Vec<_> = generic_arguments.args.iter().collect();

            let (fsm_ty, context_ty) = match (generic_tys.get(0), generic_tys.get(1)) {
                (Some(GenericArgument::Type(fsm_ty)), Some(GenericArgument::Type(context_ty))) => {
                    Ok((fsm_ty, context_ty))
                },
                _ => Err(Error::new(generic_arguments.args.span(), "Expected a pair of generic arguments!"))
            }?;

            (builder_input_pat_ident.ident.clone(), fsm_ty.clone(), context_ty.clone())
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
            context_ty: context_ty,
            fsm_ty: fsm_ty
         })
    }
}
