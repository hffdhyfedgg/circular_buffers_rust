#[cfg(feature = "alloc")]
use alloc::vec::Vec;

#[cfg(feature = "alloc")]
use crate::error::StorageError;
use crate::traits::{Storage, StorageMut};
#[cfg(feature = "alloc")]
use crate::traits::ResizableStorage;

/// Storage backed by a fixed-size static array.
///
/// Designed for `no_std` / embedded systems without dynamic allocation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ArrayStorage<T, const N: usize> {
    data: [T; N],
}

impl<T, const N: usize> ArrayStorage<T, N> {
    /// Creates a new [`ArrayStorage`] owning the provided fixed-size array.
    #[inline]
    pub const fn new(data: [T; N]) -> Self {
        Self { data }
    }
}

impl<T: Clone, const N: usize> ArrayStorage<T, N> {
    /// Creates a new [`ArrayStorage`] initialized with copies of `value`.
    pub fn from_element(value: T) -> Self {
        Self {
            data: core::array::from_fn(|_| value.clone()),
        }
    }
}

impl<T: Default, const N: usize> Default for ArrayStorage<T, N>
where
    [T; N]: Default,
{
    fn default() -> Self {
        Self {
            data: Default::default(),
        }
    }
}

impl<T, const N: usize> From<[T; N]> for ArrayStorage<T, N> {
    fn from(data: [T; N]) -> Self {
        Self::new(data)
    }
}

impl<T, const N: usize> Storage for ArrayStorage<T, N> {
    type Item = T;

    #[inline]
    fn capacity(&self) -> usize {
        N
    }

    #[inline]
    fn as_slice(&self) -> &[Self::Item] {
        &self.data
    }
}

impl<T, const N: usize> StorageMut for ArrayStorage<T, N> {
    #[inline]
    fn as_mut_slice(&mut self) -> &mut [Self::Item] {
        &mut self.data
    }
}

/// Storage borrowing an external mutable slice.
///
/// Allows operating on user-provided memory without copying or allocating.
#[derive(Debug, PartialEq, Eq)]
pub struct SliceStorage<'a, T> {
    slice: &'a mut [T],
}

impl<'a, T> SliceStorage<'a, T> {
    /// Creates a new [`SliceStorage`] borrowing the provided mutable slice.
    #[inline]
    pub fn new(slice: &'a mut [T]) -> Self {
        Self { slice }
    }
}

impl<'a, T> From<&'a mut [T]> for SliceStorage<'a, T> {
    fn from(slice: &'a mut [T]) -> Self {
        Self::new(slice)
    }
}

impl<'a, T> Storage for SliceStorage<'a, T> {
    type Item = T;

    #[inline]
    fn capacity(&self) -> usize {
        self.slice.len()
    }

    #[inline]
    fn as_slice(&self) -> &[Self::Item] {
        self.slice
    }
}

impl<'a, T> StorageMut for SliceStorage<'a, T> {
    #[inline]
    fn as_mut_slice(&mut self) -> &mut [Self::Item] {
        self.slice
    }
}

/// Storage backed by a dynamically allocated [`Vec<T>`].
///
/// Available only when the `alloc` feature flag is enabled.
#[cfg(feature = "alloc")]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AllocStorage<T> {
    buf: Vec<T>,
}

#[cfg(feature = "alloc")]
impl<T> AllocStorage<T> {
    /// Creates a new empty [`AllocStorage`].
    #[inline]
    pub const fn new() -> Self {
        Self { buf: Vec::new() }
    }

    /// Creates a new [`AllocStorage`] from an existing [`Vec<T>`].
    #[inline]
    pub fn from_vec(buf: Vec<T>) -> Self {
        Self { buf }
    }
}

#[cfg(feature = "alloc")]
impl<T: Default> AllocStorage<T> {
    /// Creates a new [`AllocStorage`] pre-populated with `capacity` default elements.
    pub fn with_capacity(capacity: usize) -> Self {
        let mut buf = Vec::new();
        buf.resize_with(capacity, T::default);
        Self { buf }
    }
}

#[cfg(feature = "alloc")]
impl<T: Clone> AllocStorage<T> {
    /// Creates a new [`AllocStorage`] pre-populated with `capacity` copies of `value`.
    pub fn from_element(value: T, capacity: usize) -> Self {
        Self {
            buf: alloc::vec![value; capacity],
        }
    }
}

