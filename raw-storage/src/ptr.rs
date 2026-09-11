use core::marker::PhantomData;
use core::ptr::NonNull;
use crate::traits::{Storage, StorageMut};

/// Storage implementation wrapping a raw pointer.
///
/// Holds a `NonNull<T>`, length, and lifetime marker `PhantomData<&'a mut T>`.
#[derive(Debug)]
pub struct PtrStorage<'a, T> {
    ptr: NonNull<T>,
    len: usize,
    _marker: PhantomData<&'a mut T>,
}

impl<'a, T> PtrStorage<'a, T> {
    /// Creates a new `PtrStorage` from a raw pointer and length.
    ///
    /// # Safety
    ///
    /// - `ptr` must be non-null and valid for reads (and writes if mutated) for `len * size_of::<T>()` bytes.
    /// - `ptr` must be properly aligned for type `T`.
    /// - The memory pointed to by `ptr` must contain `len` initialized values of type `T`.
    /// - The memory must not be accessed concurrently outside this `PtrStorage` for lifetime `'a`.
    /// - If `len == 0`, `ptr` may be dangling or null.
    ///
    /// # Panics
    ///
    /// Этот метод никогда не паникует.
    pub unsafe fn from_raw_parts(ptr: *mut T, len: usize) -> Self {
        let ptr = match NonNull::new(ptr) {
            Some(p) => p,
            None => NonNull::dangling(),
        };
        Self {
            ptr,
            len,
            _marker: PhantomData,
        }
    }
}

impl<'a, T> Storage for PtrStorage<'a, T> {
    type Item = T;

    /// # Panics
    ///
    /// Этот метод никогда не паникует.
    fn len(&self) -> usize {
        self.len
    }

    /// # Panics
    ///
    /// Этот метод никогда не паникует.
    fn capacity(&self) -> usize {
        self.len
    }

    /// Provides access to an immutable slice of stored elements.
    ///
    /// # Panics
    ///
    /// Этот метод никогда не паникует.
    fn as_slice(&self) -> &[Self::Item] {
        if self.len == 0 {
            &[]
        } else {
            #[allow(unsafe_code)]
            // SAFETY: `ptr` is guaranteed to be non-null, aligned, and pointing to `len` valid initialized `T`s by `from_raw_parts` safety invariants.
            unsafe {
                core::slice::from_raw_parts(self.ptr.as_ptr(), self.len)
            }
        }
    }
}

impl<'a, T> StorageMut for PtrStorage<'a, T> {
    /// Provides access to a mutable slice of stored elements.
    ///
    /// # Panics
    ///
    /// Этот метод никогда не паникует.
    fn as_mut_slice(&mut self) -> &mut [Self::Item] {
        if self.len == 0 {
            &mut []
        } else {
            #[allow(unsafe_code)]
            // SAFETY: `ptr` is guaranteed to be non-null, aligned, and pointing to `len` valid initialized `T`s by `from_raw_parts` safety invariants, with exclusive access for lifetime `'a`.
            unsafe {
                core::slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len)
            }
        }
    }
}

/// # Safety
///
/// `PtrStorage` owns access to `T` for lifetime `'a`. Transferring `PtrStorage` across thread boundaries is safe if `T: Send`.
#[allow(unsafe_code)]
unsafe impl<'a, T: Send> Send for PtrStorage<'a, T> {}

/// # Safety
///
/// `PtrStorage` allows access to `T`. Sharing `PtrStorage` across threads is safe if `T: Sync`.
#[allow(unsafe_code)]
unsafe impl<'a, T: Sync> Sync for PtrStorage<'a, T> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ptr_storage_read_write() {
        let mut data = [10, 20, 30, 40];
        let mut storage = unsafe { PtrStorage::from_raw_parts(data.as_mut_ptr(), data.len()) };

        assert_eq!(storage.len(), 4);
        assert_eq!(storage.capacity(), 4);
        assert_eq!(storage.as_slice(), &[10, 20, 30, 40]);

        storage.as_mut_slice()[1] = 99;
        assert_eq!(storage.as_slice(), &[10, 99, 30, 40]);
        assert_eq!(data[1], 99);
    }

    #[test]
    fn test_ptr_storage_zero_len() {
        let mut storage = unsafe { PtrStorage::<i32>::from_raw_parts(core::ptr::null_mut(), 0) };

        assert_eq!(storage.len(), 0);
        assert_eq!(storage.capacity(), 0);
        assert_eq!(storage.as_slice(), &[]);
        assert_eq!(storage.as_mut_slice(), &mut []);
    }
}
