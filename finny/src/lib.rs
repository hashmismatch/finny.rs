#![cfg_attr(not(feature = "std"), no_std)]

//! # Finny - Finite State Machines for Rust
//!
//! ## Features
//! * Declarative, builder API with procedural macros that generate the necessary boilerplate
//! * Compile-time transition graph validation
//! * No allocations required
//! * Support for generics within the shared context
//! * Transition guards and actions
//! * `no_std` support
//!
//! ## Example
//!
//! ```rust
//! extern crate finny;
//! use finny::{finny_fsm, FsmFactory, FsmResult, decl::{BuiltFsm, FsmBuilder}};
//! 
//! #[derive(Default)]
//! pub struct MyContext { val: u32 }
//! #[derive(Default)] 
//! pub struct MyStateA { n: usize }
//! #[derive(Default)]
//! pub struct MyStateB;
//! pub struct MyEvent;
//!
//! #[finny_fsm]
//! fn my_fsm(mut fsm: FsmBuilder<MyFsm, MyContext>) -> BuiltFsm {
//!     fsm.state::<MyStateA>()
//!        .on_entry(|state, ctx| {
//!            state.n += 1;
//!            ctx.context.val += 1;
//!         });
//!     fsm.state::<MyStateB>();
//!     fsm.on_event::<MyEvent>()
//!        .transition_from::<MyStateA>().to::<MyStateB>()
//!        .guard(|_ev, ctx| { ctx.context.val > 0 })
//!        .action(|_ev, ctx, state_a, state_b| { ctx.context.val += 1; });
//!     fsm.initial_state::<MyStateA>();
//!     fsm.build()
//! }
//! 
//! fn main() -> FsmResult<()> {
//!     let mut fsm = MyFsm::new(MyContext::default())?;
//!     assert_eq!(0, fsm.val);
//!     fsm.start()?;
//!     let state_a: &MyStateA = fsm.get_state();
//!     assert_eq!(1, state_a.n);
//!     assert_eq!(1, fsm.val);
//!     fsm.dispatch(MyEvent)?;
//!     assert_eq!(2, fsm.val);
//!     Ok(())
//! }
//! ```

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

mod lib {
    mod core {
        #[cfg(not(feature = "std"))]
        pub use core::*;
        #[cfg(feature = "std")]
        pub use std::*;
   }

   pub use self::core::marker::{self, PhantomData};
   pub use self::core::ops::{Deref, DerefMut};
   pub use self::core::fmt::Debug;
   pub use self::core::result::Result;

   #[cfg(feature="std")]
   pub use std::collections::VecDeque;
}