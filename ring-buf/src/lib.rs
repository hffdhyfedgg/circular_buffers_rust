#![no_std]
#![deny(unsafe_code)]

//! `ring-buf` crate: ring buffers, stacks, filter banks, and mathematical operations.

pub mod cbuf;
pub mod error;
#[cfg(feature = "ffi")]
pub mod ffi;
pub mod filter;
pub mod iter;
pub mod math;
pub mod ops;
pub mod stack;

pub use cbuf::CBuf;
pub use error::{Result, RingBufError};
#[cfg(feature = "ffi")]
pub use ffi::FfiError;
pub use filter::FilterBank;
pub use iter::{CBufIter, CBufIterMut};
pub use stack::CBufStack;
