use core::iter::FusedIterator;
use core::marker::PhantomData;
use core::ptr::NonNull;

use crate::pointer::offset_ptr;

/// Iterator over immutable references in a strided memory view.
#[derive(Debug, Clone)]
pub struct StridedIter<'a, T> {
    ptr: NonNull<T>,
    stride: usize,
    front_index: usize,
    len: usize,
    _marker: PhantomData<&'a T>,
}

impl<'a, T> StridedIter<'a, T> {
    #[inline(always)]
    pub(crate) fn new(ptr: NonNull<T>, stride: usize, len: usize) -> Self {
        Self {
            ptr,
            stride,
            front_index: 0,
            len,
            _marker: PhantomData,
        }
    }
}

impl<'a, T> Iterator for StridedIter<'a, T> {
    type Item = &'a T;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            None
        } else {
            let current = self.front_index;
            self.front_index += 1;
            self.len -= 1;
            unsafe {
                let elem_ptr = offset_ptr(self.ptr, self.stride, current);
                Some(&*elem_ptr.as_ptr())
            }
        }
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<'a, T> DoubleEndedIterator for StridedIter<'a, T> {
    #[inline(always)]
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            None
        } else {
            self.len -= 1;
            let target_idx = self.front_index + self.len;
            unsafe {
                let elem_ptr = offset_ptr(self.ptr, self.stride, target_idx);
                Some(&*elem_ptr.as_ptr())
            }
        }
    }
}

impl<'a, T> ExactSizeIterator for StridedIter<'a, T> {
    #[inline(always)]
    fn len(&self) -> usize {
        self.len
    }
}

impl<'a, T> FusedIterator for StridedIter<'a, T> {}

/// Iterator over mutable references in a strided memory view.
#[derive(Debug)]
pub struct StridedIterMut<'a, T> {
    ptr: NonNull<T>,
    stride: usize,
    front_index: usize,
    len: usize,
    _marker: PhantomData<&'a mut T>,
}

impl<'a, T> StridedIterMut<'a, T> {
    #[inline(always)]
    pub(crate) fn new(ptr: NonNull<T>, stride: usize, len: usize) -> Self {
        Self {
            ptr,
            stride,
            front_index: 0,
            len,
            _marker: PhantomData,
        }
    }
}

impl<'a, T> Iterator for StridedIterMut<'a, T> {
    type Item = &'a mut T;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            None
        } else {
            let current = self.front_index;
            self.front_index += 1;
            self.len -= 1;
            unsafe {
                let elem_ptr = offset_ptr(self.ptr, self.stride, current);
                Some(&mut *elem_ptr.as_ptr())
            }
        }
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<'a, T> DoubleEndedIterator for StridedIterMut<'a, T> {
    #[inline(always)]
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            None
        } else {
            self.len -= 1;
            let target_idx = self.front_index + self.len;
            unsafe {
                let elem_ptr = offset_ptr(self.ptr, self.stride, target_idx);
                Some(&mut *elem_ptr.as_ptr())
            }
        }
    }
}

impl<'a, T> ExactSizeIterator for StridedIterMut<'a, T> {
    #[inline(always)]
    fn len(&self) -> usize {
        self.len
    }
}

impl<'a, T> FusedIterator for StridedIterMut<'a, T> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strided_iter_forward_and_double_ended() {
        let mut data = [10, 20, 30, 40, 50, 60];
        let ptr = NonNull::new(data.as_mut_ptr()).unwrap();
        let mut iter = StridedIter::new(ptr, 2, 3);

        assert_eq!(iter.len(), 3);
        assert_eq!(iter.next(), Some(&10));
        assert_eq!(iter.next_back(), Some(&50));
        assert_eq!(iter.next(), Some(&30));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next_back(), None);
    }

    #[test]
    fn test_strided_iter_mut() {
        let mut data = [1, 2, 3, 4, 5, 6];
        let ptr = NonNull::new(data.as_mut_ptr()).unwrap();
        let iter_mut = StridedIterMut::new(ptr, 2, 3);

        for x in iter_mut {
            *x *= 10;
        }

        assert_eq!(data, [10, 2, 30, 4, 50, 6]);
    }
}
