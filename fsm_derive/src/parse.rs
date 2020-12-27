use proc_macro2::TokenStream;
use syn::{Error, Expr, ExprMethodCall, GenericArgument, parse::{self, Parse, ParseStream}, spanned::Spanned};

use crate::{fsm_fn, parse_statements::find_fsm_method_calls};


pub struct FsmFnInput {
    pub base: FsmFnBase,
    options: FsmOptions
}

#[derive(Debug)]
pub struct FsmFnBase {
    pub context_ty: syn::Type,
    pub fsm_ty: syn::Type,
    pub builder_ident: proc_macro2::Ident
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
            let output_ty = match input_fn.sig.output {
                syn::ReturnType::Type(_, ref ty) => Ok(ty),
                _ => Err(syn::Error::new(input_fn.sig.output.span(), "The return type has to be 'BuiltFsm'!"))
            }?;

            let tp = match **output_ty {
                syn::Type::Path(ref tp) => Ok(tp),
                _ => Err(syn::Error::new(output_ty.span(), "The return type has to be 'BuiltFsm'!"))
            }?;

            match tp.path.get_ident() {
                Some(ident) if ident == "BuiltFsm" => Ok(()),
                _ => Err(syn::Error::new(tp.path.span(), "The return type has to be 'BuiltFsm'!"))
            }?
        }

        let base = FsmFnBase {
            builder_ident,
            context_ty,
            fsm_ty
        };

        let options = FsmOptions::parse(&base, &input_fn)?;
        
        Ok(FsmFnInput {
            base,
            options
        })
    }
}

pub struct FsmOptions {
    pub initial_state: syn::Type
}

impl FsmOptions {
    pub fn parse(base: &FsmFnBase, item_fn: &syn::ItemFn) -> syn::Result<Self> {

        //panic!("block: {:?}", item_fn.block);

        //let calls = find_fsm_method_calls(item_fn, base);
        //panic!("calls: {:#?}", calls);

        let blocks = decode_blocks(base, item_fn)?;


        todo!("blocks done")
    }
}

#[derive(Debug)]
pub enum FsmBlock {
    MethodCall(FsmBlockMethodCall),
    Struct()
}
#[derive(Debug)]
pub struct FsmBlockStruct {

}

fn decode_blocks(base: &FsmFnBase, item_fn: &syn::ItemFn) -> syn::Result<Vec<FsmBlock>> {
    let mut ret = vec![];

    for statement in &item_fn.block.stmts {
        //panic!("s: {:?}", statement);

        match statement {
            syn::Stmt::Expr(expr) => {
                let call = decode_method_call(base, expr)?;
                ret.push(FsmBlock::MethodCall(call));
            }
            syn::Stmt::Semi(expr, _col) => {
                let call = decode_method_call(base, expr)?;
                ret.push(FsmBlock::MethodCall(call));
            }
            _ => {
                return Err(syn::Error::new(statement.span(), "Unsupported statement."));
            }
        }
    }

    Ok(ret)
}


#[derive(Debug)]
pub struct FsmBlockMethodCall {
    expr_call: ExprMethodCall
}

fn decode_method_call(base: &FsmFnBase, expr: &Expr) -> syn::Result<FsmBlockMethodCall> {
    // verify if the receiver is our builder
    let mc = match expr {
        Expr::MethodCall(mc) => Ok(mc),
        _ => Err(syn::Error::new(expr.span(), "Unsupported."))
    }?;

    let receiver_ident = get_method_receiver_ident(&mc.receiver)?;
    if receiver_ident != &base.builder_ident {
        return Err(syn::Error::new(receiver_ident.span(), "Only method calls referring to the FSM builder are allowed!"));
    }
       
    Ok(FsmBlockMethodCall {
        expr_call: mc.clone()
    })
}

fn get_method_receiver_ident(expr: &Expr) -> syn::Result<&syn::Ident> {
    let path = match expr {
        Expr::MethodCall(call) => {
            return get_method_receiver_ident(&call.receiver);
        },
        Expr::Path(ep) => Ok(ep),
        _ => {
            Err(syn::Error::new(expr.span(), "Expected a simple method receiver!"))
        }
    }?;

    let segment = match (path.path.segments.len(), path.path.segments.first()) {
        (1, Some(segment)) => Ok(segment),
        (_, _) => Err(syn::Error::new(path.path.segments.span(), "Expected a single segment"))
    }?;

    Ok(&segment.ident)
}