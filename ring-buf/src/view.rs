use core::iter::Rev;
use crate::error::{Result, RingBufError};
use crate::iter::{CBufIter, CBufIterMut};
use crate::math;

/// Immutable view over a ring buffer with arbitrary capacity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CBufView<'a, T> {
    data: &'a [T],
    head: usize,
    len: usize,
    capacity: usize,
}

impl<'a, T> CBufView<'a, T> {
    /// Creates a new immutable ring buffer view over `data`.
    ///
    /// # Errors
    /// Returns [`RingBufError::CapacityZero`] if `data` is empty.
    pub fn try_new(data: &'a [T]) -> Result<Self> {
        let capacity = data.len();
        if capacity == 0 {
            return Err(RingBufError::CapacityZero);
        }
        Ok(Self {
            data,
            head: capacity - 1,
            len: 0,
            capacity,
        })
    }

    /// Creates a view from existing raw components.
    ///
    /// # Errors
    /// Returns an error if capacity is zero or `len > capacity`.
    pub fn try_from_raw(data: &'a [T], head: usize, len: usize) -> Result<Self> {
        let capacity = data.len();
        if capacity == 0 {
            return Err(RingBufError::CapacityZero);
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
            head: if capacity > 0 { head % capacity } else { 0 },
            len,
            capacity,
        })
    }

    /// Creates a view from raw components without checking invariants in release mode.
    ///
    /// # Invariants
    /// `capacity > 0`, `len <= capacity`, `head < capacity`.
    ///
    /// # Panics
    ///
    /// Этот метод никогда не паникует.
    #[inline(always)]
    pub(crate) fn from_raw_unchecked(data: &'a [T], head: usize, len: usize) -> Self {
        let capacity = data.len();
        debug_assert!(capacity > 0);
        debug_assert!(len <= capacity);
        debug_assert!(head < capacity);
        Self {
            data,
            head,
            len,
            capacity,
        }
    }

    /// Returns the number of elements currently in the buffer.
    ///
    /// # Panics
    ///
    /// Этот метод никогда не паникует.
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if the buffer contains no elements.
    ///
    /// # Panics
    ///
    /// Этот метод никогда не паникует.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the capacity of the buffer.
    ///
    /// # Panics
    ///
    /// Этот метод никогда не паникует.
    #[inline(always)]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Returns the current head index (physical position of the newest element).
    ///
    /// # Panics
    ///
    /// Этот метод никогда не паникует.
    #[inline(always)]
    pub fn head(&self) -> usize {
        self.head
    }

    /// Returns a reference to the element at relative index `rel` (0 = newest element).
    ///
    /// # Panics
    ///
    /// Этот метод никогда не паникует.
    #[inline(always)]
    pub fn get(&self, rel: usize) -> Option<&'a T> {
        if rel >= self.len {
            None
        } else {
            let phys = math::phys_index(self.head, self.capacity, rel);
            debug_assert!(phys < self.capacity);
            self.data.get(phys)
        }
    }

    /// Returns a reference to the element using a signed relative index (`rel >= 0` from head, `rel < 0` from oldest).
    ///
    /// # Panics
    ///
    /// Этот метод никогда не паникует.
    #[inline(always)]
    pub fn get_rel(&self, rel: isize) -> Option<&'a T> {
        let idx = math::rel_to_index(rel, self.len)?;
        self.get(idx)
    }

    /// Returns the buffer data in logical order (oldest -> newest) as up to two continuous slices.
    ///
    /// # Panics
    ///
    /// Этот метод никогда не паникует.
    #[inline(always)]
    pub fn as_slices(&self) -> (&'a [T], &'a [T]) {
        math::as_slices(self.data, self.head, self.len, self.capacity)
    }

    /// Returns an iterator over immutable references in logical order (oldest -> newest).
    ///
    /// # Panics
    ///
    /// Этот метод никогда не паникует.
    #[inline(always)]
    pub fn iter(&self) -> CBufIter<'a, T> {
        let (s1, s2) = self.as_slices();
        CBufIter::new(s1, s2)
    }

    /// Returns an iterator over immutable references in reverse logical order (newest -> oldest).
    ///
    /// # Panics
    ///
    /// Этот метод никогда не паникует.
    #[inline(always)]
    pub fn iter_newest_first(&self) -> Rev<CBufIter<'a, T>> {
        self.iter().rev()
    }
}

