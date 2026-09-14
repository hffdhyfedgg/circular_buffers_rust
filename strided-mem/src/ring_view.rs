use core::marker::PhantomData;

use crate::error::{Result, StridedError};
use crate::traits::{ContiguousView, ContiguousViewMut, View, ViewMut};
use crate::views::{StridedView, StridedViewMut};

#[inline(always)]
fn out_of_bounds_err(_index: usize, _len: usize) -> StridedError {
    #[cfg(feature = "verbose-errors")]
    {
        StridedError::OutOfBounds {
            index: _index,
            len: _len,
        }
    }
    #[cfg(not(feature = "verbose-errors"))]
    {
        StridedError::OutOfBounds
    }
}

/// Кольцевое представление над внутренним представлением `V`.
///
/// Логический индекс 0 — самый НОВЫЙ элемент,
/// логический индекс `len - 1` — самый СТАРЫЙ.
pub struct RingView<'a, T, V: View<'a, Item = T>> {
    inner: V,
    head: usize,
    len: usize,
    capacity: usize,
    _marker: PhantomData<&'a T>,
}

impl<'a, T, V: View<'a, Item = T>> RingView<'a, T, V> {
    /// Проверяемый конструктор.
    ///
    /// # Errors
    /// `StridedError::OutOfBounds` если `capacity == 0`,
    /// `head >= capacity`, или `len > capacity`.
    pub fn try_new(inner: V, head: usize, len: usize) -> Result<Self> {
        let capacity = inner.len();
        if capacity == 0 || head >= capacity || len > capacity {
            return Err(out_of_bounds_err(head, capacity));
        }
        Ok(Self {
            inner,
            head,
            len,
            capacity,
            _marker: PhantomData,
        })
    }

    /// Конструктор без проверок в релизе.
    #[inline(always)]
    pub fn new_unchecked(inner: V, head: usize, len: usize) -> Self {
        let capacity = inner.len();
        debug_assert!(capacity > 0);
        debug_assert!(head < capacity);
        debug_assert!(len <= capacity);
        Self {
            inner,
            head,
            len,
            capacity,
            _marker: PhantomData,
        }
    }

    #[inline(always)]
    pub fn head(&self) -> usize {
        self.head
    }

    #[inline(always)]
    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

impl<'a, T, V: View<'a, Item = T>> View<'a> for RingView<'a, T, V> {
    type Item = T;

    #[inline(always)]
    fn len(&self) -> usize {
        self.len
    }

    #[inline(always)]
    fn get(&self, index: usize) -> Option<&'a T> {
        if index >= self.len {
            return None;
        }
        let phys = (self.head + self.capacity - index) % self.capacity;
        self.inner.get(phys)
    }
}

/// Кольцевое представление для ёмкости 2N (степень двойки).
pub struct Ring2NView<'a, T, V: View<'a, Item = T>> {
    inner: V,
    head: usize,
    len: usize,
    mask: usize,
    capacity: usize,
    _marker: PhantomData<&'a T>,
}

impl<'a, T, V: View<'a, Item = T>> Ring2NView<'a, T, V> {
    /// Проверяемый конструктор.
    ///
    /// # Errors
    /// `StridedError::OutOfBounds` если `capacity == 0`, `!capacity.is_power_of_two()`,
    /// `head >= capacity`, или `len > capacity`.
    pub fn try_new(inner: V, head: usize, len: usize) -> Result<Self> {
        let capacity = inner.len();
        if capacity == 0 || !capacity.is_power_of_two() || head >= capacity || len > capacity {
            return Err(out_of_bounds_err(head, capacity));
        }
        let mask = capacity - 1;
        Ok(Self {
            inner,
            head,
            len,
            mask,
            capacity,
            _marker: PhantomData,
        })
    }

    /// Конструктор без проверок в релизе.
    #[inline(always)]
    pub fn new_unchecked(inner: V, head: usize, len: usize) -> Self {
        let capacity = inner.len();
        debug_assert!(capacity > 0 && capacity.is_power_of_two());
        debug_assert!(head < capacity);
        debug_assert!(len <= capacity);
        let mask = capacity - 1;
        Self {
            inner,
            head,
            len,
            mask,
            capacity,
            _marker: PhantomData,
        }
    }

    #[inline(always)]
    pub fn head(&self) -> usize {
        self.head
    }

    #[inline(always)]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    #[inline(always)]
    pub fn mask(&self) -> usize {
        self.mask
    }
}

impl<'a, T, V: View<'a, Item = T>> View<'a> for Ring2NView<'a, T, V> {
    type Item = T;

    #[inline(always)]
    fn len(&self) -> usize {
        self.len
    }

    #[inline(always)]
    fn get(&self, index: usize) -> Option<&'a T> {
        if index >= self.len {
            return None;
        }
        let phys = (self.head + self.capacity - index) & self.mask;
        self.inner.get(phys)
    }
}