#[cfg(feature = "alloc")]
impl<T> Default for AllocStorage<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "alloc")]
impl<T> From<Vec<T>> for AllocStorage<T> {
    fn from(buf: Vec<T>) -> Self {
        Self::from_vec(buf)
    }
}

#[cfg(feature = "alloc")]
impl<T> Storage for AllocStorage<T> {
    type Item = T;

    #[inline]
    fn capacity(&self) -> usize {
        self.buf.len()
    }

    #[inline]
    fn as_slice(&self) -> &[Self::Item] {
        &self.buf
    }
}

#[cfg(feature = "alloc")]
impl<T> StorageMut for AllocStorage<T> {
    #[inline]
    fn as_mut_slice(&mut self) -> &mut [Self::Item] {
        &mut self.buf
    }
}

#[cfg(feature = "alloc")]
impl<T: Default> ResizableStorage for AllocStorage<T> {
    fn try_resize(&mut self, new_capacity: usize) -> Result<(), StorageError> {
        let current = self.buf.len();
        if new_capacity == current {
            return Ok(());
        }

        if new_capacity > current {
            let additional = new_capacity - current;
            self.buf.try_reserve(additional).map_err(|_| {
                #[cfg(feature = "verbose-errors")]
                {
                    StorageError::ResizeFailed {
                        requested: new_capacity,
                        current,
                    }
                }
                #[cfg(not(feature = "verbose-errors"))]
                {
                    StorageError::ResizeFailed
                }
            })?;
            self.buf.resize_with(new_capacity, T::default);
        } else {
            self.buf.truncate(new_capacity);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_array_storage_init_and_access() {
        let storage = ArrayStorage::new([1, 2, 3, 4]);
        assert_eq!(storage.capacity(), 4);
        assert_eq!(storage.as_slice(), &[1, 2, 3, 4]);
    }

    #[test]
    fn test_array_storage_from_element_and_default() {
        let storage = ArrayStorage::<i32, 5>::from_element(42);
        assert_eq!(storage.capacity(), 5);
        assert_eq!(storage.as_slice(), &[42, 42, 42, 42, 42]);

        let default_storage = ArrayStorage::<i32, 3>::default();
        assert_eq!(default_storage.capacity(), 3);
        assert_eq!(default_storage.as_slice(), &[0, 0, 0]);
    }

    #[test]
    fn test_array_storage_mutation() {
        let mut storage = ArrayStorage::new([10, 20, 30]);
        let slice = storage.as_mut_slice();
        slice[1] = 99;
        assert_eq!(storage.as_slice(), &[10, 99, 30]);
    }

    #[test]
    fn test_slice_storage_init_and_mutation() {
        let mut buf = [1, 2, 3, 4, 5];
        {
            let mut storage = SliceStorage::new(&mut buf);
            assert_eq!(storage.capacity(), 5);
            assert_eq!(storage.as_slice(), &[1, 2, 3, 4, 5]);

            storage.as_mut_slice()[2] = 42;
        }
        assert_eq!(buf, [1, 2, 42, 4, 5]);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn test_alloc_storage_operations() {
        let mut storage = AllocStorage::with_capacity(3);
        assert_eq!(storage.capacity(), 3);
        assert_eq!(storage.as_slice(), &[0, 0, 0]);

        storage.as_mut_slice()[0] = 10;
        storage.as_mut_slice()[1] = 20;
        storage.as_mut_slice()[2] = 30;
        assert_eq!(storage.as_slice(), &[10, 20, 30]);

        // Grow storage
        assert!(storage.try_resize(5).is_ok());
        assert_eq!(storage.capacity(), 5);
        assert_eq!(storage.as_slice(), &[10, 20, 30, 0, 0]);

        // Shrink storage
        assert!(storage.try_resize(2).is_ok());
        assert_eq!(storage.capacity(), 2);
        assert_eq!(storage.as_slice(), &[10, 20]);

        // Resize to same capacity
        assert!(storage.try_resize(2).is_ok());
        assert_eq!(storage.capacity(), 2);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn test_alloc_storage_from_vec_and_element() {
        let vec = alloc::vec![1, 2, 3];
        let storage = AllocStorage::from_vec(vec);
        assert_eq!(storage.capacity(), 3);
        assert_eq!(storage.as_slice(), &[1, 2, 3]);

        let elem_storage = AllocStorage::from_element(7, 4);
        assert_eq!(elem_storage.capacity(), 4);
        assert_eq!(elem_storage.as_slice(), &[7, 7, 7, 7]);
    }
}
