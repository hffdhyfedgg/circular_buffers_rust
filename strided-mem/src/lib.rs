#![no_std]

pub mod error;
pub mod indexed;
pub mod invert_view;
pub mod iter;
pub(crate) mod pointer;
pub mod ring_view;
pub mod split;
pub mod tail_view;
pub mod traits;
pub mod views;

pub use error::{Result, StridedError};
pub use invert_view::{InvertView, InvertViewMut};
pub use ring_view::{Ring2NView, Ring2NViewMut, RingView, RingViewMut};
pub use tail_view::{TailView, TailViewMut};
pub use traits::{ContiguousView, ContiguousViewMut, View, ViewMut};
pub use indexed::{split_indexed_mut, IndexedViewMut};
pub use iter::{StridedIter, StridedIterMut};
pub use split::{StridedSplitIter, StridedSplitIterMut};
pub use views::{StridedView, StridedViewMut};
