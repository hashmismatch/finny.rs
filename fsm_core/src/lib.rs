

pub mod decl;
mod fsm;

pub use fsm::*;

mod fsm_codegen_mocks;

extern crate derive_more;

pub mod bundled {
    pub mod derive_more {
        pub use crate::derive_more::TryInto;
    }
}