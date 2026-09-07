use core::iter::FusedIterator;
use core::marker::PhantomData;
use core::ptr::NonNull;

use crate::error::{Result, StridedError};
use crate::pointer::offset_ptr;
use crate::views::{StridedView, StridedViewMut};

/// A lazy iterator over dynamic strided splits.
#[derive(Debug, Clone)]
pub struct StridedSplitIter<'a, T> {
    ptr: NonNull<T>,
    stride: usize,
    len: usize,
    n: usize,
    front: usize,
    back: usize,
    _marker: PhantomData<&'a T>,
}

impl<'a, T> StridedSplitIter<'a, T> {
    #[inline]
    pub(crate) fn new(ptr: NonNull<T>, stride: usize, len: usize, n: usize) -> Self {
        Self {
            ptr,
            stride,
            len,
            n,
            front: 0,
            back: n,
            _marker: PhantomData,
        }
    }
}

impl<'a, T> Iterator for StridedSplitIter<'a, T> {
    type Item = StridedView<'a, T>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.front >= self.back {
            None
        } else {
            let i = self.front;
            self.front += 1;
            let new_stride = self.stride * self.n;
            let (sub_ptr, sub_len) = if i < self.len {
                let sub_len = (self.len - i).div_ceil(self.n);
                let sub_ptr = unsafe { offset_ptr(self.ptr, self.stride, i) };
                (sub_ptr, sub_len)
            } else {
                (self.ptr, 0)
            };
            unsafe { Some(StridedView::new_unchecked(sub_ptr, new_stride, sub_len)) }
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let rem = self.back.saturating_sub(self.front);
        (rem, Some(rem))
    }
}

impl<'a, T> DoubleEndedIterator for StridedSplitIter<'a, T> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.front >= self.back {
            None
        } else {
            self.back -= 1;
            let i = self.back;
            let new_stride = self.stride * self.n;
            let (sub_ptr, sub_len) = if i < self.len {
                let sub_len = (self.len - i).div_ceil(self.n);
                let sub_ptr = unsafe { offset_ptr(self.ptr, self.stride, i) };
                (sub_ptr, sub_len)
            } else {
                (self.ptr, 0)
            };
            unsafe { Some(StridedView::new_unchecked(sub_ptr, new_stride, sub_len)) }
        }
    }
}

impl<'a, T> ExactSizeIterator for StridedSplitIter<'a, T> {
    #[inline]
    fn len(&self) -> usize {
        self.back.saturating_sub(self.front)
    }
}

impl<'a, T> FusedIterator for StridedSplitIter<'a, T> {}

/// A lazy iterator over dynamic mutable strided splits.
#[derive(Debug)]
pub struct StridedSplitIterMut<'a, T> {
    ptr: NonNull<T>,
    stride: usize,
    len: usize,
    n: usize,
    front: usize,
    back: usize,
    _marker: PhantomData<&'a mut T>,
}

impl<'a, T> StridedSplitIterMut<'a, T> {
    #[inline]
    pub(crate) fn new(ptr: NonNull<T>, stride: usize, len: usize, n: usize) -> Self {
        Self {
            ptr,
            stride,
            len,
            n,
            front: 0,
            back: n,
            _marker: PhantomData,
        }
    }
}

impl<'a, T> Iterator for StridedSplitIterMut<'a, T> {
    type Item = StridedViewMut<'a, T>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.front >= self.back {
            None
        } else {
            let i = self.front;
            self.front += 1;
            let new_stride = self.stride * self.n;
            let (sub_ptr, sub_len) = if i < self.len {
                let sub_len = (self.len - i).div_ceil(self.n);
                let sub_ptr = unsafe { offset_ptr(self.ptr, self.stride, i) };
                (sub_ptr, sub_len)
            } else {
                (self.ptr, 0)
            };
            unsafe { Some(StridedViewMut::new_unchecked(sub_ptr, new_stride, sub_len)) }
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let rem = self.back.saturating_sub(self.front);
        (rem, Some(rem))
    }
}

