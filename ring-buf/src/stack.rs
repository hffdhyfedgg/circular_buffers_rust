use core::marker::PhantomData;
use raw_storage::traits::StorageMut;
use crate::error::{Result, RingBufError};
use crate::iter::{CBufIter, CBufIterMut};
use crate::view::{CBufView, CBufViewMut};

/// Owning multi-channel stack of ring buffers with arbitrary capacity per channel.
#[derive(Debug)]
pub struct CBufStack<T, S> {
    storage: S,
    channels: usize,
    capacity: usize,
    head: usize,
    len: usize,
    _marker: PhantomData<T>,
}

impl<T, S: StorageMut<Item = T>> CBufStack<T, S> {
    /// Creates a new `CBufStack` with `channels` and `capacity` per channel.
    ///
    /// # Errors
    /// Returns [`RingBufError::CapacityZero`] if `channels == 0` or `capacity == 0`, or
    /// [`RingBufError::StorageTooSmall`] if storage length is less than `channels * capacity`.
    pub fn try_new(mut storage: S, channels: usize, capacity: usize) -> Result<Self> {
        if channels == 0 || capacity == 0 {
            return Err(RingBufError::CapacityZero);
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
            _marker: PhantomData,
        })
    }

    /// Pushes a slice of items (one item per channel) into the stack and advances head.
    ///
    /// `items.len()` must equal `channels`.
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

    /// Advances the shared head position and updates length.
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

    /// Clears stack state (sets length to 0).
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

    /// Returns current number of elements per channel.
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if the stack is empty.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns reference to element at relative index `rel` in channel `ch`.
    #[inline(always)]
    pub fn get(&self, ch: usize, rel: usize) -> Option<&T> {
        if ch >= self.channels || rel >= self.len {
            None
        } else {
            let phys_in_ch = (self.head + self.capacity - rel) % self.capacity;
            let phys = ch * self.capacity + phys_in_ch;
            Some(&self.storage.as_slice()[phys])
        }
    }

    /// Returns mutable reference to element at relative index `rel` in channel `ch`.
    #[inline(always)]
    pub fn get_mut(&mut self, ch: usize, rel: usize) -> Option<&mut T> {
        if ch >= self.channels || rel >= self.len {
            None
        } else {
            let phys_in_ch = (self.head + self.capacity - rel) % self.capacity;
            let phys = ch * self.capacity + phys_in_ch;
            Some(&mut self.storage.as_mut_slice()[phys])
        }
    }

    /// Returns channel `ch` data in logical order as up to two continuous slices.
    #[inline(always)]
    pub fn channel_slices(&self, ch: usize) -> Option<(&[T], &[T])> {
        if ch >= self.channels {
            return None;
        }
        if self.len == 0 {
            return Some((&[], &[]));
        }
        let ch_start = ch * self.capacity;
        let ch_data = &self.storage.as_slice()[ch_start..ch_start + self.capacity];
        let oldest = (self.head + self.capacity + 1 - self.len) % self.capacity;
        if oldest + self.len <= self.capacity {
            Some((&ch_data[oldest..oldest + self.len], &[]))
        } else {
            let first_len = self.capacity - oldest;
            let second_len = self.len - first_len;
            Some((&ch_data[oldest..self.capacity], &ch_data[..second_len]))
        }
    }

    /// Returns channel `ch` data in logical order as up to two continuous mutable slices.
    #[inline(always)]
    pub fn channel_slices_mut(&mut self, ch: usize) -> Option<(&mut [T], &mut [T])> {
        if ch >= self.channels {
            return None;
        }
        if self.len == 0 {
            return Some((&mut [], &mut []));
        }
        let ch_start = ch * self.capacity;
        let capacity = self.capacity;
        let len = self.len;
        let head = self.head;
        let ch_data = &mut self.storage.as_mut_slice()[ch_start..ch_start + capacity];
        let oldest = (head + capacity + 1 - len) % capacity;
        if oldest + len <= capacity {
            let slice = &mut ch_data[oldest..oldest + len];
            Some((slice, &mut []))
        } else {
            let first_len = capacity - oldest;
            let second_len = len - first_len;
            let (left, right) = ch_data.split_at_mut(oldest);
            let (slice1, _) = right.split_at_mut(first_len);
            let (slice2, _) = left.split_at_mut(second_len);
            Some((slice1, slice2))
        }
    }

    /// Returns an iterator over elements in channel `ch` in logical order.
    #[inline(always)]
    pub fn channel_iter(&self, ch: usize) -> Option<CBufIter<'_, T>> {
        let (s1, s2) = self.channel_slices(ch)?;
        Some(CBufIter::new(s1, s2))
    }

    /// Returns a mutable iterator over elements in channel `ch` in logical order.
    #[inline(always)]
    pub fn channel_iter_mut(&mut self, ch: usize) -> Option<CBufIterMut<'_, T>> {
        let (s1, s2) = self.channel_slices_mut(ch)?;
        Some(CBufIterMut::new(s1, s2))
    }

    /// Returns a view over channel `ch`.
    #[inline(always)]
    pub fn channel_view(&self, ch: usize) -> Option<CBufView<'_, T>> {
        if ch >= self.channels {
            return None;
        }
        let ch_start = ch * self.capacity;
        let ch_data = &self.storage.as_slice()[ch_start..ch_start + self.capacity];
        CBufView::try_from_raw(ch_data, self.head, self.len).ok()
    }

    /// Returns a mutable view over channel `ch`.
    #[inline(always)]
    pub fn channel_view_mut(&mut self, ch: usize) -> Option<CBufViewMut<'_, T>> {
        if ch >= self.channels {
            return None;
        }
        let ch_start = ch * self.capacity;
        let ch_data = &mut self.storage.as_mut_slice()[ch_start..ch_start + self.capacity];
        CBufViewMut::try_from_raw(ch_data, self.head, self.len).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use raw_storage::impls::ArrayStorage;

    #[test]
    fn test_cbuf_stack() {
        let storage = ArrayStorage::<f32, 6>::default(); // 2 channels x 3 capacity
        let mut stack = CBufStack::try_new(storage, 2, 3).unwrap();

        assert_eq!(stack.channels(), 2);
        assert_eq!(stack.capacity(), 3);

        stack.push_all(&[1.0, 10.0]);
        stack.push_all(&[2.0, 20.0]);

        assert_eq!(stack.len(), 2);
        assert_eq!(stack.get(0, 0), Some(&2.0));
        assert_eq!(stack.get(1, 0), Some(&20.0));
        assert_eq!(stack.get(0, 1), Some(&1.0));
        assert_eq!(stack.get(1, 1), Some(&10.0));
    }
}
