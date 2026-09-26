use core::iter::Rev;
use core::marker::PhantomData;
use raw_storage::traits::StorageMut;
use strided_mem::ring_view::{Ring2NView, Ring2NViewMut, RingView, RingViewMut};
use strided_mem::{StridedView, StridedViewMut};

use crate::error::{CBufResult, CBufError};
use crate::iter::{CBufIter, CBufIterMut};
use crate::math;

/// Владеющий кольцевой буфер.
///
/// Управление состоянием (head, len) хранится здесь.
/// Геометрия доступа делегируется представлениям из `strided-mem`.
#[derive(Debug)]
pub struct CBuf<T, S> {
    storage: S,
    head: usize,
    len: usize,
    capacity: usize,
    _marker: PhantomData<T>,
}

impl<T, S: StorageMut<Item = T>> CBuf<T, S> {
    /// Создаёт буфер произвольной ёмкости.
    ///
    /// # Errors
    /// `CBufError::CapacityZero` если ёмкость 0.
    pub fn try_new(mut storage: S) -> CBufResult<Self> {
        let capacity = storage.as_mut_slice().len();
        if capacity == 0 {
            return Err(CBufError::CapacityZero);
        }
        Ok(Self {
            storage,
            head: capacity - 1,
            len: 0,
            capacity,
            _marker: PhantomData,
        })
    }

    /// Создаёт буфер с ёмкостью, степенью двойки.
    ///
    /// # Errors
    /// `CBufError::CapacityZero` или `CBufError::NotPowerOfTwo`.
    pub fn try_new_2n(mut storage: S) -> CBufResult<Self> {
        let capacity = storage.as_mut_slice().len();
        if capacity == 0 {
            return Err(CBufError::CapacityZero);
        }
        if !capacity.is_power_of_two() {
            #[cfg(feature = "verbose-errors")]
            return Err(CBufError::NotPowerOfTwo { capacity });
            #[cfg(not(feature = "verbose-errors"))]
            return Err(CBufError::NotPowerOfTwo);
        }
        Ok(Self {
            storage,
            head: capacity - 1,
            len: 0,
            capacity,
            _marker: PhantomData,
        })
    }

    /// Pushes an item into the ring buffer, overwriting the oldest element if full.
    ///
    /// # Panics
    ///
    /// Этот метод никогда не паникует.
    #[inline(always)]
    pub fn push(&mut self, item: T) {
        self.head = (self.head + 1) % self.capacity;
        debug_assert!(self.head < self.capacity);
        self.storage.as_mut_slice()[self.head] = item;
        if self.len < self.capacity {
            self.len += 1;
        }
    }

    /// Clears the ring buffer state (sets length to 0).
    ///
    /// # Panics
    ///
    /// Этот метод никогда не паникует.
    #[inline(always)]
    pub fn clear(&mut self) {
        self.len = 0;
        self.head = self.capacity - 1;
    }

    /// Returns the number of elements currently stored.
    ///
    /// # Panics
    ///
    /// Этот метод никогда не паникует.
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if the ring buffer contains no elements.
    ///
    /// # Panics
    ///
    /// Этот метод никогда не паникует.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the total capacity of the ring buffer.
    ///
    /// # Panics
    ///
    /// Этот метод никогда не паникует.
    #[inline(always)]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Returns the current head index.
    ///
    /// # Panics
    ///
    /// Этот метод никогда не паникует.
    #[inline(always)]
    pub fn head(&self) -> usize {
        self.head
    }

    /// Returns a reference to the element at relative index `rel` (0 = newest).
    /// Returns `None` if `rel >= len`.
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
            self.storage.as_slice().get(phys)
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

    /// Returns a mutable reference to the element at relative index `rel` (0 = newest).
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
            self.storage.as_mut_slice().get_mut(phys)
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

    /// Доступ по относительному индексу с циклическим переносом.
    ///
    /// Возвращает `None` только если `self.len == 0`.
    #[inline(always)]
    pub fn get_wrapping(&self, rel: isize) -> Option<&T> {
        if self.len == 0 {
            return None;
        }
        let logical = math::wrap_index(rel, self.len);
        let phys = math::phys_index(self.head, self.capacity, logical);
        self.storage.as_slice().get(phys)
    }

    /// Мутабельный доступ по относительному индексу с циклическим переносом.
    ///
    /// Возвращает `None` только если `self.len == 0`.
    #[inline(always)]
    pub fn get_wrapping_mut(&mut self, rel: isize) -> Option<&mut T> {
        if self.len == 0 {
            return None;
        }
        let logical = math::wrap_index(rel, self.len);
        let phys = math::phys_index(self.head, self.capacity, logical);
        self.storage.as_mut_slice().get_mut(phys)
    }

    /// Неизменяемое кольцевое представление произвольной ёмкости.
    #[inline(always)]
    pub fn as_ring_view(&self) -> RingView<'_, T, StridedView<'_, T>> {
        let sv = StridedView::from_slice(self.storage.as_slice());
        RingView::new_unchecked(sv, self.head, self.len)
    }

    /// Мутабельное кольцевое представление произвольной ёмкости.
    #[inline(always)]
    pub fn as_ring_view_mut(&mut self) -> RingViewMut<'_, T, StridedViewMut<'_, T>> {
        let sv = StridedViewMut::from_mut_slice(self.storage.as_mut_slice());
        RingViewMut::new_unchecked(sv, self.head, self.len)
    }