impl<'a, T> DoubleEndedIterator for StridedSplitIterMut<'a, T> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.front >= self.back {
            None
        } else {
            self.back -= 1;
            let i = self.back;
            let new_stride = self.stride * self.n;
            let (sub_ptr, sub_len) = if i < self.len {
                let sub_len = (self.len - i).div_ceil(self.n);
                let sub_ptr = unsafe { offset_ptr(self.ptr, self.stride, i) };
                (sub_ptr, sub_len)
            } else {
                (self.ptr, 0)
            };
            unsafe { Some(StridedViewMut::new_unchecked(sub_ptr, new_stride, sub_len)) }
        }
    }
}

impl<'a, T> ExactSizeIterator for StridedSplitIterMut<'a, T> {
    #[inline]
    fn len(&self) -> usize {
        self.back.saturating_sub(self.front)
    }
}

impl<'a, T> FusedIterator for StridedSplitIterMut<'a, T> {}

impl<'a, T> StridedViewMut<'a, T> {
    /// Attempts to split a mutable strided view into `N` disjoint mutable sub-views.
    ///
    /// # Errors
    ///
    /// Returns [`StridedError::ZeroStride`] if `N == 0`.
    /// Returns [`StridedError::OutOfBounds`] if `stride * N` overflows `usize`.
    #[inline]
    pub fn try_split_strided<const N: usize>(self) -> Result<[StridedViewMut<'a, T>; N]> {
        if N == 0 {
            return Err(StridedError::ZeroStride);
        }

        let stride = self.stride();
        let new_stride = stride.checked_mul(N).ok_or({
            #[cfg(feature = "verbose-errors")]
            {
                StridedError::OutOfBounds {
                    index: N,
                    len: self.len(),
                }
            }
            #[cfg(not(feature = "verbose-errors"))]
            {
                StridedError::OutOfBounds
            }
        })?;

        let len = self.len();
        let ptr = unsafe { NonNull::new_unchecked(self.as_ptr() as *mut T) };

        let views = core::array::from_fn(|i| {
            if i < len {
                let sub_len = (len - i).div_ceil(N);
                // SAFETY:
                // 1. i < len, so offset_ptr(ptr, stride, i) is within the original view's allocated range.
                // 2. Each sub-view accesses elements at indices i + k * N. Since distinct i values are
                //    in distinct residue classes modulo N, elements accessed by different sub-views are disjoint.
                unsafe {
                    let sub_ptr = offset_ptr(ptr, stride, i);
                    StridedViewMut::new_unchecked(sub_ptr, new_stride, sub_len)
                }
            } else {
                // SAFETY: sub_len is 0, so no memory will ever be accessed.
                unsafe { StridedViewMut::new_unchecked(ptr, new_stride, 0) }
            }
        });

        Ok(views)
    }

    /// Splits a mutable strided view into `N` disjoint mutable sub-views.
    ///
    /// # Math & Soundness
    ///
    /// Sub-view `i` ($0 \le i < N$) contains elements at original indices $i + k \cdot N$.
    /// Since $i_1 \not\equiv i_2 \pmod N$ for distinct $i_1, i_2 < N$, the set of memory
    /// locations accessed by each sub-view is strictly disjoint. No two sub-views can
    /// alias the same memory location, preserving Rust's exclusive mutability invariants.
    ///
    /// # Panics
    ///
    /// Panics if `N == 0` or if `stride * N` overflows `usize`.
    #[inline]
    pub fn split_strided<const N: usize>(self) -> [StridedViewMut<'a, T>; N] {
        self.try_split_strided::<N>().expect("failed to split strided view")
    }

