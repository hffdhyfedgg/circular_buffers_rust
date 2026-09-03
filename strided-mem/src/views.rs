use core::marker::PhantomData;
use core::ptr::NonNull;
use raw_storage::{Storage, StorageMut};

use crate::pointer::offset_ptr;

/// An immutable view over elements spaced by a constant stride.
#[derive(Debug)]
pub struct StridedView<'a, T> {
    ptr: NonNull<T>,
    stride: usize,
    len: usize,
    _marker: PhantomData<&'a T>,
}

/// A mutable view over elements spaced by a constant stride.
#[derive(Debug)]
pub struct StridedViewMut<'a, T> {
    ptr: NonNull<T>,
    stride: usize,
    len: usize,
    _marker: PhantomData<&'a mut T>,
}

impl<'a, T> StridedView<'a, T> {
    /// Creates a new [`StridedView`] without bounds or aliasing checks.
    ///
    /// # Safety
    ///
    /// The caller must guarantee that:
    /// 1. `ptr` is valid for reads for `len` elements spaced by `stride`.
    /// 2. Memory range `[ptr + i * stride]` for `0 <= i < len` does not alias any active mutable references for lifetime `'a`.
    /// 3. `stride` is greater than zero if `len > 1`.
    #[inline(always)]
    pub unsafe fn new_unchecked(ptr: NonNull<T>, stride: usize, len: usize) -> Self {
        Self {
            ptr,
            stride,
            len,
            _marker: PhantomData,
        }
    }

    /// Creates a [`StridedView`] with `stride = 1` from a contiguous immutable slice.
    #[inline]
    pub fn from_slice(slice: &'a [T]) -> Self {
        let len = slice.len();
        let ptr = NonNull::new(slice.as_ptr() as *mut T).unwrap_or_else(NonNull::dangling);
        // SAFETY: slice.as_ptr() is valid for slice.len() elements with stride 1.
        unsafe { Self::new_unchecked(ptr, 1, len) }
    }

    /// Returns the number of elements accessible in this view.
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if the view contains no elements.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the stride (step size) between elements.
    #[inline(always)]
    pub fn stride(&self) -> usize {
        self.stride
    }

    /// Returns the raw pointer to the base element of the view.
    #[inline(always)]
    pub fn as_ptr(&self) -> *const T {
        self.ptr.as_ptr()
    }

    /// Returns an immutable reference to the element at `index`, or `None` if out of bounds.
    #[inline(always)]
    pub fn get(&self, index: usize) -> Option<&'a T> {
        if index >= self.len {
            return None;
        }
        // SAFETY: index < self.len, and self.ptr + index * stride is valid and non-aliasing for 'a.
        unsafe {
            let elem_ptr = offset_ptr(self.ptr, self.stride, index);
            Some(&*elem_ptr.as_ptr())
        }
    }
}

impl<'a, T> StridedViewMut<'a, T> {
    /// Creates a new [`StridedViewMut`] without bounds or aliasing checks.
    ///
    /// # Safety
    ///
    /// The caller must guarantee that:
    /// 1. `ptr` is valid for reads and writes for `len` elements spaced by `stride`.
    /// 2. Memory range `[ptr + i * stride]` for `0 <= i < len` does not alias any other active references for lifetime `'a`.
    /// 3. `stride` is greater than zero if `len > 1`.
    #[inline(always)]
    pub unsafe fn new_unchecked(ptr: NonNull<T>, stride: usize, len: usize) -> Self {
        Self {
            ptr,
            stride,
            len,
            _marker: PhantomData,
        }
    }

    /// Creates a [`StridedViewMut`] with `stride = 1` from a contiguous mutable slice.
    #[inline]
    pub fn from_mut_slice(slice: &'a mut [T]) -> Self {
        let len = slice.len();
        let ptr = NonNull::new(slice.as_mut_ptr()).unwrap_or_else(NonNull::dangling);
        // SAFETY: slice.as_mut_ptr() is valid for slice.len() elements with stride 1.
        unsafe { Self::new_unchecked(ptr, 1, len) }
    }

    /// Returns the number of elements accessible in this view.
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if the view contains no elements.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the stride (step size) between elements.
    #[inline(always)]
    pub fn stride(&self) -> usize {
        self.stride
    }

    /// Returns the raw pointer to the base element of the view.
    #[inline(always)]
    pub fn as_ptr(&self) -> *const T {
        self.ptr.as_ptr()
    }

