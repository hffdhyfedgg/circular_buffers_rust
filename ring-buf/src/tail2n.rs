use core::iter::Rev;
use core::marker::PhantomData;
use raw_storage::traits::StorageMut;
use crate::error::{Result, RingBufError};
use crate::iter::{CBufIter, CBufIterMut};
use crate::math;

/// Owning single ring buffer with power-of-two capacity and a tail offset.
#[derive(Debug)]
pub struct CBuf2NTail<T, S> {
    storage: S,
    head: usize,
    len: usize,
    capacity: usize,
    mask: usize,
    tail_len: usize,
    _marker: PhantomData<T>,
}

impl<T, S: StorageMut<Item = T>> CBuf2NTail<T, S> {
    /// Creates a new `CBuf2NTail` backed by `storage` with specified `tail_len`.
    ///
    /// # Errors
    /// Returns [`RingBufError::CapacityZero`] if capacity is zero,
    /// [`RingBufError::NotPowerOfTwo`] if capacity is not a power of two, or
    /// [`RingBufError::InvalidTailLength`] if `tail_len > capacity`.
    pub fn try_new(mut storage: S, tail_len: usize) -> Result<Self> {
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
        if tail_len > capacity {
            #[cfg(feature = "verbose-errors")]
            return Err(RingBufError::InvalidTailLength { tail_len, capacity });
            #[cfg(not(feature = "verbose-errors"))]
            return Err(RingBufError::InvalidTailLength);
        }
        Ok(Self {
            storage,
            head: capacity - 1,
            len: 0,
            capacity,
            mask: capacity - 1,
            tail_len,
            _marker: PhantomData,
        })
    }

    /// Resizes the tail length.
    ///
    /// # Errors
    /// Returns [`RingBufError::InvalidTailLength`] if `new_tail_len > capacity`.
    pub fn resize_tail(&mut self, new_tail_len: usize) -> Result<()> {
        if new_tail_len > self.capacity {
            #[cfg(feature = "verbose-errors")]
            return Err(RingBufError::InvalidTailLength {
                tail_len: new_tail_len,
                capacity: self.capacity,
            });
            #[cfg(not(feature = "verbose-errors"))]
            return Err(RingBufError::InvalidTailLength);
        }
        self.tail_len = new_tail_len;
        Ok(())
    }

    /// Returns the effective length accessible for reading: `min(len, capacity - tail_len)`.
    #[inline(always)]
    pub fn effective_len(&self) -> usize {
        self.len.min(self.capacity - self.tail_len)
    }

    /// Returns the tail length.
    #[inline(always)]
    pub fn tail_len(&self) -> usize {
        self.tail_len
    }

    /// Returns the raw number of elements stored.
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns the capacity.
    #[inline(always)]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Returns `true` if effective length is 0.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.effective_len() == 0
    }

    /// Pushes an item into full capacity without tail restriction.
    #[inline(always)]
    pub fn push(&mut self, item: T) {
        self.head = math::next_head_2n(self.head, self.mask);
        self.storage.as_mut_slice()[self.head] = item;
        if self.len < self.capacity {
            self.len += 1;
        }
    }

    /// Clears buffer state.
    #[inline(always)]
    pub fn clear(&mut self) {
        self.len = 0;
        self.head = self.capacity - 1;
    }

    /// Returns reference to element at relative index `rel` (limited to effective length).
    #[inline(always)]
    pub fn get(&self, rel: usize) -> Option<&T> {
        let eff = self.effective_len();
        if rel >= eff {
            None
        } else {
            let phys = math::phys_index_2n(self.head, self.capacity, rel, self.mask);
            Some(&self.storage.as_slice()[phys])
        }
    }

    /// Returns mutable reference to element at relative index `rel` (limited to effective length).
    #[inline(always)]
    pub fn get_mut(&mut self, rel: usize) -> Option<&mut T> {
        let eff = self.effective_len();
        if rel >= eff {
            None
        } else {
            let phys = math::phys_index_2n(self.head, self.capacity, rel, self.mask);
            Some(&mut self.storage.as_mut_slice()[phys])
        }
    }

    /// Returns data in logical order (limited to effective length) as up to two continuous slices.
    #[inline(always)]
    pub fn as_slices(&self) -> (&[T], &[T]) {
        let eff = self.effective_len();
        math::as_slices_2n(self.storage.as_slice(), self.head, eff, self.capacity, self.mask)
    }

    /// Returns data in logical order (limited to effective length) as up to two continuous mutable slices.
    #[inline(always)]
    pub fn as_slices_mut(&mut self) -> (&mut [T], &mut [T]) {
        let eff = self.effective_len();
        math::as_slices_mut_2n(self.storage.as_mut_slice(), self.head, eff, self.capacity, self.mask)
    }

    /// Returns iterator over effective elements in logical order.
    #[inline(always)]
    pub fn iter(&self) -> CBufIter<'_, T> {
        let (s1, s2) = self.as_slices();
        CBufIter::new(s1, s2)
    }

    /// Returns iterator over effective elements in reverse logical order (newest -> oldest).
    #[inline(always)]
    pub fn iter_newest_first(&self) -> Rev<CBufIter<'_, T>> {
        self.iter().rev()
    }

    /// Returns mutable iterator over effective elements in logical order.
    #[inline(always)]
    pub fn iter_mut(&mut self) -> CBufIterMut<'_, T> {
        let (s1, s2) = self.as_slices_mut();
        CBufIterMut::new(s1, s2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use raw_storage::impls::ArrayStorage;

    #[test]
    fn test_cbuf2n_tail() {
        let storage = ArrayStorage::<i32, 4>::default();
        let mut tail_buf = CBuf2NTail::try_new(storage, 1).unwrap();

        assert_eq!(tail_buf.tail_len(), 1);
        assert_eq!(tail_buf.capacity(), 4);
        assert_eq!(tail_buf.effective_len(), 0);

        tail_buf.push(10);
        tail_buf.push(20);
        tail_buf.push(30);
        tail_buf.push(40);

        // len = 4, capacity - tail_len = 3. Effective length = 3.
        assert_eq!(tail_buf.len(), 4);
        assert_eq!(tail_buf.effective_len(), 3);

        assert_eq!(tail_buf.get(0), Some(&40));
        assert_eq!(tail_buf.get(1), Some(&30));
        assert_eq!(tail_buf.get(2), Some(&20));
        assert_eq!(tail_buf.get(3), None); // tail truncates element 10

        let mut rev = [0i32; 3];
        for (i, v) in tail_buf.iter_newest_first().copied().enumerate() {
            rev[i] = v;
        }
        assert_eq!(rev, [40, 30, 20]);

        tail_buf.resize_tail(0).unwrap();
        assert_eq!(tail_buf.effective_len(), 4);
        assert_eq!(tail_buf.get(3), Some(&10));
    }
}