impl<'a, T> IntoIterator for CBufView<'a, T> {
    type Item = &'a T;
    type IntoIter = CBufIter<'a, T>;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &CBufView<'a, T> {
    type Item = &'a T;
    type IntoIter = CBufIter<'a, T>;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// Mutable view over a ring buffer with arbitrary capacity.
#[derive(Debug)]
pub struct CBufViewMut<'a, T> {
    data: &'a mut [T],
    head: usize,
    len: usize,
    capacity: usize,
}

impl<'a, T> CBufViewMut<'a, T> {
    /// Creates a new mutable ring buffer view over `data`.
    ///
    /// # Errors
    /// Returns [`RingBufError::CapacityZero`] if `data` is empty.
    pub fn try_new(data: &'a mut [T]) -> Result<Self> {
        let capacity = data.len();
        if capacity == 0 {
            return Err(RingBufError::CapacityZero);
        }
        Ok(Self {
            data,
            head: capacity - 1,
            len: 0,
            capacity,
        })
    }

    /// Creates a view from existing raw components.
    ///
    /// # Errors
    /// Returns an error if capacity is zero or `len > capacity`.
    pub fn try_from_raw(data: &'a mut [T], head: usize, len: usize) -> Result<Self> {
        let capacity = data.len();
        if capacity == 0 {
            return Err(RingBufError::CapacityZero);
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
            head: if capacity > 0 { head % capacity } else { 0 },
            len,
            capacity,
        })
    }

    /// Creates a view from raw components without checking invariants in release mode.
    ///
    /// # Invariants
    /// `capacity > 0`, `len <= capacity`, `head < capacity`.
    ///
    /// # Panics
    ///
    /// Этот метод никогда не паникует.
    #[inline(always)]
    pub(crate) fn from_raw_unchecked(data: &'a mut [T], head: usize, len: usize) -> Self {
        let capacity = data.len();
        debug_assert!(capacity > 0);
        debug_assert!(len <= capacity);
        debug_assert!(head < capacity);
        Self {
            data,
            head,
            len,
            capacity,
        }
    }

    /// Pushes an item into the ring buffer, overwriting the oldest element if full.
    ///
    /// Uses branching logic to advance head (no modulo operator).
    ///
    /// # Panics
    ///
    /// Этот метод никогда не паникует.
    #[inline(always)]
    pub fn push(&mut self, item: T) {
        self.head = math::next_head(self.head, self.capacity);
        debug_assert!(self.head < self.capacity);
        if let Some(slot) = self.data.get_mut(self.head) {
            *slot = item;
        }
        if self.len < self.capacity {
            self.len += 1;
        }
    }

    /// Clears the view state (sets length to 0).
    ///
    /// # Panics
    ///
    /// Этот метод никогда не паникует.
    #[inline(always)]
    pub fn clear(&mut self) {
        self.len = 0;
        self.head = self.capacity - 1;
    }

    /// Returns the number of elements currently in the buffer.
    ///
    /// # Panics
    ///
    /// Этот метод никогда не паникует.
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if the buffer contains no elements.
    ///
    /// # Panics
    ///
    /// Этот метод никогда не паникует.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the capacity of the buffer.
    ///
    /// # Panics
    ///
    /// Этот метод никогда не паникует.
    #[inline(always)]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Returns the current head index (physical position of the newest element).
    ///
    /// # Panics
    ///
    /// Этот метод никогда не паникует.
    #[inline(always)]
    pub fn head(&self) -> usize {
        self.head
    }

    /// Returns a reference to the element at relative index `rel` (0 = newest element).
    ///
    /// # Panics
    ///
    /// Этот метод никогда не паникует.
    #[inline(always)]
    pub fn get(&self, rel: usize) -> Option<&T> {
        if rel >= self.len {
            None
        } else {
            let phys = math::phys_index(self.head, self.capacity, rel);
            debug_assert!(phys < self.capacity);
            self.data.get(phys)
        }
    }

