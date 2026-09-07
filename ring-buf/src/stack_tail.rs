use core::marker::PhantomData;
use raw_storage::traits::StorageMut;
use crate::error::{Result, RingBufError};
use crate::iter::{CBufIter, CBufIterMut};

/// Owning multi-channel stack of ring buffers with arbitrary capacity per channel and a tail offset.
#[derive(Debug)]
pub struct CBufStackTail<T, S> {
    storage: S,
    channels: usize,
    capacity: usize,
    head: usize,
    len: usize,
    tail_len: usize,
    _marker: PhantomData<T>,
}

impl<T, S: StorageMut<Item = T>> CBufStackTail<T, S> {
    /// Creates a new `CBufStackTail`.
    ///
    /// # Errors
    /// Returns [`RingBufError::CapacityZero`] if `channels == 0` or `capacity == 0`,
    /// [`RingBufError::InvalidTailLength`] if `tail_len > capacity`, or
    /// [`RingBufError::StorageTooSmall`] if storage length is less than `channels * capacity`.
    pub fn try_new(mut storage: S, channels: usize, capacity: usize, tail_len: usize) -> Result<Self> {
        if channels == 0 || capacity == 0 {
            return Err(RingBufError::CapacityZero);
        }
        if tail_len > capacity {
            #[cfg(feature = "verbose-errors")]
            return Err(RingBufError::InvalidTailLength { tail_len, capacity });
            #[cfg(not(feature = "verbose-errors"))]
            return Err(RingBufError::InvalidTailLength);
        }
        let required = channels * capacity;
        let actual = storage.as_mut_slice().len();
        if actual < required {
            #[cfg(feature = "verbose-errors")]
            return Err(RingBufError::StorageTooSmall { required, actual });
            #[cfg(not(feature = "verbose-errors"))]
            return Err(RingBufError::StorageTooSmall);
        }
        Ok(Self {
            storage,
            channels,
            capacity,
            head: capacity - 1,
            len: 0,
            tail_len,
            _marker: PhantomData,
        })
    }

    /// Resizes the tail length.
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

    /// Pushes a slice of items into the stack and advances head.
    pub fn push_all(&mut self, items: &[T])
    where
        T: Copy,
    {
        if items.len() < self.channels {
            return;
        }
        let mut new_head = self.head + 1;
        if new_head == self.capacity {
            new_head = 0;
        }
        let data = self.storage.as_mut_slice();
        for ch in 0..self.channels {
            let offset = ch * self.capacity + new_head;
            data[offset] = items[ch];
        }
        self.head = new_head;
        if self.len < self.capacity {
            self.len += 1;
        }
    }

    /// Advances the shared head position.
    #[inline(always)]
    pub fn advance_head(&mut self) {
        self.head += 1;
        if self.head == self.capacity {
            self.head = 0;
        }
        if self.len < self.capacity {
            self.len += 1;
        }
    }

    /// Sets item for channel `ch` at current head position without advancing head.
    #[inline(always)]
    pub fn set_channel(&mut self, ch: usize, item: T) -> Result<()> {
        if ch >= self.channels {
            #[cfg(feature = "verbose-errors")]
            return Err(RingBufError::ChannelOutOfBounds {
                channel: ch,
                max_channels: self.channels,
            });
            #[cfg(not(feature = "verbose-errors"))]
            return Err(RingBufError::ChannelOutOfBounds);
        }
        let phys = ch * self.capacity + self.head;
        self.storage.as_mut_slice()[phys] = item;
        Ok(())
    }

    /// Clears stack state.
    #[inline(always)]
    pub fn clear(&mut self) {
        self.len = 0;
        self.head = self.capacity - 1;
    }

    /// Returns the number of channels.
    #[inline(always)]
    pub fn channels(&self) -> usize {
        self.channels
    }

