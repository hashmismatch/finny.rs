use syn::{Expr, ExprMethodCall, MethodTurbofish, spanned::Spanned};

use crate::parse::FsmFnBase;


#[derive(Debug)]
pub enum FsmBlock {
    MethodCall(FsmBlockMethodCall),
    Struct()
}
#[derive(Debug)]
pub struct FsmBlockStruct {

}

pub fn decode_blocks(base: &FsmFnBase, item_fn: &syn::ItemFn) -> syn::Result<Vec<FsmBlock>> {
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
    pub expr_call: ExprMethodCall,
    pub method_calls: Vec<ExprMethodCall>
}

pub fn decode_method_call(base: &FsmFnBase, expr: &Expr) -> syn::Result<FsmBlockMethodCall> {
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
        expr_call: mc.clone(),
        method_calls: flatten_method_calls(&mc)?
    })
}

pub fn get_method_receiver_ident(expr: &Expr) -> syn::Result<&syn::Ident> {
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

pub fn flatten_method_calls(mc: &ExprMethodCall) -> syn::Result<Vec<ExprMethodCall>> {
    let mut ret = vec![];
    ret.push(mc.clone());

    let mut t = &mc.receiver;
    loop {
        match **t {
            Expr::MethodCall(ref ex) => {
                ret.push(ex.clone());
                t = &ex.receiver;
            }
            Expr::Path(_) => { break; }
            _ => { return Err(syn::Error::new(mc.receiver.span(), "Unsupported.")); }
        }
    }

    ret.reverse();
    Ok(ret)
}

pub fn get_generics(turbofish: &Option<MethodTurbofish>) -> syn::Result<Vec<syn::Type>> {
    let mut ret = vec![];

    if let Some(turbofish) = turbofish {
        for arg in &turbofish.args {
            match arg {
                syn::GenericMethodArgument::Type(ty) => { ret.push(ty.clone()); },
                _ => { return Err(syn::Error::new(arg.span(), "Unsupported.")); }
            }
        }
    }

    Ok(ret)
}