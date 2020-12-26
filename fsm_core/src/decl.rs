//! Traits to be used with the derive function macro

use std::marker::PhantomData;

pub struct FsmBuilder<TFsm, TContext> {
    _fsm: PhantomData<TFsm>,
    _context: PhantomData<TContext>
}

pub struct BuiltFsm;

impl<TFsm, TContext> FsmBuilder<TFsm, TContext> {
    pub fn build(self) -> BuiltFsm {
        BuiltFsm
    }
}

/*
impl FsmDecl {
	pub fn new_fsm<F>() -> FsmDecl2<F, ()> where F: Fsm {
		FsmDecl2 {
			fsm_ty: PhantomData::default(),
			fsm_ctx_ty: PhantomData::default()
		}
	}
}
*/