/// Мутабельное кольцевое представление над внутренним представлением `V`.
pub struct RingViewMut<'a, T, V: ViewMut<'a, Item = T>> {
    inner: V,
    head: usize,
    len: usize,
    capacity: usize,
    _marker: PhantomData<&'a mut T>,
}

impl<'a, T, V: ViewMut<'a, Item = T>> RingViewMut<'a, T, V> {
    pub fn try_new(inner: V, head: usize, len: usize) -> Result<Self> {
        let capacity = inner.len();
        if capacity == 0 || head >= capacity || len > capacity {
            return Err(out_of_bounds_err(head, capacity));
        }
        Ok(Self {
            inner,
            head,
            len,
            capacity,
            _marker: PhantomData,
        })
    }

    #[inline(always)]
    pub fn new_unchecked(inner: V, head: usize, len: usize) -> Self {
        let capacity = inner.len();
        debug_assert!(capacity > 0);
        debug_assert!(head < capacity);
        debug_assert!(len <= capacity);
        Self {
            inner,
            head,
            len,
            capacity,
            _marker: PhantomData,
        }
    }

    #[inline(always)]
    pub fn head(&self) -> usize {
        self.head
    }

    #[inline(always)]
    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

impl<'a, T, V: ViewMut<'a, Item = T>> View<'a> for RingViewMut<'a, T, V> {
    type Item = T;

    #[inline(always)]
    fn len(&self) -> usize {
        self.len
    }

    #[inline(always)]
    fn get(&self, index: usize) -> Option<&'a T> {
        if index >= self.len {
            return None;
        }
        let phys = (self.head + self.capacity - index) % self.capacity;
        self.inner.get(phys)
    }
}

impl<'a, T, V: ViewMut<'a, Item = T>> ViewMut<'a> for RingViewMut<'a, T, V> {
    #[inline(always)]
    fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index >= self.len {
            return None;
        }
        let phys = (self.head + self.capacity - index) % self.capacity;
        self.inner.get_mut(phys)
    }
}

/// Мутабельное кольцевое представление для ёмкости 2N (степень двойки).
pub struct Ring2NViewMut<'a, T, V: ViewMut<'a, Item = T>> {
    inner: V,
    head: usize,
    len: usize,
    mask: usize,
    capacity: usize,
    _marker: PhantomData<&'a mut T>,
}

impl<'a, T, V: ViewMut<'a, Item = T>> Ring2NViewMut<'a, T, V> {
    pub fn try_new(inner: V, head: usize, len: usize) -> Result<Self> {
        let capacity = inner.len();
        if capacity == 0 || !capacity.is_power_of_two() || head >= capacity || len > capacity {
            return Err(out_of_bounds_err(head, capacity));
        }
        let mask = capacity - 1;
        Ok(Self {
            inner,
            head,
            len,
            mask,
            capacity,
            _marker: PhantomData,
        })
    }

    #[inline(always)]
    pub fn new_unchecked(inner: V, head: usize, len: usize) -> Self {
        let capacity = inner.len();
        debug_assert!(capacity > 0 && capacity.is_power_of_two());
        debug_assert!(head < capacity);
        debug_assert!(len <= capacity);
        let mask = capacity - 1;
        Self {
            inner,
            head,
            len,
            mask,
            capacity,
            _marker: PhantomData,
        }
    }

    #[inline(always)]
    pub fn head(&self) -> usize {
        self.head
    }

    #[inline(always)]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    #[inline(always)]
    pub fn mask(&self) -> usize {
        self.mask
    }
}

impl<'a, T, V: ViewMut<'a, Item = T>> View<'a> for Ring2NViewMut<'a, T, V> {
    type Item = T;

    #[inline(always)]
    fn len(&self) -> usize {
        self.len
    }

    #[inline(always)]
    fn get(&self, index: usize) -> Option<&'a T> {
        if index >= self.len {
            return None;
        }
        let phys = (self.head + self.capacity - index) & self.mask;
        self.inner.get(phys)
    }
}

impl<'a, T, V: ViewMut<'a, Item = T>> ViewMut<'a> for Ring2NViewMut<'a, T, V> {
    #[inline(always)]
    fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index >= self.len {
            return None;
        }
        let phys = (self.head + self.capacity - index) & self.mask;
        self.inner.get_mut(phys)
    }
}

