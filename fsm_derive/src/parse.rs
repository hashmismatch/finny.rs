use std::collections::{HashMap, HashSet};

use proc_macro2::TokenStream;
use syn::{Error, Expr, ExprMethodCall, GenericArgument, ItemFn, parse::{self, Parse, ParseStream}, spanned::Spanned};

use crate::parse_blocks::{FsmBlock, decode_blocks, get_generics, get_method_receiver_ident};


pub struct FsmFnInput {
    pub base: FsmFnBase,
    pub decl: FsmDeclarations
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

        let blocks = decode_blocks(&base, &input_fn)?;



        let fsm_declarations = FsmDeclarations::parse(&base, &input_fn, &blocks)?;

        panic!("declarations: {:#?}", fsm_declarations);
        
        Ok(FsmFnInput {
            base,
            decl: fsm_declarations
        })
    }
}

#[derive(Debug)]
pub struct FsmDeclarations {
    pub initial_state: syn::Type,
    pub states: HashMap<syn::Type, FsmState>,
    pub events: HashMap<syn::Type, FsmEvent>
}

#[derive(Debug)]
pub struct FsmState {
    pub ty: syn::Type
}
#[derive(Debug)]
pub struct FsmEvent {
    pub ty: syn::Type
}

impl FsmDeclarations {
    pub fn parse(base: &FsmFnBase, input_fn: &ItemFn, blocks: &Vec<FsmBlock>) -> syn::Result<Self> {
        let mut initial_state = None;
        let mut states = HashMap::new();
        let mut events = HashMap::new();
        

        for block in blocks {
            match block {
                FsmBlock::MethodCall(mc) => {

                    let methods = mc.method_calls
                        .iter()
                        .map(|m| MethodOverview::parse(m))
                        .collect::<syn::Result<Vec<_>>>()?;

                    let methods: Vec<_> = methods.iter().map(|m| m.as_ref()).collect();

                    match methods.as_slice() {
                        [MethodOverviewRef { name: "build", .. } ] => {
                            
                        },
                        [MethodOverviewRef { name: "initial_state", generics: [ty], .. }] => {
                            initial_state = Some(ty.clone());
                        },
                        [MethodOverviewRef { name: "state", generics: [ty_state] }, st @ .. ] => {

                            let mut state = states
                                .entry(ty_state.clone())
                                .or_insert(FsmState { ty: ty_state.clone() });
                            
                        },
                        [MethodOverviewRef { name: "on_event", generics: [ty_event] }, ev @ .. ] => {

                            let mut event = events
                                .entry(ty_event.clone())
                                .or_insert(FsmEvent { ty: ty_event.clone() });
                                                        
                        }
                        _ => { return Err(syn::Error::new(mc.expr_call.span(), "Unsupported method.")); }
                    }

                },
                _ => todo!("unsupported block!")
            }
        }
        
        let dec = FsmDeclarations {
            initial_state: initial_state.ok_or(syn::Error::new(input_fn.span(), "Missing the initial state declaration!"))?,
            states,
            events
        };
        Ok(dec)
    }
}

struct MethodOverview {
    name: String,
    generics: Vec<syn::Type>
}

impl MethodOverview {
    pub fn parse(m: &ExprMethodCall) -> syn::Result<Self> {
        let generics = get_generics(&m.turbofish)?;

        Ok(Self {
            name: m.method.to_string(),
            generics
        })
    }

    pub fn as_ref(&self) -> MethodOverviewRef {
        MethodOverviewRef {
            name: self.name.as_str(),
            generics: self.generics.as_slice()
        }
    }
}

struct MethodOverviewRef<'a> {
    name: &'a str,
    generics: &'a [syn::Type]
}