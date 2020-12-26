use proc_macro2::TokenStream;
use syn::{parse::{self, Parse, ParseStream}, spanned::Spanned};


pub struct FsmFnInput {
    
}

impl FsmFnInput {
    pub fn parse(attr: TokenStream, item: TokenStream) -> syn::Result<Self> {

        let input_fn: syn::ItemFn = syn::parse2(item)?;

        // generics check
        if input_fn.sig.generics.params.len() > 0 {
            return Err(syn::Error::new(input_fn.sig.generics.span(), "Generics aren't supported!"));
        }

        // input type check


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

        
        Ok(FsmFnInput { })
    }
}