impl<'a, T> ContiguousView<'a> for RingView<'a, T, StridedView<'a, T>> {
    fn as_slices(&self) -> (&'a [T], &'a [T]) {
        if self.len == 0 {
            return (&[], &[]);
        }
        let ptr = self.inner.as_ptr();
        let full_slice = unsafe { core::slice::from_raw_parts(ptr, self.capacity) };
        let oldest = (self.head + self.capacity + 1 - self.len) % self.capacity;
        if oldest + self.len <= self.capacity {
            (&full_slice[oldest..oldest + self.len], &[])
        } else {
            let first_len = self.capacity - oldest;
            let second_len = self.len - first_len;
            (&full_slice[oldest..self.capacity], &full_slice[..second_len])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ring_view_and_2n_view() {
        let data = [10, 20, 30, 40];
        let sv = StridedView::from_slice(&data);

        // head = 1 (item 20 is newest), len = 3. Elements: index 0 (newest=20), index 1 (10), index 2 (40).
        let ring = RingView::try_new(sv, 1, 3).unwrap();
        assert_eq!(ring.len(), 3);
        assert_eq!(ring.get(0), Some(&20));
        assert_eq!(ring.get(1), Some(&10));
        assert_eq!(ring.get(2), Some(&40));
        assert_eq!(ring.get(3), None);

        let (s1, s2) = ring.as_slices();
        // oldest is index 3 (value 40). s1: [40], s2: [10, 20]
        assert_eq!(s1, &[40]);
        assert_eq!(s2, &[10, 20]);

        // Power of 2 view
        let ring2n = Ring2NView::try_new(sv, 1, 3).unwrap();
        assert_eq!(ring2n.len(), 3);
        assert_eq!(ring2n.get(0), Some(&20));
        assert_eq!(ring2n.get(1), Some(&10));
        assert_eq!(ring2n.get(2), Some(&40));
        assert_eq!(ring2n.get(3), None);

        let (s2n1, s2n2) = ring2n.as_slices();
        assert_eq!(s2n1, &[40]);
        assert_eq!(s2n2, &[10, 20]);
    }

    #[test]
    fn test_ring_view_mut() {
        let mut data = [10, 20, 30, 40];
        let sv = StridedViewMut::from_mut_slice(&mut data);

        let mut ring_mut = RingViewMut::try_new(sv, 1, 3).unwrap();
        assert_eq!(ring_mut.get(0), Some(&20));
        if let Some(val) = ring_mut.get_mut(0) {
            *val = 200;
        }
        assert_eq!(ring_mut.get(0), Some(&200));
    }
}

impl<'a, T> ContiguousView<'a> for Ring2NView<'a, T, StridedView<'a, T>> {
    fn as_slices(&self) -> (&'a [T], &'a [T]) {
        if self.len == 0 {
            return (&[], &[]);
        }
        let ptr = self.inner.as_ptr();
        let full_slice = unsafe { core::slice::from_raw_parts(ptr, self.capacity) };
        let oldest = (self.head + self.capacity + 1 - self.len) & self.mask;
        if oldest + self.len <= self.capacity {
            (&full_slice[oldest..oldest + self.len], &[])
        } else {
            let first_len = self.capacity - oldest;
            let second_len = self.len - first_len;
            (&full_slice[oldest..self.capacity], &full_slice[..second_len])
        }
    }
}

impl<'a, T> ContiguousViewMut<'a> for RingViewMut<'a, T, StridedViewMut<'a, T>> {
    fn as_slices_mut(&mut self) -> (&'a mut [T], &'a mut [T]) {
        if self.len == 0 {
            return (&mut [], &mut []);
        }
        let ptr = self.inner.as_mut_ptr();
        let full_slice = unsafe { core::slice::from_raw_parts_mut(ptr, self.capacity) };
        let oldest = (self.head + self.capacity + 1 - self.len) % self.capacity;
        if oldest + self.len <= self.capacity {
            let (left, _) = full_slice.split_at_mut(oldest + self.len);
            (&mut left[oldest..], &mut [])
        } else {
            let first_len = self.capacity - oldest;
            let second_len = self.len - first_len;
            let (left, right) = full_slice.split_at_mut(oldest);
            (right, &mut left[..second_len])
        }
    }
}

impl<'a, T> ContiguousViewMut<'a> for Ring2NViewMut<'a, T, StridedViewMut<'a, T>> {
    fn as_slices_mut(&mut self) -> (&'a mut [T], &'a mut [T]) {
        if self.len == 0 {
            return (&mut [], &mut []);
        }
        let ptr = self.inner.as_mut_ptr();
        let full_slice = unsafe { core::slice::from_raw_parts_mut(ptr, self.capacity) };
        let oldest = (self.head + self.capacity + 1 - self.len) & self.mask;
        if oldest + self.len <= self.capacity {
            let (left, _) = full_slice.split_at_mut(oldest + self.len);
            (&mut left[oldest..], &mut [])
        } else {
            let first_len = self.capacity - oldest;
            let second_len = self.len - first_len;
            let (left, right) = full_slice.split_at_mut(oldest);
            (right, &mut left[..second_len])
        }
    }
}
