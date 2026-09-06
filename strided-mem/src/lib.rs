#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod error;
pub mod indexed;
pub mod iter;
pub mod pointer;
pub mod split;
pub mod views;

pub use error::{Result, StridedError};
pub use indexed::{split_indexed_mut, IndexedViewMut};
pub use iter::{StridedIter, StridedIterMut};
pub use split::{SplitStrided, StridedSplitIter};
pub use views::{StridedView, StridedViewMut};