    /// Returns a reference to the element using a signed relative index (`rel >= 0` from head, `rel < 0` from oldest).
    ///
    /// # Panics
    ///
    /// Этот метод никогда не паникует.
    #[inline(always)]
    pub fn get_rel(&self, rel: isize) -> Option<&T> {
        let idx = math::rel_to_index(rel, self.len)?;
        self.get(idx)
    }

    /// Returns a mutable reference to the element at relative index `rel` (0 = newest element).
    ///
    /// # Panics
    ///
    /// Этот метод никогда не паникует.
    #[inline(always)]
    pub fn get_mut(&mut self, rel: usize) -> Option<&mut T> {
        if rel >= self.len {
            None
        } else {
            let phys = math::phys_index(self.head, self.capacity, rel);
            debug_assert!(phys < self.capacity);
            self.data.get_mut(phys)
        }
    }

    /// Returns a mutable reference to the element using a signed relative index (`rel >= 0` from head, `rel < 0` from oldest).
    ///
    /// # Panics
    ///
    /// Этот метод никогда не паникует.
    #[inline(always)]
    pub fn get_rel_mut(&mut self, rel: isize) -> Option<&mut T> {
        let idx = math::rel_to_index(rel, self.len)?;
        self.get_mut(idx)
    }

    /// Returns the buffer data in logical order (oldest -> newest) as up to two continuous slices.
    ///
    /// # Panics
    ///
    /// Этот метод никогда не паникует.
    #[inline(always)]
    pub fn as_slices(&self) -> (&[T], &[T]) {
        math::as_slices(self.data, self.head, self.len, self.capacity)
    }

    /// Returns the buffer data in logical order (oldest -> newest) as up to two continuous mutable slices.
    ///
    /// # Panics
    ///
    /// Этот метод никогда не паникует.
    #[inline(always)]
    pub fn as_slices_mut(&mut self) -> (&mut [T], &mut [T]) {
        math::as_slices_mut(self.data, self.head, self.len, self.capacity)
    }

    /// Returns an iterator over immutable references in logical order (oldest -> newest).
    ///
    /// # Panics
    ///
    /// Этот метод никогда не паникует.
    #[inline(always)]
    pub fn iter(&self) -> CBufIter<'_, T> {
        let (s1, s2) = self.as_slices();
        CBufIter::new(s1, s2)
    }

    /// Returns an iterator over immutable references in reverse logical order (newest -> oldest).
    ///
    /// # Panics
    ///
    /// Этот метод никогда не паникует.
    #[inline(always)]
    pub fn iter_newest_first(&self) -> Rev<CBufIter<'_, T>> {
        self.iter().rev()
    }

    /// Returns an iterator over mutable references in logical order (oldest -> newest).
    ///
    /// # Panics
    ///
    /// Этот метод никогда не паникует.
    #[inline(always)]
    pub fn iter_mut(&mut self) -> CBufIterMut<'_, T> {
        let (s1, s2) = self.as_slices_mut();
        CBufIterMut::new(s1, s2)
    }

    /// Borrows an immutable view from this mutable view.
    ///
    /// # Panics
    ///
    /// Этот метод никогда не паникует.
    #[inline(always)]
    pub fn as_view(&self) -> CBufView<'_, T> {
        CBufView {
            data: self.data,
            head: self.head,
            len: self.len,
            capacity: self.capacity,
        }
    }
}

