#![no_std]
#![deny(unsafe_code)]

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod error;
pub mod impls;
pub mod traits;

pub use error::{Result, StorageError};
pub use impls::{ArrayStorage, SliceStorage};
#[cfg(feature = "alloc")]
pub use impls::AllocStorage;
pub use traits::{Storage, StorageMut};
#[cfg(feature = "alloc")]
pub use traits::ResizableStorage;