    /// Returns the capacity per channel.
    #[inline(always)]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Returns raw length per channel.
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if effective length is 0.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.effective_len() == 0
    }

    /// Returns reference to element at relative index `rel` in channel `ch` (limited to effective length).
    #[inline(always)]
    pub fn get(&self, ch: usize, rel: usize) -> Option<&T> {
        let eff = self.effective_len();
        if ch >= self.channels || rel >= eff {
            None
        } else {
            let phys_in_ch = (self.head + self.capacity - rel) % self.capacity;
            let phys = ch * self.capacity + phys_in_ch;
            Some(&self.storage.as_slice()[phys])
        }
    }

    /// Returns mutable reference to element at relative index `rel` in channel `ch` (limited to effective length).
    #[inline(always)]
    pub fn get_mut(&mut self, ch: usize, rel: usize) -> Option<&mut T> {
        let eff = self.effective_len();
        if ch >= self.channels || rel >= eff {
            None
        } else {
            let phys_in_ch = (self.head + self.capacity - rel) % self.capacity;
            let phys = ch * self.capacity + phys_in_ch;
            Some(&mut self.storage.as_mut_slice()[phys])
        }
    }

    /// Returns channel `ch` data in logical order (limited to effective length) as up to two continuous slices.
    #[inline(always)]
    pub fn channel_slices(&self, ch: usize) -> Option<(&[T], &[T])> {
        if ch >= self.channels {
            return None;
        }
        let eff = self.effective_len();
        if eff == 0 {
            return Some((&[], &[]));
        }
        let ch_start = ch * self.capacity;
        let ch_data = &self.storage.as_slice()[ch_start..ch_start + self.capacity];
        let oldest = (self.head + self.capacity + 1 - eff) % self.capacity;
        if oldest + eff <= self.capacity {
            Some((&ch_data[oldest..oldest + eff], &[]))
        } else {
            let first_len = self.capacity - oldest;
            let second_len = eff - first_len;
            Some((&ch_data[oldest..self.capacity], &ch_data[..second_len]))
        }
    }

    /// Returns channel `ch` data in logical order (limited to effective length) as up to two continuous mutable slices.
    #[inline(always)]
    pub fn channel_slices_mut(&mut self, ch: usize) -> Option<(&mut [T], &mut [T])> {
        if ch >= self.channels {
            return None;
        }
        let eff = self.effective_len();
        if eff == 0 {
            return Some((&mut [], &mut []));
        }
        let ch_start = ch * self.capacity;
        let capacity = self.capacity;
        let head = self.head;
        let ch_data = &mut self.storage.as_mut_slice()[ch_start..ch_start + capacity];
        let oldest = (head + capacity + 1 - eff) % capacity;
        if oldest + eff <= capacity {
            let slice = &mut ch_data[oldest..oldest + eff];
            Some((slice, &mut []))
        } else {
            let first_len = capacity - oldest;
            let second_len = eff - first_len;
            let (left, right) = ch_data.split_at_mut(oldest);
            let (slice1, _) = right.split_at_mut(first_len);
            let (slice2, _) = left.split_at_mut(second_len);
            Some((slice1, slice2))
        }
    }

    /// Returns iterator over channel `ch` effective elements in logical order.
    #[inline(always)]
    pub fn channel_iter(&self, ch: usize) -> Option<CBufIter<'_, T>> {
        let (s1, s2) = self.channel_slices(ch)?;
        Some(CBufIter::new(s1, s2))
    }

    /// Returns mutable iterator over channel `ch` effective elements in logical order.
    #[inline(always)]
    pub fn channel_iter_mut(&mut self, ch: usize) -> Option<CBufIterMut<'_, T>> {
        let (s1, s2) = self.channel_slices_mut(ch)?;
        Some(CBufIterMut::new(s1, s2))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use raw_storage::impls::ArrayStorage;

    #[test]
    fn test_cbuf_stack_tail() {
        let storage = ArrayStorage::<f32, 6>::default(); // 2 channels x 3 capacity
        let mut st = CBufStackTail::try_new(storage, 2, 3, 1).unwrap();

        st.push_all(&[1.0, 10.0]);
        st.push_all(&[2.0, 20.0]);
        st.push_all(&[3.0, 30.0]);

        assert_eq!(st.len(), 3);
        assert_eq!(st.effective_len(), 2);

        assert_eq!(st.get(0, 0), Some(&3.0));
        assert_eq!(st.get(0, 1), Some(&2.0));
        assert_eq!(st.get(0, 2), None); // tail truncated 1.0
    }
}