impl<'a, 'b, T> IntoIterator for &'b CBufViewMut<'a, T> {
    type Item = &'b T;
    type IntoIter = CBufIter<'b, T>;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> IntoIterator for CBufViewMut<'a, T> {
    type Item = &'a mut T;
    type IntoIter = CBufIterMut<'a, T>;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        let (s1, s2) = math::as_slices_mut(self.data, self.head, self.len, self.capacity);
        CBufIterMut::new(s1, s2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arbitrary_capacity_push_get_slices() {
        let mut mem = [0i32; 3]; // non-power-of-two capacity
        let mut view = CBufViewMut::try_new(&mut mem).unwrap();

        assert_eq!(view.capacity(), 3);
        assert_eq!(view.len(), 0);

        view.push(100);
        assert_eq!(view.len(), 1);
        assert_eq!(view.get(0), Some(&100));

        view.push(200);
        view.push(300);
        assert_eq!(view.len(), 3);
        assert_eq!(view.get(0), Some(&300));
        assert_eq!(view.get(1), Some(&200));
        assert_eq!(view.get(2), Some(&100));
        assert_eq!(view.as_slices(), (&[100, 200, 300][..], &[][..]));

        // Overwrite
        view.push(400);
        assert_eq!(view.len(), 3);
        assert_eq!(view.get(0), Some(&400));
        assert_eq!(view.get(1), Some(&300));
        assert_eq!(view.get(2), Some(&200));

        let (s1, s2) = view.as_slices();
        assert_eq!(s1, &[200, 300]);
        assert_eq!(s2, &[400]);

        let mut collected = [0i32; 3];
        for (i, v) in view.iter().copied().enumerate() {
            collected[i] = v;
        }
        assert_eq!(collected, [200, 300, 400]);

        let mut rev_collected = [0i32; 3];
        for (i, v) in view.iter_newest_first().copied().enumerate() {
            rev_collected[i] = v;
        }
        assert_eq!(rev_collected, [400, 300, 200]);
    }

    #[test]
    fn test_empty_buffer_operations() {
        let mem = [1i32, 2, 3];
        let view = CBufView::try_new(&mem).unwrap();
        assert_eq!(view.len(), 0);
        assert!(view.is_empty());
        assert_eq!(view.get(0), None);
        assert_eq!(view.get_rel(0), None);
        assert_eq!(view.get_rel(-1), None);
        assert_eq!(view.as_slices(), (&[][..], &[][..]));
        assert_eq!(view.iter().next(), None);
        assert_eq!(view.iter_newest_first().next(), None);

        let mut mem_mut = [1i32, 2, 3];
        let mut view_mut = CBufViewMut::try_new(&mut mem_mut).unwrap();
        assert_eq!(view_mut.len(), 0);
        assert!(view_mut.is_empty());
        assert_eq!(view_mut.get(0), None);
        assert_eq!(view_mut.get_mut(0), None);
        assert_eq!(view_mut.get_rel(0), None);
        assert_eq!(view_mut.get_rel_mut(-1), None);
        assert_eq!(view_mut.as_slices(), (&[][..], &[][..]));
        assert_eq!(view_mut.as_slices_mut(), (&mut [][..], &mut [][..]));
        assert_eq!(view_mut.iter().next(), None);
        assert_eq!(view_mut.iter_mut().next(), None);
        assert_eq!(view_mut.iter_newest_first().next(), None);
    }

    #[test]
    fn test_relative_indexing() {
        let mut mem = [0i32; 3];
        let mut view = CBufViewMut::try_new(&mut mem).unwrap();

        view.push(10);
        view.push(20);
        view.push(30);

        // len = 3, newest = 30 (rel 0), 20 (rel 1), oldest = 10 (rel 2)
        assert_eq!(view.get_rel(0), Some(&30));
        assert_eq!(view.get_rel(1), Some(&20));
        assert_eq!(view.get_rel(2), Some(&10));
        assert_eq!(view.get_rel(3), None);

        // Negative indices: -1 = oldest (10), -2 = 20, -3 = newest (30), -4 = None
        assert_eq!(view.get_rel(-1), Some(&10));
        assert_eq!(view.get_rel(-2), Some(&20));
        assert_eq!(view.get_rel(-3), Some(&30));
        assert_eq!(view.get_rel(-4), None);

        *view.get_rel_mut(-1).unwrap() = 100;
        assert_eq!(view.get_rel(-1), Some(&100));
        assert_eq!(view.get_rel(2), Some(&100));
    }
}
