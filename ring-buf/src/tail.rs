use core::marker::PhantomData;
use raw_storage::traits::StorageMut;
use crate::error::{Result, RingBufError};
use crate::iter::{CBufIter, CBufIterMut};

/// Owning single ring buffer with arbitrary capacity and a tail offset.
#[derive(Debug)]
pub struct CBufTail<T, S> {
    storage: S,
    head: usize,
    len: usize,
    capacity: usize,
    tail_len: usize,
    _marker: PhantomData<T>,
}

impl<T, S: StorageMut<Item = T>> CBufTail<T, S> {
    /// Creates a new `CBufTail` backed by `storage` with specified `tail_len`.
    ///
    /// # Errors
    /// Returns [`RingBufError::CapacityZero`] if capacity is zero, or
    /// [`RingBufError::InvalidTailLength`] if `tail_len > capacity`.
    pub fn try_new(mut storage: S, tail_len: usize) -> Result<Self> {
        let capacity = storage.as_mut_slice().len();
        if capacity == 0 {
            return Err(RingBufError::CapacityZero);
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
        self.head += 1;
        if self.head == self.capacity {
            self.head = 0;
        }
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
            let phys = (self.head + self.capacity - rel) % self.capacity;
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
            let phys = (self.head + self.capacity - rel) % self.capacity;
            Some(&mut self.storage.as_mut_slice()[phys])
        }
    }

    /// Returns data in logical order (limited to effective length) as up to two continuous slices.
    #[inline(always)]
    pub fn as_slices(&self) -> (&[T], &[T]) {
        let eff = self.effective_len();
        if eff == 0 {
            return (&[], &[]);
        }
        let oldest = (self.head + self.capacity + 1 - eff) % self.capacity;
        let data = self.storage.as_slice();
        if oldest + eff <= self.capacity {
            (&data[oldest..oldest + eff], &[])
        } else {
            let first_len = self.capacity - oldest;
            let second_len = eff - first_len;
            (&data[oldest..self.capacity], &data[..second_len])
        }
    }

    /// Returns data in logical order (limited to effective length) as up to two continuous mutable slices.
    #[inline(always)]
    pub fn as_slices_mut(&mut self) -> (&mut [T], &mut [T]) {
        let eff = self.effective_len();
        if eff == 0 {
            return (&mut [], &mut []);
        }
        let oldest = (self.head + self.capacity + 1 - eff) % self.capacity;
        let capacity = self.capacity;
        let data = self.storage.as_mut_slice();
        if oldest + eff <= capacity {
            let slice = &mut data[oldest..oldest + eff];
            (slice, &mut [])
        } else {
            let first_len = capacity - oldest;
            let second_len = eff - first_len;
            let (left, right) = data.split_at_mut(oldest);
            let (slice1, _) = right.split_at_mut(first_len);
            let (slice2, _) = left.split_at_mut(second_len);
            (slice1, slice2)
        }
    }

    /// Returns iterator over effective elements in logical order.
    #[inline(always)]
    pub fn iter(&self) -> CBufIter<'_, T> {
        let (s1, s2) = self.as_slices();
        CBufIter::new(s1, s2)
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
    fn test_cbuf_tail() {
        let storage = ArrayStorage::<i32, 5>::default();
        let mut tail_buf = CBufTail::try_new(storage, 2).unwrap();

        assert_eq!(tail_buf.tail_len(), 2);
        assert_eq!(tail_buf.capacity(), 5);
        assert_eq!(tail_buf.effective_len(), 0);

        for val in 1..=5 {
            tail_buf.push(val * 10);
        }

        // len = 5, capacity - tail_len = 3. Effective length = 3.
        assert_eq!(tail_buf.len(), 5);
        assert_eq!(tail_buf.effective_len(), 3);

        assert_eq!(tail_buf.get(0), Some(&50));
        assert_eq!(tail_buf.get(1), Some(&40));
        assert_eq!(tail_buf.get(2), Some(&30));
        assert_eq!(tail_buf.get(3), None);

        tail_buf.resize_tail(1).unwrap();
        assert_eq!(tail_buf.effective_len(), 4);
        assert_eq!(tail_buf.get(3), Some(&20));
    }
}
