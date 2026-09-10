use core::iter::Rev;
use core::marker::PhantomData;
use raw_storage::traits::StorageMut;
use crate::error::{Result, RingBufError};
use crate::iter::{CBufIter, CBufIterMut};
use crate::math;

/// Owning multi-channel stack of ring buffers with power-of-two capacity per channel and a tail offset.
#[derive(Debug)]
pub struct CBuf2NStackTail<T, S> {
    storage: S,
    channels: usize,
    capacity: usize,
    mask: usize,
    head: usize,
    len: usize,
    tail_len: usize,
    _marker: PhantomData<T>,
}

impl<T, S: StorageMut<Item = T>> CBuf2NStackTail<T, S> {
    /// Creates a new `CBuf2NStackTail`.
    ///
    /// # Errors
    /// Returns [`RingBufError::CapacityZero`] if `channels == 0` or `capacity == 0`,
    /// [`RingBufError::NotPowerOfTwo`] if `capacity` is not a power of two,
    /// [`RingBufError::InvalidTailLength`] if `tail_len > capacity`, or
    /// [`RingBufError::StorageTooSmall`] if storage length is less than `channels * capacity`.
    pub fn try_new(mut storage: S, channels: usize, capacity: usize, tail_len: usize) -> Result<Self> {
        if channels == 0 || capacity == 0 {
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
            mask: capacity - 1,
            head: capacity - 1,
            len: 0,
            tail_len,
            _marker: PhantomData,
        })
    }

    /// Resizes the tail length.
    ///
    /// # Errors
    /// Returns [`RingBufError::InvalidTailLength`] if `new_tail_len > capacity`.
    ///
    /// # Panics
    ///
    /// Этот метод никогда не паникует.
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
    ///
    /// # Panics
    ///
    /// Этот метод никогда не паникует.
    #[inline(always)]
    pub fn effective_len(&self) -> usize {
        self.len.min(self.capacity - self.tail_len)
    }

    /// Returns the tail length.
    ///
    /// # Panics
    ///
    /// Этот метод никогда не паникует.
    #[inline(always)]
    pub fn tail_len(&self) -> usize {
        self.tail_len
    }

    /// Pushes a slice of items into the stack and advances head.
    ///
    /// # Panics
    ///
    /// Этот метод никогда не паникует.
    pub fn push_all(&mut self, items: &[T])
    where
        T: Copy,
    {
        if items.len() < self.channels {
            return;
        }
        let new_head = math::next_head_2n(self.head, self.mask);
        let data = self.storage.as_mut_slice();
        for ch in 0..self.channels {
            let offset = ch * self.capacity + new_head;
            debug_assert!(offset < data.len());
            if let (Some(slot), Some(&item)) = (data.get_mut(offset), items.get(ch)) {
                *slot = item;
            }
        }
        self.head = new_head;
        if self.len < self.capacity {
            self.len += 1;
        }
    }

    /// Advances the shared head position.
    ///
    /// # Panics
    ///
    /// Этот метод никогда не паникует.
    #[inline(always)]
    pub fn advance_head(&mut self) {
        self.head = math::next_head_2n(self.head, self.mask);
        if self.len < self.capacity {
            self.len += 1;
        }
    }

    /// Sets item for channel `ch` at current head position without advancing head.
    ///
    /// # Errors
    /// Returns [`RingBufError::ChannelOutOfBounds`] if `ch >= channels`.
    ///
    /// # Panics
    ///
    /// Этот метод никогда не паникует.
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
        debug_assert!(phys < self.storage.as_mut_slice().len());
        if let Some(slot) = self.storage.as_mut_slice().get_mut(phys) {
            *slot = item;
        }
        Ok(())
    }

    /// Clears stack state.
    ///
    /// # Panics
    ///
    /// Этот метод никогда не паникует.
    #[inline(always)]
    pub fn clear(&mut self) {
        self.len = 0;
        self.head = self.capacity - 1;
    }

    /// Returns the number of channels.
    ///
    /// # Panics
    ///
    /// Этот метод никогда не паникует.
    #[inline(always)]
    pub fn channels(&self) -> usize {
        self.channels
    }

    /// Returns the capacity per channel.
    ///
    /// # Panics
    ///
    /// Этот метод никогда не паникует.
    #[inline(always)]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Returns raw length per channel.
    ///
    /// # Panics
    ///
    /// Этот метод никогда не паникует.
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if effective length is 0.
    ///
    /// # Panics
    ///
    /// Этот метод никогда не паникует.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.effective_len() == 0
    }

    /// Returns reference to element at relative index `rel` in channel `ch` (limited to effective length).
    ///
    /// # Panics
    ///
    /// Этот метод никогда не паникует.
    #[inline(always)]
    pub fn get(&self, ch: usize, rel: usize) -> Option<&T> {
        let eff = self.effective_len();
        if ch >= self.channels || rel >= eff {
            None
        } else {
            let phys_in_ch = math::phys_index_2n(self.head, self.capacity, rel, self.mask);
            let phys = ch * self.capacity + phys_in_ch;
            debug_assert!(phys < self.storage.as_slice().len());
            self.storage.as_slice().get(phys)
        }
    }

    /// Returns reference to element using signed relative index in channel `ch` (limited to effective length).
    ///
    /// # Panics
    ///
    /// Этот метод никогда не паникует.
    #[inline(always)]
    pub fn channel_get_rel(&self, ch: usize, rel: isize) -> Option<&T> {
        let eff = self.effective_len();
        let idx = math::rel_to_index(rel, eff)?;
        self.get(ch, idx)
    }

    /// Returns mutable reference to element at relative index `rel` in channel `ch` (limited to effective length).
    ///
    /// # Panics
    ///
    /// Этот метод никогда не паникует.
    #[inline(always)]
    pub fn get_mut(&mut self, ch: usize, rel: usize) -> Option<&mut T> {
        let eff = self.effective_len();
        if ch >= self.channels || rel >= eff {
            None
        } else {
            let phys_in_ch = math::phys_index_2n(self.head, self.capacity, rel, self.mask);
            let phys = ch * self.capacity + phys_in_ch;
            debug_assert!(phys < self.storage.as_mut_slice().len());
            self.storage.as_mut_slice().get_mut(phys)
        }
    }

    /// Returns mutable reference to element using signed relative index in channel `ch` (limited to effective length).
    ///
    /// # Panics
    ///
    /// Этот метод никогда не паникует.
    #[inline(always)]
    pub fn channel_get_rel_mut(&mut self, ch: usize, rel: isize) -> Option<&mut T> {
        let eff = self.effective_len();
        let idx = math::rel_to_index(rel, eff)?;
        self.get_mut(ch, idx)
    }

    /// Returns channel `ch` data in logical order (limited to effective length) as up to two continuous slices.
    ///
    /// # Panics
    ///
    /// Этот метод никогда не паникует.
    #[inline(always)]
    pub fn channel_slices(&self, ch: usize) -> Option<(&[T], &[T])> {
        if ch >= self.channels {
            return None;
        }
        let eff = self.effective_len();
        let ch_start = ch * self.capacity;
        let ch_data = self.storage.as_slice().get(ch_start..ch_start + self.capacity)?;
        Some(math::as_slices_2n(ch_data, self.head, eff, self.capacity, self.mask))
    }

    /// Returns channel `ch` data in logical order (limited to effective length) as up to two continuous mutable slices.
    ///
    /// # Panics
    ///
    /// Этот метод никогда не паникует.
    #[inline(always)]
    pub fn channel_slices_mut(&mut self, ch: usize) -> Option<(&mut [T], &mut [T])> {
        if ch >= self.channels {
            return None;
        }
        let eff = self.effective_len();
        let ch_start = ch * self.capacity;
        let capacity = self.capacity;
        let mask = self.mask;
        let head = self.head;
        let ch_data = self.storage.as_mut_slice().get_mut(ch_start..ch_start + capacity)?;
        Some(math::as_slices_mut_2n(ch_data, head, eff, capacity, mask))
    }

    /// Returns iterator over channel `ch` effective elements in logical order.
    ///
    /// # Panics
    ///
    /// Этот метод никогда не паникует.
    #[inline(always)]
    pub fn channel_iter(&self, ch: usize) -> Option<CBufIter<'_, T>> {
        let (s1, s2) = self.channel_slices(ch)?;
        Some(CBufIter::new(s1, s2))
    }

    /// Returns iterator over channel `ch` effective elements in reverse logical order (newest -> oldest).
    ///
    /// # Panics
    ///
    /// Этот метод никогда не паникует.
    #[inline(always)]
    pub fn channel_iter_newest_first(&self, ch: usize) -> Option<Rev<CBufIter<'_, T>>> {
        Some(self.channel_iter(ch)?.rev())
    }

    /// Returns mutable iterator over channel `ch` effective elements in logical order.
    ///
    /// # Panics
    ///
    /// Этот метод никогда не паникует.
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
    fn test_cbuf2n_stack_tail() {
        let storage = ArrayStorage::<f32, 8>::default(); // 2 channels x 4 capacity
        let mut st = CBuf2NStackTail::try_new(storage, 2, 4, 1).unwrap();

        st.push_all(&[1.0, 10.0]);
        st.push_all(&[2.0, 20.0]);
        st.push_all(&[3.0, 30.0]);
        st.push_all(&[4.0, 40.0]);

        assert_eq!(st.len(), 4);
        assert_eq!(st.effective_len(), 3);

        assert_eq!(st.get(0, 0), Some(&4.0));
        assert_eq!(st.get(0, 2), Some(&2.0));
        assert_eq!(st.get(0, 3), None); // tail truncated 1.0

        assert_eq!(st.channel_get_rel(0, -1), Some(&2.0));
        assert_eq!(st.channel_get_rel(0, -3), Some(&4.0));
        assert_eq!(st.channel_get_rel(0, -4), None);

        let mut ch0_rev = [0.0f32; 3];
        for (i, v) in st.channel_iter_newest_first(0).unwrap().copied().enumerate() {
            ch0_rev[i] = v;
        }
        assert_eq!(ch0_rev, [4.0, 3.0, 2.0]);
    }
}
