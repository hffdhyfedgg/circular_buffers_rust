use core::iter::Rev;
use crate::error::{Result, RingBufError};
use crate::iter::{CBufIter, CBufIterMut};
use crate::math;

/// Immutable view over a ring buffer with power-of-two capacity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CBuf2NView<'a, T> {
    data: &'a [T],
    head: usize,
    len: usize,
    capacity: usize,
    mask: usize,
}

impl<'a, T> CBuf2NView<'a, T> {
    /// Creates a new immutable 2N ring buffer view over `data`.
    ///
    /// # Errors
    /// Returns [`RingBufError::CapacityZero`] if `data` is empty, or
    /// [`RingBufError::NotPowerOfTwo`] if `data.len()` is not a power of two.
    pub fn try_new(data: &'a [T]) -> Result<Self> {
        let capacity = data.len();
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
            data,
            head: capacity - 1,
            len: 0,
            capacity,
            mask: capacity - 1,
        })
    }

    /// Creates a view from existing raw components.
    ///
    /// # Errors
    /// Returns an error if capacity is zero, not a power of two, or `len > capacity`.
    pub fn try_from_raw(data: &'a [T], head: usize, len: usize) -> Result<Self> {
        let capacity = data.len();
        if capacity == 0 {
            return Err(RingBufError::CapacityZero);
        }
        if !capacity.is_power_of_two() {
            #[cfg(feature = "verbose-errors")]
            return Err(RingBufError::NotPowerOfTwo { capacity });
            #[cfg(not(feature = "verbose-errors"))]
            return Err(RingBufError::NotPowerOfTwo);
        }
        if len > capacity {
            #[cfg(feature = "verbose-errors")]
            return Err(RingBufError::StorageTooSmall {
                required: len,
                actual: capacity,
            });
            #[cfg(not(feature = "verbose-errors"))]
            return Err(RingBufError::StorageTooSmall);
        }
        Ok(Self {
            data,
            head: head & (capacity - 1),
            len,
            capacity,
            mask: capacity - 1,
        })
    }

    /// Returns the number of elements currently in the buffer.
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if the buffer contains no elements.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the capacity of the buffer.
    #[inline(always)]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Returns the current head index (physical position of the newest element).
    #[inline(always)]
    pub fn head(&self) -> usize {
        self.head
    }

    /// Returns a reference to the element at relative index `rel` (0 = newest element).
    #[inline(always)]
    pub fn get(&self, rel: usize) -> Option<&'a T> {
        if rel >= self.len {
            None
        } else {
            let phys = math::phys_index_2n(self.head, self.capacity, rel, self.mask);
            Some(&self.data[phys])
        }
    }

    /// Returns the buffer data in logical order (oldest -> newest) as up to two continuous slices.
    #[inline(always)]
    pub fn as_slices(&self) -> (&'a [T], &'a [T]) {
        math::as_slices_2n(self.data, self.head, self.len, self.capacity, self.mask)
    }

    /// Returns an iterator over immutable references in logical order (oldest -> newest).
    #[inline(always)]
    pub fn iter(&self) -> CBufIter<'a, T> {
        let (s1, s2) = self.as_slices();
        CBufIter::new(s1, s2)
    }

    /// Returns an iterator over immutable references in reverse logical order (newest -> oldest).
    #[inline(always)]
    pub fn iter_newest_first(&self) -> Rev<CBufIter<'a, T>> {
        self.iter().rev()
    }
}

