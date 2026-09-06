#[cfg(feature = "alloc")]
use crate::error::StorageError;

/// Base contract for immutable continuous memory access.
pub trait Storage {
    /// The type of elements held in storage.
    type Item;

    /// Returns the number of initialized elements currently stored.
    fn len(&self) -> usize;

    /// Returns `true` if the storage contains no elements.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the total capacity of allocated memory (in terms of element count).
    fn capacity(&self) -> usize;

    /// Provides access to a continuous immutable slice of the stored memory.
    fn as_slice(&self) -> &[Self::Item];
}

/// Unsafe contract for direct memory-mapped I/O (MMIO) hardware register access.
///
/// # Safety
///
/// Creating an intermediate `&mut T` reference to MMIO registers can result in
/// Undefined Behavior (UB) due to compiler optimizations or aliasing rules.
/// Implementations of this trait perform volatile reads and writes directly
/// using raw pointers without constructing reference types.
#[allow(unsafe_code)]
pub unsafe trait VolatileStorage {
    /// The type of elements held in storage.
    type Item;

    /// Returns the number of registers or elements mapped by this storage.
    fn len(&self) -> usize;

    /// Performs a volatile read from the memory-mapped storage at `index`.
    ///
    /// # Safety
    ///
    /// `index` must be strictly less than `self.len()`, and the underlying
    /// memory must be valid for volatile reads.
    unsafe fn read_volatile(&self, index: usize) -> Self::Item;

    /// Performs a volatile write to the memory-mapped storage at `index`.
    ///
    /// # Safety
    ///
    /// `index` must be strictly less than `self.len()`, and the underlying
    /// memory must be valid for volatile writes.
    unsafe fn write_volatile(&mut self, index: usize, value: Self::Item);
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
