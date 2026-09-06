#![no_std]

pub mod error;
pub mod pointer;
pub mod split;
pub mod views;

pub use error::{Result, StridedError};
pub use pointer::offset_ptr;
pub use views::{StridedView, StridedViewMut};
