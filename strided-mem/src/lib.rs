#![no_std]

pub mod error;
pub mod indexed;
pub mod iter;
pub(crate) mod pointer;
pub mod split;
pub mod views;

pub use error::{Result, StridedError};
pub use indexed::{split_indexed_mut, IndexedViewMut};
pub use iter::{StridedIter, StridedIterMut};
pub use split::{StridedSplitIter, StridedSplitIterMut};
pub use views::{StridedView, StridedViewMut};
