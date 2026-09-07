use crate::error::{Result, RingBufError};
use crate::iter::{CBufIter, CBufIterMut};

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
            let phys = (self.head + self.capacity - rel) % self.capacity;
            Some(&self.data[phys])
        }
    }

    /// Returns the buffer data in logical order (oldest -> newest) as up to two continuous slices.
    #[inline(always)]
    pub fn as_slices(&self) -> (&'a [T], &'a [T]) {
        if self.len == 0 {
            return (&[], &[]);
        }
        let oldest = (self.head + self.capacity + 1 - self.len) % self.capacity;
        if oldest + self.len <= self.capacity {
            (&self.data[oldest..oldest + self.len], &[])
        } else {
            let first_len = self.capacity - oldest;
            let second_len = self.len - first_len;
            (&self.data[oldest..self.capacity], &self.data[..second_len])
        }
    }

    /// Returns an iterator over immutable references in logical order (oldest -> newest).
    #[inline(always)]
    pub fn iter(&self) -> CBufIter<'a, T> {
        let (s1, s2) = self.as_slices();
        CBufIter::new(s1, s2)
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

    /// Pushes an item into the ring buffer, overwriting the oldest element if full.
    ///
    /// Uses branching logic to advance head (no modulo operator).
    #[inline(always)]
    pub fn push(&mut self, item: T) {
        self.head += 1;
        if self.head == self.capacity {
            self.head = 0;
        }
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
            let phys = (self.head + self.capacity - rel) % self.capacity;
            Some(&self.data[phys])
        }
    }

    /// Returns a mutable reference to the element at relative index `rel` (0 = newest element).
    #[inline(always)]
    pub fn get_mut(&mut self, rel: usize) -> Option<&mut T> {
        if rel >= self.len {
            None
        } else {
            let phys = (self.head + self.capacity - rel) % self.capacity;
            Some(&mut self.data[phys])
        }
    }

    /// Returns the buffer data in logical order (oldest -> newest) as up to two continuous slices.
    #[inline(always)]
    pub fn as_slices(&self) -> (&[T], &[T]) {
        if self.len == 0 {
            return (&[], &[]);
        }
        let oldest = (self.head + self.capacity + 1 - self.len) % self.capacity;
        if oldest + self.len <= self.capacity {
            (&self.data[oldest..oldest + self.len], &[])
        } else {
            let first_len = self.capacity - oldest;
            let second_len = self.len - first_len;
            (&self.data[oldest..self.capacity], &self.data[..second_len])
        }
    }

    /// Returns the buffer data in logical order (oldest -> newest) as up to two continuous mutable slices.
    #[inline(always)]
    pub fn as_slices_mut(&mut self) -> (&mut [T], &mut [T]) {
        if self.len == 0 {
            return (&mut [], &mut []);
        }
        let oldest = (self.head + self.capacity + 1 - self.len) % self.capacity;
        let len = self.len;
        let capacity = self.capacity;
        if oldest + len <= capacity {
            let slice = &mut self.data[oldest..oldest + len];
            (slice, &mut [])
        } else {
            let first_len = capacity - oldest;
            let second_len = len - first_len;
            let (left, right) = self.data.split_at_mut(oldest);
            let (slice1, _) = right.split_at_mut(first_len);
            let (slice2, _) = left.split_at_mut(second_len);
            (slice1, slice2)
        }
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

    /// Borrows an immutable view from this mutable view.
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
        let capacity = self.capacity;
        let head = self.head;
        let len = self.len;
        if len == 0 {
            return CBufIterMut::new(&mut [], &mut []);
        }
        let oldest = (head + capacity + 1 - len) % capacity;
        if oldest + len <= capacity {
            let slice = &mut self.data[oldest..oldest + len];
            CBufIterMut::new(slice, &mut [])
        } else {
            let first_len = capacity - oldest;
            let second_len = len - first_len;
            let (left, right) = self.data.split_at_mut(oldest);
            let (slice1, _) = right.split_at_mut(first_len);
            let (slice2, _) = left.split_at_mut(second_len);
            CBufIterMut::new(slice1, slice2)
        }
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
    }
}
