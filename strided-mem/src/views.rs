use core::marker::PhantomData;
use core::ops::{Index, IndexMut};
use core::ptr::NonNull;

use crate::iter::{StridedIter, StridedIterMut};
use crate::pointer::offset_ptr;

/// Immutable view over memory elements with a fixed stride.
#[derive(Debug)]
pub struct StridedView<'a, T> {
    ptr: NonNull<T>,
    stride: usize,
    len: usize,
    _marker: PhantomData<&'a T>,
}

impl<'a, T> Clone for StridedView<'a, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, T> Copy for StridedView<'a, T> {}

unsafe impl<'a, T: Sync> Sync for StridedView<'a, T> {}
unsafe impl<'a, T: Sync> Send for StridedView<'a, T> {}

impl<'a, T> StridedView<'a, T> {
    /// Creates a new immutable strided view from a raw pointer, stride, and length.
    ///
    /// # Safety
    ///
    /// - `ptr` must be non-null, properly aligned for `T`, and valid for reads of `len` strided elements.
    /// - The memory range covered by `[ptr + i * stride]` for `0 <= i < len` must not be mutated by
    ///   any other thread or reference for the duration of `'a`.
    /// - `stride * len` in bytes must not exceed `isize::MAX`.
    #[inline(always)]
    pub(crate) unsafe fn new_unchecked(ptr: NonNull<T>, stride: usize, len: usize) -> Self {
        Self {
            ptr,
            stride,
            len,
            _marker: PhantomData,
        }
    }

    /// Returns the number of elements in the strided view.
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if the view contains no elements.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the step (stride) between consecutive elements.
    #[inline(always)]
    pub fn stride(&self) -> usize {
        self.stride
    }

    /// Returns a reference to an element at `index`, or `None` if out of bounds.
    #[inline(always)]
    pub fn get(&self, index: usize) -> Option<&'a T> {
        if index >= self.len {
            None
        } else {
            unsafe {
                let elem_ptr = offset_ptr(self.ptr, index, self.stride);
                Some(&*elem_ptr)
            }
        }
    }

    /// Returns an iterator over immutable references to the strided elements.
    #[inline(always)]
    pub fn iter(&self) -> StridedIter<'a, T> {
        StridedIter::new(self.ptr, self.stride, self.len)
    }
}

impl<'a, T> Index<usize> for StridedView<'a, T> {
    type Output = T;

    #[inline(always)]
    fn index(&self, index: usize) -> &Self::Output {
        if index >= self.len {
            panic!("index {} out of bounds for StridedView of length {}", index, self.len);
        }
        unsafe {
            let elem_ptr = offset_ptr(self.ptr, index, self.stride);
            &*elem_ptr
        }
    }
}

impl<'a, T> IntoIterator for StridedView<'a, T> {
    type Item = &'a T;
    type IntoIter = StridedIter<'a, T>;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// Mutable view over memory elements with a fixed stride.
#[derive(Debug)]
pub struct StridedViewMut<'a, T> {
    ptr: NonNull<T>,
    stride: usize,
    len: usize,
    _marker: PhantomData<&'a mut T>,
}

unsafe impl<'a, T: Send> Send for StridedViewMut<'a, T> {}
unsafe impl<'a, T: Sync> Sync for StridedViewMut<'a, T> {}

impl<'a, T> StridedViewMut<'a, T> {
    /// Creates a new mutable strided view from a raw pointer, stride, and length.
    ///
    /// # Safety
    ///
    /// - `ptr` must be non-null, properly aligned for `T`, and valid for reads and writes of `len` strided elements.
    /// - The memory locations `[ptr + i * stride]` for `0 <= i < len` must be disjoint from any other active references.
    /// - `stride * len` in bytes must not exceed `isize::MAX`.
    #[inline(always)]
    pub(crate) unsafe fn new_unchecked(ptr: NonNull<T>, stride: usize, len: usize) -> Self {
        Self {
            ptr,
            stride,
            len,
            _marker: PhantomData,
        }
    }

    /// Returns the number of elements in the strided view.
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if the view contains no elements.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the step (stride) between consecutive elements.
    #[inline(always)]
    pub fn stride(&self) -> usize {
        self.stride
    }

