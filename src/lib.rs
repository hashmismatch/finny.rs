#![cfg_attr(not(feature = "std"), no_std)]


#![cfg_attr(feature="core_collections", feature(alloc))]
#![cfg_attr(feature="core_collections", feature(collections))]


#[cfg(any(feature="core_collections"))]
#[macro_use]
extern crate alloc;

#[cfg(any(feature="core_collections"))]
#[macro_use]
extern crate collections;  

mod prelude;

mod machine;

pub use machine::*;

#[macro_use]
mod macros;