    /// Dynamically splits a mutable view into `n` sub-views via an iterator.
    #[inline]
    pub fn try_split_dyn(self, n: usize) -> Result<StridedSplitIterMut<'a, T>> {
        if n == 0 {
            return Err(StridedError::ZeroStride);
        }
        self.stride().checked_mul(n).ok_or({
            #[cfg(feature = "verbose-errors")]
            {
                StridedError::OutOfBounds {
                    index: n,
                    len: self.len(),
                }
            }
            #[cfg(not(feature = "verbose-errors"))]
            {
                StridedError::OutOfBounds
            }
        })?;
        let len = self.len();
        let stride = self.stride();
        let ptr = unsafe { NonNull::new_unchecked(self.as_ptr() as *mut T) };
        Ok(StridedSplitIterMut::new(ptr, stride, len, n))
    }

    #[inline]
    pub fn split_dyn(self, n: usize) -> StridedSplitIterMut<'a, T> {
        self.try_split_dyn(n).expect("failed to split strided view dynamically")
    }

    #[inline]
    pub fn try_split_strided_dyn(self, n: usize) -> Result<StridedSplitIterMut<'a, T>> {
        self.try_split_dyn(n)
    }

    #[inline]
    pub fn split_strided_dyn(self, n: usize) -> StridedSplitIterMut<'a, T> {
        self.split_dyn(n)
    }

    /// Stack-bounded split into `n` sub-views where `n <= MAX_N`.
    #[inline]
    pub fn split_bounded<const MAX_N: usize>(
        self,
        n: usize,
    ) -> Result<[StridedViewMut<'a, T>; MAX_N]> {
        if n == 0 {
            return Err(StridedError::ZeroStride);
        }
        if n > MAX_N {
            return Err({
                #[cfg(feature = "verbose-errors")]
                {
                    StridedError::OutOfBounds {
                        index: n,
                        len: MAX_N,
                    }
                }
                #[cfg(not(feature = "verbose-errors"))]
                {
                    StridedError::OutOfBounds
                }
            });
        }
        let stride = self.stride();
        let new_stride = stride.checked_mul(n).ok_or({
            #[cfg(feature = "verbose-errors")]
            {
                StridedError::OutOfBounds {
                    index: n,
                    len: self.len(),
                }
            }
            #[cfg(not(feature = "verbose-errors"))]
            {
                StridedError::OutOfBounds
            }
        })?;

        let len = self.len();
        let ptr = unsafe { NonNull::new_unchecked(self.as_ptr() as *mut T) };

        let views = core::array::from_fn(|i| {
            if i < n {
                let sub_len = if i < len { (len - i).div_ceil(n) } else { 0 };
                unsafe {
                    let sub_ptr = if i < len { offset_ptr(ptr, stride, i) } else { ptr };
                    StridedViewMut::new_unchecked(sub_ptr, new_stride, sub_len)
                }
            } else {
                unsafe { StridedViewMut::new_unchecked(ptr, new_stride, 0) }
            }
        });

        Ok(views)
    }
}

impl<'a, T> StridedView<'a, T> {
    /// Attempts to split an immutable strided view into `N` disjoint immutable sub-views.
    ///
    /// # Errors
    ///
    /// Returns [`StridedError::ZeroStride`] if `N == 0`.
    /// Returns [`StridedError::OutOfBounds`] if `stride * N` overflows `usize`.
    #[inline]
    pub fn try_split_strided<const N: usize>(self) -> Result<[StridedView<'a, T>; N]> {
        if N == 0 {
            return Err(StridedError::ZeroStride);
        }

        let stride = self.stride();
        let new_stride = stride.checked_mul(N).ok_or({
            #[cfg(feature = "verbose-errors")]
            {
                StridedError::OutOfBounds {
                    index: N,
                    len: self.len(),
                }
            }
            #[cfg(not(feature = "verbose-errors"))]
            {
                StridedError::OutOfBounds
            }
        })?;

        let len = self.len();
        let ptr = unsafe { NonNull::new_unchecked(self.as_ptr() as *mut T) };

        let views = core::array::from_fn(|i| {
            if i < len {
                let sub_len = (len - i).div_ceil(N);
                // SAFETY: Elements accessed by sub-view i are a subset of the valid original view.
                unsafe {
                    let sub_ptr = offset_ptr(ptr, stride, i);
                    StridedView::new_unchecked(sub_ptr, new_stride, sub_len)
                }
            } else {
                // SAFETY: sub_len is 0, so no memory will ever be accessed.
                unsafe { StridedView::new_unchecked(ptr, new_stride, 0) }
            }
        });

        Ok(views)
    }

