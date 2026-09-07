use core::marker::PhantomData;
use raw_storage::traits::StorageMut;
use crate::error::{Result, RingBufError};
use crate::iter::{CBufIter, CBufIterMut};
use crate::view2n::{CBuf2NView, CBuf2NViewMut};

/// Owning single ring buffer with power-of-two capacity.
#[derive(Debug)]
pub struct CBuf2N<T, S> {
    storage: S,
    head: usize,
    len: usize,
    mask: usize,
    capacity: usize,
    _marker: PhantomData<T>,
}

impl<T, S: StorageMut<Item = T>> CBuf2N<T, S> {
    /// Creates a new `CBuf2N` backed by `storage`.
    ///
    /// # Errors
    /// Returns [`RingBufError::CapacityZero`] if storage capacity is 0, or
    /// [`RingBufError::NotPowerOfTwo`] if storage capacity is not a power of two.
    pub fn try_new(mut storage: S) -> Result<Self> {
        let capacity = storage.as_mut_slice().len();
        if capacity == 0 {
            return Err(RingBufError::CapacityZero);
        }
        if !capacity.is_power_of_two() {
            #[cfg(feature = "verbose-errors")]
            return Err(RingBufError::NotPowerOfTwo { capacity });
            #[cfg(not(feature = "verbose-errors"))]
            return Err(RingBufError::NotPowerOfTwo);
        }
        Ok(Self {
            storage,
            head: capacity - 1,
            len: 0,
            mask: capacity - 1,
            capacity,
            _marker: PhantomData,
        })
    }

    /// Pushes an item into the ring buffer, overwriting the oldest element if full.
    #[inline(always)]
    pub fn push(&mut self, item: T) {
        self.head = (self.head + 1) & self.mask;
        self.storage.as_mut_slice()[self.head] = item;
        if self.len < self.capacity {
            self.len += 1;
        }
    }

    /// Clears the ring buffer state (sets length to 0).
    #[inline(always)]
    pub fn clear(&mut self) {
        self.len = 0;
        self.head = self.capacity - 1;
    }

    /// Returns the number of elements currently stored.
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if the ring buffer contains no elements.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the total capacity of the ring buffer.
    #[inline(always)]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Returns the current head index.
    #[inline(always)]
    pub fn head(&self) -> usize {
        self.head
    }

    /// Returns a reference to the element at relative index `rel` (0 = newest).
    #[inline(always)]
    pub fn get(&self, rel: usize) -> Option<&T> {
        if rel >= self.len {
            None
        } else {
            let phys = (self.head + self.capacity - rel) & self.mask;
            Some(&self.storage.as_slice()[phys])
        }
    }

    /// Returns a mutable reference to the element at relative index `rel` (0 = newest).
    #[inline(always)]
    pub fn get_mut(&mut self, rel: usize) -> Option<&mut T> {
        if rel >= self.len {
            None
        } else {
            let phys = (self.head + self.capacity - rel) & self.mask;
            Some(&mut self.storage.as_mut_slice()[phys])
        }
    }

    /// Returns the buffer data in logical order (oldest -> newest) as up to two continuous slices.
    #[inline(always)]
    pub fn as_slices(&self) -> (&[T], &[T]) {
        if self.len == 0 {
            return (&[], &[]);
        }
        let oldest = (self.head + self.capacity + 1 - self.len) & self.mask;
        let data = self.storage.as_slice();
        if oldest + self.len <= self.capacity {
            (&data[oldest..oldest + self.len], &[])
        } else {
            let first_len = self.capacity - oldest;
            let second_len = self.len - first_len;
            (&data[oldest..self.capacity], &data[..second_len])
        }
    }

    /// Returns the buffer data in logical order (oldest -> newest) as up to two continuous mutable slices.
    #[inline(always)]
    pub fn as_slices_mut(&mut self) -> (&mut [T], &mut [T]) {
        if self.len == 0 {
            return (&mut [], &mut []);
        }
        let oldest = (self.head + self.capacity + 1 - self.len) & self.mask;
        let len = self.len;
        let capacity = self.capacity;
        let data = self.storage.as_mut_slice();
        if oldest + len <= capacity {
            let slice = &mut data[oldest..oldest + len];
            (slice, &mut [])
        } else {
            let first_len = capacity - oldest;
            let second_len = len - first_len;
            let (left, right) = data.split_at_mut(oldest);
            let (slice1, _) = right.split_at_mut(first_len);
            let (slice2, _) = left.split_at_mut(second_len);
            (slice1, slice2)
        }
    }

    /// Returns an immutable view over the ring buffer.
    #[inline(always)]
    pub fn as_view(&self) -> CBuf2NView<'_, T> {
        CBuf2NView::try_from_raw(self.storage.as_slice(), self.head, self.len).unwrap()
    }

    /// Returns a mutable view over the ring buffer.
    #[inline(always)]
    pub fn as_view_mut(&mut self) -> CBuf2NViewMut<'_, T> {
        CBuf2NViewMut::try_from_raw(self.storage.as_mut_slice(), self.head, self.len).unwrap()
    }

    /// Returns an iterator over immutable references in logical order (oldest -> newest).
    #[inline(always)]
    pub fn iter(&self) -> CBufIter<'_, T> {
        let (s1, s2) = self.as_slices();
        CBufIter::new(s1, s2)
    }

    /// Returns an iterator over mutable references in logical order (oldest -> newest).
    #[inline(always)]
    pub fn iter_mut(&mut self) -> CBufIterMut<'_, T> {
        let (s1, s2) = self.as_slices_mut();
        CBufIterMut::new(s1, s2)
    }

    /// Consumes `self` and returns the underlying storage `S`.
    pub fn into_storage(self) -> S {
        self.storage
    }
}

impl<'a, T, S: StorageMut<Item = T>> IntoIterator for &'a CBuf2N<T, S> {
    type Item = &'a T;
    type IntoIter = CBufIter<'a, T>;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T, S: StorageMut<Item = T>> IntoIterator for &'a mut CBuf2N<T, S> {
    type Item = &'a mut T;
    type IntoIter = CBufIterMut<'a, T>;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use raw_storage::impls::ArrayStorage;

    #[test]
    fn test_cbuf2n_owning() {
        let storage = ArrayStorage::<i32, 4>::default();
        let mut buf = CBuf2N::try_new(storage).unwrap();

        assert_eq!(buf.capacity(), 4);
        assert_eq!(buf.len(), 0);

        buf.push(1);
        buf.push(2);
        buf.push(3);
        assert_eq!(buf.len(), 3);
        assert_eq!(buf.get(0), Some(&3));
        assert_eq!(buf.get(2), Some(&1));

        buf.push(4);
        buf.push(5); // overwrites 1
        assert_eq!(buf.len(), 4);
        assert_eq!(buf.get(0), Some(&5));
        let (s1, s2) = buf.as_slices();
        assert_eq!(s1, &[2, 3, 4]);
        assert_eq!(s2, &[5]);
    }
}
