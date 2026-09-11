#![no_std]
#![warn(unsafe_code)]
// Note: `unsafe_code` is warned crate-wide because `unsafe` is strictly restricted to
// `ptr.rs` with mandatory `/// # Safety` documentation.

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod error;
pub mod impls;
pub mod ptr;
pub mod traits;

pub use error::{Result, StorageError};
pub use impls::{ArrayStorage, SliceStorage};
#[cfg(feature = "alloc")]
pub use impls::AllocStorage;
pub use ptr::PtrStorage;
pub use traits::{Storage, StorageMut, VolatileStorage};
#[cfg(feature = "alloc")]
pub use traits::ResizableStorage;