    /// Returns an immutable reference to an element at `index`, or `None` if out of bounds.
    #[inline(always)]
    pub fn get(&self, index: usize) -> Option<&T> {
        if index >= self.len {
            None
        } else {
            unsafe {
                let elem_ptr = offset_ptr(self.ptr, index, self.stride);
                Some(&*elem_ptr)
            }
        }
    }

    /// Returns a mutable reference to an element at `index`, or `None` if out of bounds.
    #[inline(always)]
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index >= self.len {
            None
        } else {
            unsafe {
                let elem_ptr = offset_ptr(self.ptr, index, self.stride);
                Some(&mut *elem_ptr)
            }
        }
    }

    /// Converts this mutable view into an immutable view.
    #[inline(always)]
    pub fn as_view(&self) -> StridedView<'_, T> {
        unsafe { StridedView::new_unchecked(self.ptr, self.stride, self.len) }
    }

    /// Returns an iterator over immutable references to the strided elements.
    #[inline(always)]
    pub fn iter(&self) -> StridedIter<'_, T> {
        StridedIter::new(self.ptr, self.stride, self.len)
    }

    /// Returns an iterator over mutable references to the strided elements.
    #[inline(always)]
    pub fn iter_mut(&mut self) -> StridedIterMut<'_, T> {
        StridedIterMut::new(self.ptr, self.stride, self.len)
    }

    /// Consumes the view and returns an iterator over mutable references with lifetime `'a`.
    #[inline(always)]
    pub fn into_iter_mut(self) -> StridedIterMut<'a, T> {
        StridedIterMut::new(self.ptr, self.stride, self.len)
    }

    /// Internal accessor for base pointer.
    #[inline(always)]
    pub(crate) fn as_ptr_non_null(&self) -> NonNull<T> {
        self.ptr
    }
}

impl<'a, T> Index<usize> for StridedViewMut<'a, T> {
    type Output = T;

    #[inline(always)]
    fn index(&self, index: usize) -> &Self::Output {
        if index >= self.len {
            panic!("index {} out of bounds for StridedViewMut of length {}", index, self.len);
        }
        unsafe {
            let elem_ptr = offset_ptr(self.ptr, index, self.stride);
            &*elem_ptr
        }
    }
}

impl<'a, T> IndexMut<usize> for StridedViewMut<'a, T> {
    #[inline(always)]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        if index >= self.len {
            panic!("index {} out of bounds for StridedViewMut of length {}", index, self.len);
        }
        unsafe {
            let elem_ptr = offset_ptr(self.ptr, index, self.stride);
            &mut *elem_ptr
        }
    }
}

impl<'a, T> IntoIterator for StridedViewMut<'a, T> {
    type Item = &'a mut T;
    type IntoIter = StridedIterMut<'a, T>;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.into_iter_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strided_view_copy_and_index() {
        let mut data = [10, 20, 30, 40, 50];
        let ptr = NonNull::new(data.as_mut_ptr()).unwrap();
        let view = unsafe { StridedView::new_unchecked(ptr, 2, 3) };

        assert_eq!(view.len(), 3);
        assert_eq!(view.stride(), 2);
        assert_eq!(view[0], 10);
        assert_eq!(view[1], 30);
        assert_eq!(view[2], 50);

        let view_copy = view;
        assert_eq!(view_copy[1], 30);
    }

    #[test]
    #[should_panic(expected = "out of bounds")]
    fn test_strided_view_index_panic() {
        let mut data = [1, 2, 3];
        let ptr = NonNull::new(data.as_mut_ptr()).unwrap();
        let view = unsafe { StridedView::new_unchecked(ptr, 1, 3) };
        let _ = view[3];
    }

    #[test]
    fn test_strided_view_mut_index_and_mutation() {
        let mut data = [10, 20, 30, 40, 50];
        let ptr = NonNull::new(data.as_mut_ptr()).unwrap();
        let mut view_mut = unsafe { StridedViewMut::new_unchecked(ptr, 2, 3) };

        view_mut[1] = 99;
        assert_eq!(data, [10, 20, 99, 40, 50]);

        assert_eq!(view_mut.get(0), Some(&10));
        assert_eq!(view_mut.get(1), Some(&99));
        assert_eq!(view_mut.get(3), None);
    }
}