impl<'a, T> IntoIterator for CBuf2NView<'a, T> {
    type Item = &'a T;
    type IntoIter = CBufIter<'a, T>;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &CBuf2NView<'a, T> {
    type Item = &'a T;
    type IntoIter = CBufIter<'a, T>;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// Mutable view over a ring buffer with power-of-two capacity.
#[derive(Debug)]
pub struct CBuf2NViewMut<'a, T> {
    data: &'a mut [T],
    head: usize,
    len: usize,
    capacity: usize,
    mask: usize,
}

impl<'a, T> CBuf2NViewMut<'a, T> {
    /// Creates a new mutable 2N ring buffer view over `data`.
    ///
    /// # Errors
    /// Returns [`RingBufError::CapacityZero`] if `data` is empty, or
    /// [`RingBufError::NotPowerOfTwo`] if `data.len()` is not a power of two.
    pub fn try_new(data: &'a mut [T]) -> Result<Self> {
        let capacity = data.len();
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
            data,
            head: capacity - 1,
            len: 0,
            capacity,
            mask: capacity - 1,
        })
    }

    /// Creates a view from existing raw components.
    ///
    /// # Errors
    /// Returns an error if capacity is zero, not a power of two, or `len > capacity`.
    pub fn try_from_raw(data: &'a mut [T], head: usize, len: usize) -> Result<Self> {
        let capacity = data.len();
        if capacity == 0 {
            return Err(RingBufError::CapacityZero);
        }
        if !capacity.is_power_of_two() {
            #[cfg(feature = "verbose-errors")]
            return Err(RingBufError::NotPowerOfTwo { capacity });
            #[cfg(not(feature = "verbose-errors"))]
            return Err(RingBufError::NotPowerOfTwo);
        }
        if len > capacity {
            #[cfg(feature = "verbose-errors")]
            return Err(RingBufError::StorageTooSmall {
                required: len,
                actual: capacity,
            });
            #[cfg(not(feature = "verbose-errors"))]
            return Err(RingBufError::StorageTooSmall);
        }
        Ok(Self {
            data,
            head: head & (capacity - 1),
            len,
            capacity,
            mask: capacity - 1,
        })
    }

    /// Pushes an item into the ring buffer, overwriting the oldest element if full.
    #[inline(always)]
    pub fn push(&mut self, item: T) {
        self.head = math::next_head_2n(self.head, self.mask);
        self.data[self.head] = item;
        if self.len < self.capacity {
            self.len += 1;
        }
    }

    /// Clears the view state (sets length to 0).
    #[inline(always)]
    pub fn clear(&mut self) {
        self.len = 0;
        self.head = self.capacity - 1;
    }

    /// Returns the number of elements currently in the buffer.
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if the buffer contains no elements.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the capacity of the buffer.
    #[inline(always)]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Returns the current head index (physical position of the newest element).
    #[inline(always)]
    pub fn head(&self) -> usize {
        self.head
    }

    /// Returns a reference to the element at relative index `rel` (0 = newest element).
    #[inline(always)]
    pub fn get(&self, rel: usize) -> Option<&T> {
        if rel >= self.len {
            None
        } else {
            let phys = math::phys_index_2n(self.head, self.capacity, rel, self.mask);
            Some(&self.data[phys])
        }
    }

    /// Returns a mutable reference to the element at relative index `rel` (0 = newest element).
    #[inline(always)]
    pub fn get_mut(&mut self, rel: usize) -> Option<&mut T> {
        if rel >= self.len {
            None
        } else {
            let phys = math::phys_index_2n(self.head, self.capacity, rel, self.mask);
            Some(&mut self.data[phys])
        }
    }

    /// Returns the buffer data in logical order (oldest -> newest) as up to two continuous slices.
    #[inline(always)]
    pub fn as_slices(&self) -> (&[T], &[T]) {
        math::as_slices_2n(self.data, self.head, self.len, self.capacity, self.mask)
    }

    /// Returns the buffer data in logical order (oldest -> newest) as up to two continuous mutable slices.
    #[inline(always)]
    pub fn as_slices_mut(&mut self) -> (&mut [T], &mut [T]) {
        math::as_slices_mut_2n(self.data, self.head, self.len, self.capacity, self.mask)
    }

    /// Returns an iterator over immutable references in logical order (oldest -> newest).
    #[inline(always)]
    pub fn iter(&self) -> CBufIter<'_, T> {
        let (s1, s2) = self.as_slices();
        CBufIter::new(s1, s2)
    }

    /// Returns an iterator over immutable references in reverse logical order (newest -> oldest).
    #[inline(always)]
    pub fn iter_newest_first(&self) -> Rev<CBufIter<'_, T>> {
        self.iter().rev()
    }

    /// Returns an iterator over mutable references in logical order (oldest -> newest).
    #[inline(always)]
    pub fn iter_mut(&mut self) -> CBufIterMut<'_, T> {
        let (s1, s2) = self.as_slices_mut();
        CBufIterMut::new(s1, s2)
    }

    /// Borrows an immutable view from this mutable view.
    #[inline(always)]
    pub fn as_view(&self) -> CBuf2NView<'_, T> {
        CBuf2NView {
            data: self.data,
            head: self.head,
            len: self.len,
            capacity: self.capacity,
            mask: self.mask,
        }
    }
}

impl<'a, 'b, T> IntoIterator for &'b CBuf2NViewMut<'a, T> {
    type Item = &'b T;
    type IntoIter = CBufIter<'b, T>;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> IntoIterator for CBuf2NViewMut<'a, T> {
    type Item = &'a mut T;
    type IntoIter = CBufIterMut<'a, T>;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        let (s1, s2) = math::as_slices_mut_2n(self.data, self.head, self.len, self.capacity, self.mask);
        CBufIterMut::new(s1, s2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_view2n_power_of_two_check() {
        let mut buf = [0u32; 3];
        assert!(CBuf2NView::try_new(&buf).is_err());
        assert!(CBuf2NViewMut::try_new(&mut buf).is_err());

        let mut buf4 = [0u32; 4];
        assert!(CBuf2NView::try_new(&buf4).is_ok());
        assert!(CBuf2NViewMut::try_new(&mut buf4).is_ok());
    }

    #[test]
    fn test_view2n_push_get_slices() {
        let mut mem = [0i32; 4];
        let mut view = CBuf2NViewMut::try_new(&mut mem).unwrap();

        assert_eq!(view.len(), 0);
        assert_eq!(view.get(0), None);

        view.push(10);
        assert_eq!(view.len(), 1);
        assert_eq!(view.get(0), Some(&10));
        assert_eq!(view.as_slices(), (&[10][..], &[][..]));

        view.push(20);
        view.push(30);
        assert_eq!(view.len(), 3);
        assert_eq!(view.get(0), Some(&30));
        assert_eq!(view.get(1), Some(&20));
        assert_eq!(view.get(2), Some(&10));
        assert_eq!(view.as_slices(), (&[10, 20, 30][..], &[][..]));

        view.push(40);
        assert_eq!(view.len(), 4);
        assert_eq!(view.as_slices(), (&[10, 20, 30, 40][..], &[][..]));

        // Wrap around
        view.push(50);
        assert_eq!(view.len(), 4);
        assert_eq!(view.get(0), Some(&50));
        assert_eq!(view.get(3), Some(&20));
        let (s1, s2) = view.as_slices();
        assert_eq!(s1, &[20, 30, 40]);
        assert_eq!(s2, &[50]);

        let (m1, m2) = view.as_slices_mut();
        assert_eq!(m1, &[20, 30, 40]);
        assert_eq!(m2, &[50]);

        let mut collected = [0i32; 4];
        for (i, v) in view.iter().copied().enumerate() {
            collected[i] = v;
        }
        assert_eq!(collected, [20, 30, 40, 50]);

        let mut rev_collected = [0i32; 4];
        for (i, v) in view.iter_newest_first().copied().enumerate() {
            rev_collected[i] = v;
        }
        assert_eq!(rev_collected, [50, 40, 30, 20]);
    }
}
