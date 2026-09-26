use core::iter::Rev;
use core::marker::PhantomData;
use raw_storage::traits::StorageMut;
use strided_mem::ring_view::{RingView, RingViewMut};
use strided_mem::{StridedView, StridedViewMut};

use crate::error::{CBufResult, CBufError};
use crate::iter::{CBufIter, CBufIterMut};
use crate::math;

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
    /// Returns [`CBufError::CapacityZero`] if `channels == 0` or `capacity == 0`, or
    /// [`CBufError::StorageTooSmall`] if storage length is less than `channels * capacity`.
    pub fn try_new(mut storage: S, channels: usize, capacity: usize) -> CBufResult<Self> {
        if channels == 0 || capacity == 0 {
            return Err(CBufError::CapacityZero);
        }
        let required = channels * capacity;
        let actual = storage.as_mut_slice().len();
        if actual < required {
            #[cfg(feature = "verbose-errors")]
            return Err(CBufError::StorageTooSmall { required, actual });
            #[cfg(not(feature = "verbose-errors"))]
            return Err(CBufError::StorageTooSmall);
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
    /// `items.len()` must equal or exceed `self.channels`.
    ///
    /// # Errors
    /// Returns [`CBufError::StorageTooSmall`] if `items.len() < self.channels`.
    pub fn push_all(&mut self, items: &[T]) -> CBufResult<()>
    where
        T: Copy,
    {
        if items.len() < self.channels {
            #[cfg(feature = "verbose-errors")]
            return Err(CBufError::StorageTooSmall {
                required: self.channels,
                actual: items.len(),
            });
            #[cfg(not(feature = "verbose-errors"))]
            return Err(CBufError::StorageTooSmall);
        }
        let new_head = math::next_head(self.head, self.capacity);
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
        Ok(())
    }

    /// Advances the shared head position and updates length.
    ///
    /// # Panics
    ///
    /// Этот метод никогда не паникует.
    #[inline(always)]
    pub fn advance_head(&mut self) {
        self.head = math::next_head(self.head, self.capacity);
        if self.len < self.capacity {
            self.len += 1;
        }
    }

    /// Sets item for channel `ch` at current head position without advancing head.
    ///
    /// # Errors
    /// Returns [`CBufError::ChannelOutOfBounds`] if `ch >= channels`.
    ///
    /// # Panics
    ///
    /// Этот метод никогда не паникует.
    #[inline(always)]
    pub fn set_channel(&mut self, ch: usize, item: T) -> CBufResult<()> {
        if ch >= self.channels {
            #[cfg(feature = "verbose-errors")]
            return Err(CBufError::ChannelOutOfBounds {
                channel: ch,
                max_channels: self.channels,
            });
            #[cfg(not(feature = "verbose-errors"))]
            return Err(CBufError::ChannelOutOfBounds);
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
            let phys_in_ch = math::phys_index(self.head, self.capacity, rel);
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
            let phys_in_ch = math::phys_index(self.head, self.capacity, rel);
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
        Some(math::as_slices(ch_data, self.head, self.len, self.capacity))
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
        let len = self.len;
        let head = self.head;
        let ch_data = self.storage.as_mut_slice().get_mut(ch_start..ch_start + capacity)?;
        Some(math::as_slices_mut(ch_data, head, len, capacity))
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

    /// Возвращает кольцевое представление канала `ch`.
    ///
    /// # Errors
    /// `CBufError::ChannelOutOfBounds` если `ch >= self.channels`.
    pub fn channel_ring_view(&self, ch: usize) -> CBufResult<RingView<'_, T, StridedView<'_, T>>> {
        if ch >= self.channels {
            #[cfg(feature = "verbose-errors")]
            return Err(CBufError::ChannelOutOfBounds {
                channel: ch,
                max_channels: self.channels,
            });
            #[cfg(not(feature = "verbose-errors"))]
            return Err(CBufError::ChannelOutOfBounds);
        }
        let ch_start = ch * self.capacity;
        let ch_slice = &self.storage.as_slice()[ch_start..ch_start + self.capacity];
        let sv = StridedView::from_slice(ch_slice);
        Ok(RingView::new_unchecked(sv, self.head, self.len))
    }

    /// Возвращает мутабельное кольцевое представление канала `ch`.
    ///
    /// # Errors
    /// `CBufError::ChannelOutOfBounds` если `ch >= self.channels`.
    pub fn channel_ring_view_mut(&mut self, ch: usize) -> CBufResult<RingViewMut<'_, T, StridedViewMut<'_, T>>> {
        if ch >= self.channels {
            #[cfg(feature = "verbose-errors")]
            return Err(CBufError::ChannelOutOfBounds {
                channel: ch,
                max_channels: self.channels,
            });
            #[cfg(not(feature = "verbose-errors"))]
            return Err(CBufError::ChannelOutOfBounds);
        }
        let ch_start = ch * self.capacity;
        let ch_slice = &mut self.storage.as_mut_slice()[ch_start..ch_start + self.capacity];
        let sv = StridedViewMut::from_mut_slice(ch_slice);
        Ok(RingViewMut::new_unchecked(sv, self.head, self.len))
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

        stack.push_all(&[1.0, 10.0]).unwrap();
        stack.push_all(&[2.0, 20.0]).unwrap();

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

        // Error tests
        assert!(stack.push_all(&[1.0]).is_err());
        assert!(stack.channel_ring_view(2).is_err());

        let view0 = stack.channel_ring_view(0).unwrap();
        use strided_mem::traits::View;
        assert_eq!(view0.get(0), Some(&2.0));
    }
}
