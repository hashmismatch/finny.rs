
pub use core::marker::PhantomData;
pub use core::iter;
pub use core::cell::RefCell;
pub use core::fmt;
pub use core::fmt::Debug;
pub use core::fmt::Write as FmtWrite;
pub use core::fmt::Error as FmtError;
pub use core::ops::Range;
pub use core::num::Wrapping;
pub use core::cmp::*;
pub use core::mem;
pub use core::intrinsics::write_bytes;
pub use core::ops::Deref;
pub use core::slice;


#[cfg(feature="core_collections")]
pub use alloc::rc::Rc;
#[cfg(feature="core_collections")]
pub use alloc::arc::Arc;
#[cfg(feature="core_collections")]
pub use alloc::boxed::Box;

#[cfg(feature="core_collections")]
pub use collections::vec::Vec;
#[cfg(feature="core_collections")]
pub use collections::string::*;
#[cfg(feature="core_collections")]
pub use collections::fmt::format as format_to_string;
#[cfg(feature="core_collections")]
pub use collections::fmt::{Display, Formatter};
#[cfg(feature="core_collections")]
pub use collections::borrow::Cow;
#[cfg(feature="core_collections")]
pub use collections::str::{from_utf8, FromStr};


/*
use core::intrinsics;

pub trait LocalFloat: Sized {
    fn ceil(self) -> Self;
    fn round(self) -> Self;
    fn floor(self) -> Self;
}

impl LocalFloat for f32 {
	#[inline]
    fn ceil(self) -> f32 {
        unsafe { intrinsics::ceilf32(self) }
    }

	#[inline]
    fn round(self) -> f32 {
        unsafe { intrinsics::roundf32(self) }
    }

    #[inline]
    fn floor(self) -> f32 {
        unsafe { intrinsics::floorf32(self) }
    }
}
*/