    /// Неизменяемое кольцевое представление для ёмкости 2N.
    ///
    /// # Panics
    /// `debug_assert!` что `self.capacity.is_power_of_two()`.
    #[inline(always)]
    pub fn as_ring2n_view(&self) -> Ring2NView<'_, T, StridedView<'_, T>> {
        debug_assert!(self.capacity.is_power_of_two());
        let sv = StridedView::from_slice(self.storage.as_slice());
        Ring2NView::new_unchecked(sv, self.head, self.len)
    }

    /// Мутабельное кольцевое представление для ёмкости 2N.
    ///
    /// # Panics
    /// `debug_assert!` что `self.capacity.is_power_of_two()`.
    #[inline(always)]
    pub fn as_ring2n_view_mut(&mut self) -> Ring2NViewMut<'_, T, StridedViewMut<'_, T>> {
        debug_assert!(self.capacity.is_power_of_two());
        let sv = StridedViewMut::from_mut_slice(self.storage.as_mut_slice());
        Ring2NViewMut::new_unchecked(sv, self.head, self.len)
    }

    /// Returns the buffer data in logical order (oldest -> newest) as up to two continuous slices.
    ///
    /// # Panics
    ///
    /// Этот метод никогда не паникует.
    #[inline(always)]
    pub fn as_slices(&self) -> (&[T], &[T]) {
        math::as_slices(self.storage.as_slice(), self.head, self.len, self.capacity)
    }

    /// Returns the buffer data in logical order (oldest -> newest) as up to two continuous mutable slices.
    ///
    /// # Panics
    ///
    /// Этот метод никогда не паникует.
    #[inline(always)]
    pub fn as_slices_mut(&mut self) -> (&mut [T], &mut [T]) {
        math::as_slices_mut(self.storage.as_mut_slice(), self.head, self.len, self.capacity)
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

    /// Consumes `self` and returns the underlying storage `S`.
    ///
    /// # Panics
    ///
    /// Этот метод никогда не паникует.
    pub fn into_storage(self) -> S {
        self.storage
    }
}

impl<'a, T, S: StorageMut<Item = T>> IntoIterator for &'a CBuf<T, S> {
    type Item = &'a T;
    type IntoIter = CBufIter<'a, T>;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T, S: StorageMut<Item = T>> IntoIterator for &'a mut CBuf<T, S> {
    type Item = &'a mut T;
    type IntoIter = CBufIterMut<'a, T>;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use raw_storage::impls::ArrayStorage;

    #[test]
    fn test_cbuf_owning() {
        let storage = ArrayStorage::<i32, 3>::default();
        let mut buf = CBuf::try_new(storage).unwrap();

        assert_eq!(buf.capacity(), 3);
        assert_eq!(buf.len(), 0);

        buf.push(10);
        buf.push(20);
        buf.push(30);
        assert_eq!(buf.len(), 3);
        assert_eq!(buf.get(0), Some(&30));
        assert_eq!(buf.get_rel(-1), Some(&10));

        buf.push(40); // overwrites 10
        assert_eq!(buf.len(), 3);
        assert_eq!(buf.get(0), Some(&40));
        assert_eq!(buf.get_rel(-1), Some(&20));
        let (s1, s2) = buf.as_slices();
        assert_eq!(s1, &[20, 30]);
        assert_eq!(s2, &[40]);

        let mut rev = [0i32; 3];
        for (i, v) in buf.iter_newest_first().copied().enumerate() {
            rev[i] = v;
        }
        assert_eq!(rev, [40, 30, 20]);
    }

    #[test]
    fn test_cbuf_wrapping_indexing() {
        let storage = ArrayStorage::<i32, 3>::default();
        let mut buf = CBuf::try_new(storage).unwrap();

        assert_eq!(buf.get_wrapping(0), None);

        buf.push(10);
        buf.push(20);
        buf.push(30);
        buf.push(40); // len = 3, elements: 0=40, 1=30, 2=20

        assert_eq!(buf.get_wrapping(0), Some(&40));
        assert_eq!(buf.get_wrapping(3), Some(&40));
        assert_eq!(buf.get_wrapping(4), Some(&30));
        assert_eq!(buf.get_wrapping(-1), Some(&20));
        assert_eq!(buf.get_wrapping(-4), Some(&20));
    }

    #[test]
    fn test_cbuf_view_composition() {
        use strided_mem::traits::View;
        use strided_mem::{InvertView, TailView};

        let storage = ArrayStorage::<i32, 4>::default();
        let mut buf = CBuf::try_new_2n(storage).unwrap();
        buf.push(10);
        buf.push(20);
        buf.push(30);
        buf.push(40); // 0=40, 1=30, 2=20, 3=10

        let ring = buf.as_ring2n_view();
        assert_eq!(ring.get(0), Some(&40));

        let tail = TailView::try_new(ring, 3).unwrap();
        assert_eq!(tail.len(), 3);
        assert_eq!(tail.get(0), Some(&40));
        assert_eq!(tail.get(2), Some(&20));
        assert_eq!(tail.get(3), None);

        let inv_tail = InvertView::new(tail);
        assert_eq!(inv_tail.len(), 3);
        assert_eq!(inv_tail.get(0), Some(&20));
        assert_eq!(inv_tail.get(2), Some(&40));
    }
}
