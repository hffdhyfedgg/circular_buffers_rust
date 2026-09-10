use core::iter::Rev;
use core::marker::PhantomData;
use raw_storage::traits::StorageMut;
use crate::error::{Result, RingBufError};
use crate::iter::{CBufIter, CBufIterMut};
use crate::math;
use crate::view2n::{CBuf2NView, CBuf2NViewMut};

/// Owning multi-channel stack of ring buffers with power-of-two capacity per channel.
#[derive(Debug)]
pub struct CBuf2NStack<T, S> {
    storage: S,
    channels: usize,
    capacity: usize,
    mask: usize,
    head: usize,
    len: usize,
    _marker: PhantomData<T>,
}

impl<T, S: StorageMut<Item = T>> CBuf2NStack<T, S> {
    /// Creates a new `CBuf2NStack` with `channels` and `capacity` per channel.
    ///
    /// # Errors
    /// Returns [`RingBufError::CapacityZero`] if `channels == 0` or `capacity == 0`,
    /// [`RingBufError::NotPowerOfTwo`] if `capacity` is not a power of two, or
    /// [`RingBufError::StorageTooSmall`] if storage length is less than `channels * capacity`.
    pub fn try_new(mut storage: S, channels: usize, capacity: usize) -> Result<Self> {
        if channels == 0 || capacity == 0 {
            return Err(RingBufError::CapacityZero);
        }
        if !capacity.is_power_of_two() {
            #[cfg(feature = "verbose-errors")]
            return Err(RingBufError::NotPowerOfTwo { capacity });
            #[cfg(not(feature = "verbose-errors"))]
            return Err(RingBufError::NotPowerOfTwo);
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
            _marker: PhantomData,
        })
    }

    /// Pushes a slice of items (one item per channel) into the stack and advances head.
    ///
    /// `items.len()` must equal `channels`.
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

    /// Advances the shared head position and updates length.
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

    /// Clears stack state (sets length to 0).
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

    /// Returns current number of elements per channel.
    ///
    /// # Panics
    ///
    /// Этот метод никогда не паникует.
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if the stack is empty.
    ///
    /// # Panics
    ///
    /// Этот метод никогда не паникует.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns reference to element at relative index `rel` in channel `ch`.
    ///
    /// # Panics
    ///
    /// Этот метод никогда не паникует.
    #[inline(always)]
    pub fn get(&self, ch: usize, rel: usize) -> Option<&T> {
        if ch >= self.channels || rel >= self.len {
            None
        } else {
            let phys_in_ch = math::phys_index_2n(self.head, self.capacity, rel, self.mask);
            let phys = ch * self.capacity + phys_in_ch;
            debug_assert!(phys < self.storage.as_slice().len());
            self.storage.as_slice().get(phys)
        }
    }

    /// Returns reference to element using signed relative index in channel `ch`.
    ///
    /// # Panics
    ///
    /// Этот метод никогда не паникует.
    #[inline(always)]
    pub fn channel_get_rel(&self, ch: usize, rel: isize) -> Option<&T> {
        let idx = math::rel_to_index(rel, self.len)?;
        self.get(ch, idx)
    }

    /// Returns mutable reference to element at relative index `rel` in channel `ch`.
    ///
    /// # Panics
    ///
    /// Этот метод никогда не паникует.
    #[inline(always)]
    pub fn get_mut(&mut self, ch: usize, rel: usize) -> Option<&mut T> {
        if ch >= self.channels || rel >= self.len {
            None
        } else {
            let phys_in_ch = math::phys_index_2n(self.head, self.capacity, rel, self.mask);
            let phys = ch * self.capacity + phys_in_ch;
            debug_assert!(phys < self.storage.as_mut_slice().len());
            self.storage.as_mut_slice().get_mut(phys)
        }
    }

    /// Returns mutable reference to element using signed relative index in channel `ch`.
    ///
    /// # Panics
    ///
    /// Этот метод никогда не паникует.
    #[inline(always)]
    pub fn channel_get_rel_mut(&mut self, ch: usize, rel: isize) -> Option<&mut T> {
        let idx = math::rel_to_index(rel, self.len)?;
        self.get_mut(ch, idx)
    }

    /// Returns channel `ch` data in logical order as up to two continuous slices.
    ///
    /// # Panics
    ///
    /// Этот метод никогда не паникует.
    #[inline(always)]
    pub fn channel_slices(&self, ch: usize) -> Option<(&[T], &[T])> {
        if ch >= self.channels {
            return None;
        }
        let ch_start = ch * self.capacity;
        let ch_data = self.storage.as_slice().get(ch_start..ch_start + self.capacity)?;
        Some(math::as_slices_2n(ch_data, self.head, self.len, self.capacity, self.mask))
    }

    /// Returns channel `ch` data in logical order as up to two continuous mutable slices.
    ///
    /// # Panics
    ///
    /// Этот метод никогда не паникует.
    #[inline(always)]
    pub fn channel_slices_mut(&mut self, ch: usize) -> Option<(&mut [T], &mut [T])> {
        if ch >= self.channels {
            return None;
        }
        let ch_start = ch * self.capacity;
        let capacity = self.capacity;
        let mask = self.mask;
        let len = self.len;
        let head = self.head;
        let ch_data = self.storage.as_mut_slice().get_mut(ch_start..ch_start + capacity)?;
        Some(math::as_slices_mut_2n(ch_data, head, len, capacity, mask))
    }

    /// Returns an iterator over elements in channel `ch` in logical order.
    ///
    /// # Panics
    ///
    /// Этот метод никогда не паникует.
    #[inline(always)]
    pub fn channel_iter(&self, ch: usize) -> Option<CBufIter<'_, T>> {
        let (s1, s2) = self.channel_slices(ch)?;
        Some(CBufIter::new(s1, s2))
    }

    /// Returns an iterator over elements in channel `ch` in reverse logical order (newest -> oldest).
    ///
    /// # Panics
    ///
    /// Этот метод никогда не паникует.
    #[inline(always)]
    pub fn channel_iter_newest_first(&self, ch: usize) -> Option<Rev<CBufIter<'_, T>>> {
        Some(self.channel_iter(ch)?.rev())
    }

    /// Returns a mutable iterator over elements in channel `ch` in logical order.
    ///
    /// # Panics
    ///
    /// Этот метод никогда не паникует.
    #[inline(always)]
    pub fn channel_iter_mut(&mut self, ch: usize) -> Option<CBufIterMut<'_, T>> {
        let (s1, s2) = self.channel_slices_mut(ch)?;
        Some(CBufIterMut::new(s1, s2))
    }

    /// Returns a 2N view over channel `ch`.
    ///
    /// # Panics
    ///
    /// Этот метод никогда не паникует.
    #[inline(always)]
    pub fn channel_view(&self, ch: usize) -> Option<CBuf2NView<'_, T>> {
        if ch >= self.channels {
            return None;
        }
        let ch_start = ch * self.capacity;
        let ch_data = self.storage.as_slice().get(ch_start..ch_start + self.capacity)?;
        CBuf2NView::try_from_raw(ch_data, self.head, self.len).ok()
    }

    /// Returns a mutable 2N view over channel `ch`.
    ///
    /// # Panics
    ///
    /// Этот метод никогда не паникует.
    #[inline(always)]
    pub fn channel_view_mut(&mut self, ch: usize) -> Option<CBuf2NViewMut<'_, T>> {
        if ch >= self.channels {
            return None;
        }
        let ch_start = ch * self.capacity;
        let ch_data = self.storage.as_mut_slice().get_mut(ch_start..ch_start + self.capacity)?;
        CBuf2NViewMut::try_from_raw(ch_data, self.head, self.len).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use raw_storage::impls::ArrayStorage;

    #[test]
    fn test_cbuf2n_stack() {
        let storage = ArrayStorage::<f32, 8>::default(); // 2 channels x 4 capacity
        let mut stack = CBuf2NStack::try_new(storage, 2, 4).unwrap();

        assert_eq!(stack.channels(), 2);
        assert_eq!(stack.capacity(), 4);

        stack.push_all(&[1.0, 10.0]);
        stack.push_all(&[2.0, 20.0]);

        assert_eq!(stack.len(), 2);
        assert_eq!(stack.get(0, 0), Some(&2.0));
        assert_eq!(stack.get(1, 0), Some(&20.0));
        assert_eq!(stack.get(0, 1), Some(&1.0));
        assert_eq!(stack.get(1, 1), Some(&10.0));

        assert_eq!(stack.channel_get_rel(0, -1), Some(&1.0));
        assert_eq!(stack.channel_get_rel(1, -1), Some(&10.0));

        let mut ch0_rev = [0.0f32; 2];
        for (i, v) in stack.channel_iter_newest_first(0).unwrap().copied().enumerate() {
            ch0_rev[i] = v;
        }
        assert_eq!(ch0_rev, [2.0, 1.0]);
    }
}
