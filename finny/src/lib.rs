//! Finny
//!
//! Finite State Machines for Rust

pub mod decl;
mod fsm;

pub use fsm::*;

extern crate finny_derive;
extern crate derive_more;

pub use finny_derive::finny_fsm;

/// External bundled libraries to be used by the procedural macros.
pub mod bundled {
    /// Derive_more crate for deriving the enum conversions.
    pub mod derive_more {
        pub use crate::derive_more::From;
    }
}