    /// Returns the raw mutable pointer to the base element of the view.
    #[inline(always)]
    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.ptr.as_ptr()
    }

    /// Returns an immutable reference to the element at `index`, or `None` if out of bounds.
    #[inline(always)]
    pub fn get(&self, index: usize) -> Option<&T> {
        if index >= self.len {
            return None;
        }
        // SAFETY: index < self.len, and self.ptr + index * stride is valid.
        unsafe {
            let elem_ptr = offset_ptr(self.ptr, self.stride, index);
            Some(&*elem_ptr.as_ptr())
        }
    }

    /// Returns a mutable reference to the element at `index`, or `None` if out of bounds.
    #[inline(always)]
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index >= self.len {
            return None;
        }
        // SAFETY: index < self.len, and self.ptr + index * stride is valid and unaliased.
        unsafe {
            let elem_ptr = offset_ptr(self.ptr, self.stride, index);
            Some(&mut *elem_ptr.as_ptr())
        }
    }

    /// Reborrows `self` as an immutable [`StridedView`].
    #[inline(always)]
    pub fn as_view(&self) -> StridedView<'_, T> {
        // SAFETY: self.ptr is valid for self.len elements with self.stride, and &self guarantees no concurrent mutability.
        unsafe { StridedView::new_unchecked(self.ptr, self.stride, self.len) }
    }

    /// Reborrows `self` as a shorter-lived mutable [`StridedViewMut`].
    #[inline(always)]
    pub fn reborrow(&mut self) -> StridedViewMut<'_, T> {
        // SAFETY: self.ptr is valid for self.len elements with self.stride, and &mut self guarantees exclusive access.
        unsafe { StridedViewMut::new_unchecked(self.ptr, self.stride, self.len) }
    }
}

// SAFETY: StridedView provides shared immutable access to T.
// If T: Sync, shared references &T can be safely shared across threads.
unsafe impl<'a, T: Sync> Send for StridedView<'a, T> {}
unsafe impl<'a, T: Sync> Sync for StridedView<'a, T> {}

// SAFETY: StridedViewMut provides exclusive mutable access to T.
// If T: Send, ownership/exclusive access can be transferred across threads.
unsafe impl<'a, T: Send> Send for StridedViewMut<'a, T> {}
// If T: Sync, shared references &StridedViewMut can be safely accessed across threads.
unsafe impl<'a, T: Sync> Sync for StridedViewMut<'a, T> {}

impl<'a, S: Storage> From<&'a S> for StridedView<'a, S::Item> {
    #[inline]
    fn from(storage: &'a S) -> Self {
        StridedView::from_slice(storage.as_slice())
    }
}

impl<'a, S: StorageMut> From<&'a mut S> for StridedViewMut<'a, S::Item> {
    #[inline]
    fn from(storage: &'a mut S) -> Self {
        StridedViewMut::from_mut_slice(storage.as_mut_slice())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use raw_storage::{ArrayStorage, SliceStorage};
    #[cfg(feature = "alloc")]
    use raw_storage::AllocStorage;
    #[cfg(feature = "alloc")]
    extern crate alloc;

    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}

    #[test]
    fn test_send_sync_bounds() {
        assert_send::<StridedView<'_, i32>>();
        assert_sync::<StridedView<'_, i32>>();
        assert_send::<StridedViewMut<'_, i32>>();
        assert_sync::<StridedViewMut<'_, i32>>();
    }

    #[test]
    fn test_strided_view_from_slice() {
        let data = [10, 20, 30, 40];
        let view = StridedView::from_slice(&data);
        assert_eq!(view.len(), 4);
        assert_eq!(view.stride(), 1);
        assert!(!view.is_empty());

        assert_eq!(view.get(0), Some(&10));
        assert_eq!(view.get(3), Some(&40));
        assert_eq!(view.get(4), None);
    }

    #[test]
    fn test_strided_view_mut_from_slice_and_mutation() {
        let mut data = [10, 20, 30, 40];
        {
            let mut view = StridedViewMut::from_mut_slice(&mut data);
            assert_eq!(view.len(), 4);
            assert_eq!(view.get(1), Some(&20));

            if let Some(val) = view.get_mut(1) {
                *val = 99;
            }
            assert_eq!(view.get(1), Some(&99));
            assert_eq!(view.get_mut(10), None);
        }
        assert_eq!(data, [10, 99, 30, 40]);
    }

    #[test]
    fn test_strided_view_from_raw_storage() {
        let array_storage = ArrayStorage::new([1, 2, 3]);
        let view = StridedView::from(&array_storage);
        assert_eq!(view.len(), 3);
        assert_eq!(view.get(0), Some(&1));

        let mut slice_buf = [4, 5, 6, 7];
        let mut slice_storage = SliceStorage::new(&mut slice_buf);
        {
            let mut mut_view = StridedViewMut::from(&mut slice_storage);
            if let Some(v) = mut_view.get_mut(0) {
                *v = 40;
            }
        }
        assert_eq!(slice_buf[0], 40);

        #[cfg(feature = "alloc")]
        {
            let mut alloc_storage = AllocStorage::from_vec(alloc::vec![100, 200]);
            let mut_view = StridedViewMut::from(&mut alloc_storage);
            assert_eq!(mut_view.get(1), Some(&200));
        }
    }

    #[test]
    fn test_reborrow_and_as_view() {
        let mut data = [1, 2, 3];
        let mut view_mut = StridedViewMut::from_mut_slice(&mut data);

        let view_immut = view_mut.as_view();
        assert_eq!(view_immut.get(0), Some(&1));

        let mut reborrowed = view_mut.reborrow();
        if let Some(v) = reborrowed.get_mut(0) {
            *v = 10;
        }
        assert_eq!(view_mut.get(0), Some(&10));
    }
}
