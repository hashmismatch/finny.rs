#![cfg_attr(not(feature = "std"), no_std)]


#![cfg_attr(feature="core_collections", feature(alloc))]
#![cfg_attr(feature="core_collections", feature(collections))]


#[cfg(any(feature="core_collections"))]
#[macro_use]
extern crate alloc;

#[cfg(any(feature="core_collections"))]
#[macro_use]
extern crate collections;  

extern crate serde;

#[cfg(feature="info_serializable")]
#[macro_use]
extern crate serde_derive;

#[cfg(feature="info_serializable")]
#[macro_use]
extern crate serde_json;


mod prelude;

mod machine;

pub use machine::*;

#[macro_use]
mod macros;

pub mod console_inspect;
pub mod inspect_combinator;
pub mod inspect_data;