use crate::parse::FsmFnBase;
use syn::visit::{Visit, visit_ident, visit_item_struct, visit_expr_method_call};


pub fn extract_method_generic_ty(i: &syn::ExprMethodCall) -> syn::Type {
    let mut gens = extract_method_generic_ty_all(i);
    if gens.len() != 1 {
        panic!("Expected a single generic argument");
    }
    gens.pop().expect("Missing argument?")
}

pub fn extract_method_generic_ty_all(i: &syn::ExprMethodCall) -> Vec<syn::Type> {
    if let Some(ref turbofish) = i.turbofish {
        let mut ret = vec![];
        for arg in &turbofish.args {
            if let syn::GenericMethodArgument::Type(ref ty) = *arg {
                ret.push(ty.clone())
            }
        }
        return ret;
    }

    panic!("Turbofish missing?");
}


#[derive(Debug, Clone)]
pub struct FsmMethodCall {
    pub calls: Vec<syn::ExprMethodCall>
}

pub fn find_fsm_method_calls(fn_body: &syn::ItemFn, fsm_decl_base: &FsmFnBase) -> Vec<FsmMethodCall> {
    #[derive(Debug)]
    struct FindInlineStates<'a> {
        fsm_decl_base: &'a FsmFnBase,
        calls: Vec<syn::ExprMethodCall>,
        level: usize,
        errors: Vec<syn::Error>
    }

    impl<'a, 'ast> syn::visit::Visit<'ast> for FindInlineStates<'a> {
        fn visit_expr_method_call(&mut self, i: &'ast syn::ExprMethodCall) {
            let is_on_fsm = {
                let mut f = IdentFinder {
                    ident: self.fsm_decl_base.builder_ident.clone(),
                    found: false
                };

                f.visit_expr(&i.receiver);

                f.found
            };
            
            if is_on_fsm && self.level == 0 {
                self.calls.push(i.clone());
            }

            self.level += 1;
            visit_expr_method_call(self, i);
            self.level -= 1;
        }

        fn visit_expr_closure(&mut self, i: &'ast syn::ExprClosure) {
            // do not descend down closures
        }
    }

    let mut ret = vec![];

    let mut finder = FindInlineStates {
        fsm_decl_base,
        calls: vec![],
        level: 0,
        errors: vec![]
    };
    finder.visit_item_fn(fn_body);

    if finder.errors.len() > 0 {
        
    }

    #[derive(Default)]
    struct MethodVisitor {
        statement: usize,
        calls: Vec<syn::ExprMethodCall>
    }
    impl<'ast> syn::visit::Visit<'ast> for MethodVisitor {
        fn visit_expr_method_call(&mut self, i: &'ast syn::ExprMethodCall) {
            self.calls.push(i.clone());            
            visit_expr_method_call(self, i);
        }

        fn visit_expr_closure(&mut self, i: &'ast syn::ExprClosure) {
            // do not descend down closures
        }
    }    

    for (idx, call) in finder.calls.iter().enumerate() {
        let mut method_visitor: MethodVisitor = Default::default();
        method_visitor.visit_expr_method_call(call);

        let mut calls = method_visitor.calls.clone();
        calls.reverse();
        ret.push(FsmMethodCall {
            calls: calls
        });
    }

    ret
}

struct IdentFinder {
    ident: syn::Ident,
    found: bool
}

impl<'ast> syn::visit::Visit<'ast> for IdentFinder {
    fn visit_ident(&mut self, i: &'ast syn::Ident) {
        if self.ident.eq(i) {
            self.found = true;
        }

        visit_ident(self, i);
    }
}

#[derive(Default)]
struct StructFinder {
    structs: Vec<syn::ItemStruct>
}

impl<'ast> syn::visit::Visit<'ast> for StructFinder {
    fn visit_item_struct(&mut self, i: &'ast syn::ItemStruct) {
        self.structs.push(i.clone());
        visit_item_struct(self, i);
    }
}


pub fn find_inline_structs(fn_body: &syn::ItemFn) -> Vec<syn::ItemStruct> {
    let mut finder = StructFinder::default();
    finder.visit_item_fn(fn_body);
    finder.structs
}

pub fn get_base_name(ty: &syn::Type) -> syn::Ident {
    #[derive(Default)]
    struct FirstIdent {
        ident: Option<syn::Ident>
    }

    impl<'ast> syn::visit::Visit<'ast> for FirstIdent {
        fn visit_ident(&mut self, i: &'ast syn::Ident) {
            if self.ident.is_none() {
                self.ident = Some(i.clone());
            }
            visit_ident(self, i);
        }
    }

    let mut f = FirstIdent::default();
    f.visit_type(ty);
    f.ident.expect("No ident found")
}