    /// Splits an immutable strided view into `N` disjoint immutable sub-views.
    ///
    /// # Panics
    ///
    /// Panics if `N == 0` or if `stride * N` overflows `usize`.
    #[inline]
    pub fn split_strided<const N: usize>(self) -> [StridedView<'a, T>; N] {
        self.try_split_strided::<N>().expect("failed to split strided view")
    }

    /// Dynamically splits an immutable view into `n` sub-views via an iterator.
    #[inline]
    pub fn try_split_dyn(self, n: usize) -> Result<StridedSplitIter<'a, T>> {
        if n == 0 {
            return Err(StridedError::ZeroStride);
        }
        self.stride().checked_mul(n).ok_or({
            #[cfg(feature = "verbose-errors")]
            {
                StridedError::OutOfBounds {
                    index: n,
                    len: self.len(),
                }
            }
            #[cfg(not(feature = "verbose-errors"))]
            {
                StridedError::OutOfBounds
            }
        })?;
        let len = self.len();
        let stride = self.stride();
        let ptr = unsafe { NonNull::new_unchecked(self.as_ptr() as *mut T) };
        Ok(StridedSplitIter::new(ptr, stride, len, n))
    }

    #[inline]
    pub fn split_dyn(self, n: usize) -> StridedSplitIter<'a, T> {
        self.try_split_dyn(n).expect("failed to split strided view dynamically")
    }

    #[inline]
    pub fn try_split_strided_dyn(self, n: usize) -> Result<StridedSplitIter<'a, T>> {
        self.try_split_dyn(n)
    }

    #[inline]
    pub fn split_strided_dyn(self, n: usize) -> StridedSplitIter<'a, T> {
        self.split_dyn(n)
    }

    /// Stack-bounded split into `n` sub-views where `n <= MAX_N`.
    #[inline]
    pub fn split_bounded<const MAX_N: usize>(
        self,
        n: usize,
    ) -> Result<[StridedView<'a, T>; MAX_N]> {
        if n == 0 {
            return Err(StridedError::ZeroStride);
        }
        if n > MAX_N {
            return Err({
                #[cfg(feature = "verbose-errors")]
                {
                    StridedError::OutOfBounds {
                        index: n,
                        len: MAX_N,
                    }
                }
                #[cfg(not(feature = "verbose-errors"))]
                {
                    StridedError::OutOfBounds
                }
            });
        }
        let stride = self.stride();
        let new_stride = stride.checked_mul(n).ok_or({
            #[cfg(feature = "verbose-errors")]
            {
                StridedError::OutOfBounds {
                    index: n,
                    len: self.len(),
                }
            }
            #[cfg(not(feature = "verbose-errors"))]
            {
                StridedError::OutOfBounds
            }
        })?;

        let len = self.len();
        let ptr = unsafe { NonNull::new_unchecked(self.as_ptr() as *mut T) };

        let views = core::array::from_fn(|i| {
            if i < n {
                let sub_len = if i < len { (len - i).div_ceil(n) } else { 0 };
                unsafe {
                    let sub_ptr = if i < len { offset_ptr(ptr, stride, i) } else { ptr };
                    StridedView::new_unchecked(sub_ptr, new_stride, sub_len)
                }
            } else {
                unsafe { StridedView::new_unchecked(ptr, new_stride, 0) }
            }
        });

        Ok(views)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_strided_even_odd() {
        let mut data = [10, 20, 30, 40, 50];
        let view = StridedViewMut::from_mut_slice(&mut data);

        let [mut even, mut odd] = view.split_strided::<2>();

        assert_eq!(even.len(), 3); // indices 0, 2, 4 (elements 10, 30, 50)
        assert_eq!(odd.len(), 2);  // indices 1, 3    (elements 20, 40)
        assert_eq!(even.len() + odd.len(), 5);

        assert_eq!(even.get(0), Some(&10));
        assert_eq!(even.get(1), Some(&30));
        assert_eq!(even.get(2), Some(&50));
        assert_eq!(even.get(3), None);

        assert_eq!(odd.get(0), Some(&20));
        assert_eq!(odd.get(1), Some(&40));
        assert_eq!(odd.get(2), None);

        // Mutate even subview element
        if let Some(v) = even.get_mut(1) {
            *v = 300;
        }

        // Mutate odd subview element
        if let Some(v) = odd.get_mut(0) {
            *v = 200;
        }

        assert_eq!(data, [10, 200, 300, 40, 50]);
    }

    #[test]
    fn test_split_strided_by_three() {
        let mut data = [1, 2, 3, 4, 5, 6, 7];
        let view = StridedViewMut::from_mut_slice(&mut data);

        let [v0, v1, v2] = view.split_strided::<3>();

        assert_eq!(v0.len(), 3); // 1, 4, 7
        assert_eq!(v1.len(), 2); // 2, 5
        assert_eq!(v2.len(), 2); // 3, 6
        assert_eq!(v0.len() + v1.len() + v2.len(), 7);

        assert_eq!(v0.get(0), Some(&1));
        assert_eq!(v0.get(1), Some(&4));
        assert_eq!(v0.get(2), Some(&7));

        assert_eq!(v1.get(0), Some(&2));
        assert_eq!(v1.get(1), Some(&5));

        assert_eq!(v2.get(0), Some(&3));
        assert_eq!(v2.get(1), Some(&6));
    }

    #[test]
    fn test_split_strided_short_len() {
        let mut data = [100, 200];
        let view = StridedViewMut::from_mut_slice(&mut data);

        let [v0, v1, v2] = view.split_strided::<3>();

        assert_eq!(v0.len(), 1); // 100
        assert_eq!(v1.len(), 1); // 200
        assert_eq!(v2.len(), 0); // empty

        assert_eq!(v0.get(0), Some(&100));
        assert_eq!(v1.get(0), Some(&200));
        assert_eq!(v2.get(0), None);
    }

    #[test]
    fn test_try_split_strided_zero_and_overflow() {
        let mut data = [1, 2, 3, 4];
        let mut view = StridedViewMut::from_mut_slice(&mut data);

        assert_eq!(view.reborrow().try_split_strided::<0>().unwrap_err(), StridedError::ZeroStride);

        // Construct view with large stride
        let ptr = NonNull::new(data.as_mut_ptr()).unwrap();
        let large_view = unsafe { StridedViewMut::new_unchecked(ptr, usize::MAX / 2 + 10, 2) };
        assert!(large_view.try_split_strided::<2>().is_err());
    }

    #[test]
    fn test_strided_split_iter() {
        let mut data = [10, 20, 30, 40, 50, 60, 70];
        let view = StridedViewMut::from_mut_slice(&mut data);

        let mut iter = view.split_dyn(3);
        assert_eq!(iter.len(), 3);

        let mut v0 = iter.next().unwrap();
        let mut v1 = iter.next().unwrap();
        let mut v2 = iter.next().unwrap();
        assert!(iter.next().is_none());

        assert_eq!(v0.len(), 3); // 10, 40, 70
        assert_eq!(v1.len(), 2); // 20, 50
        assert_eq!(v2.len(), 2); // 30, 60

        v0[1] = 400;
        v1[0] = 200;
        v2[1] = 600;

        assert_eq!(data, [10, 200, 30, 400, 50, 600, 70]);
    }

    #[test]
    fn test_split_bounded() {
        let mut data = [1, 2, 3, 4, 5];
        let mut view = StridedViewMut::from_mut_slice(&mut data);

        // Exceeding MAX_N
        let err = view.reborrow().split_bounded::<2>(3);
        assert!(err.is_err());

        // Zero N
        let err_zero = view.reborrow().split_bounded::<4>(0);
        assert_eq!(err_zero.unwrap_err(), StridedError::ZeroStride);

        // Valid bounded split (n = 2, MAX_N = 4)
        let views = view.split_bounded::<4>(2).unwrap();
        assert_eq!(views[0].len(), 3); // 1, 3, 5
        assert_eq!(views[1].len(), 2); // 2, 4
        assert_eq!(views[2].len(), 0); // empty slot
        assert_eq!(views[3].len(), 0); // empty slot

        assert_eq!(views[0][0], 1);
        assert_eq!(views[1][0], 2);
    }
}
