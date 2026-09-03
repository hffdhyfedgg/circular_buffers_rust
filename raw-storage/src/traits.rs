#[cfg(feature = "alloc")]
use crate::error::StorageError;

/// Base contract for immutable continuous memory access.
pub trait Storage {
    /// The type of elements held in storage.
    type Item;

    /// Returns the total capacity of allocated memory (in terms of element count).
    fn capacity(&self) -> usize;

    /// Provides access to a continuous immutable slice of the stored memory.
    fn as_slice(&self) -> &[Self::Item];
}

/// Contract for mutable continuous memory access.
pub trait StorageMut: Storage {
    /// Provides access to a continuous mutable slice of the stored memory.
    fn as_mut_slice(&mut self) -> &mut [Self::Item];
}

/// Contract for dynamically resizable storage.
#[cfg(feature = "alloc")]
pub trait ResizableStorage: StorageMut {
    /// Attempts to resize the storage to the requested new capacity.
    ///
    /// # Errors
    ///
    /// Returns [`StorageError::ResizeFailed`] if the memory allocation or reservation fails.
    fn try_resize(&mut self, new_capacity: usize) -> Result<(), StorageError>;
}
