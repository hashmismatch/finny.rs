extern crate syn;

#[cfg(feature="std")]
pub fn core_fmt_debug() -> syn::Ty {
    syn::parse_type("std::fmt::Debug").unwrap()
}

#[cfg(not(feature="std"))]
pub fn core_fmt_debug() -> syn::Ty {
    syn::parse_type("core::fmt::Debug").unwrap()
}

