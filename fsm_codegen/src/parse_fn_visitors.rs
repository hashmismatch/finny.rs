extern crate quote;
extern crate syn;

use fsm_def::*;
use graph::*;


use syn::visit::*;

use parse_fn::LetFsmDeclaration;


pub fn extract_method_generic_ty(i: &syn::ExprMethodCall) -> syn::Type {
    if let Some(ref turbofish) = i.turbofish {
        if turbofish.args.len() != 1 { panic!("Expected a single generic argument"); }
        if let syn::GenericMethodArgument::Type(ref ty) = turbofish.args[0] {
            return ty.clone();
        }
    }

    panic!("Turbofish missing?");
}


#[derive(Debug, Clone)]
pub struct FsmMethodCall {
    pub calls: Vec<syn::ExprMethodCall>
}

pub fn find_fsm_method_calls(fn_body: &syn::ItemFn, fsm_decl: &LetFsmDeclaration) -> Vec<FsmMethodCall> {
    #[derive(Debug)]
    struct FindInlineStates<'a> {
        fsm_decl: &'a LetFsmDeclaration,
        calls: Vec<syn::ExprMethodCall>,
        level: usize
    }

    impl<'a, 'ast> syn::visit::Visit<'ast> for FindInlineStates<'a> {
        fn visit_expr_method_call(&mut self, i: &'ast syn::ExprMethodCall) {
            let is_on_fsm = {
                let mut f = IdentFinder {
                    ident: self.fsm_decl.fsm_let_ident.clone(),
                    found: false
                };

                f.visit_expr(&i.receiver);

                f.found
            };

            if is_on_fsm && self.level == 0 {
                //panic!("us: {:#?}", i);
                self.calls.push(i.clone());
            }

            self.level += 1;
            visit_expr_method_call(self, i);
            self.level -= 1;
        }
    }

    let mut ret = vec![];

    let mut finder = FindInlineStates {
        fsm_decl: fsm_decl,
        calls: vec![],
        level: 0
    };
    finder.visit_item_fn(fn_body);

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
        if self.ident == i {
            self.found = true;
        }

        visit_ident(self, i);
    }
}
