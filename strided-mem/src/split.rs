use core::marker::PhantomData;
use core::ptr::NonNull;

use crate::error::{Result, StridedError};
use crate::pointer::offset_ptr;
use crate::views::StridedViewMut;

/// Lazy iterator splitting memory into strided mutable views on the fly (Approach A).
#[derive(Debug)]
pub struct StridedSplitIter<'a, T> {
    base_ptr: NonNull<T>,
    base_stride: usize,
    total_len: usize,
    n: usize,
    current_i: usize,
    _marker: PhantomData<&'a mut T>,
}

impl<'a, T> StridedSplitIter<'a, T> {
    pub(crate) fn new(base_ptr: NonNull<T>, base_stride: usize, total_len: usize, n: usize) -> Self {
        Self {
            base_ptr,
            base_stride,
            total_len,
            n,
            current_i: 0,
            _marker: PhantomData,
        }
    }
}

impl<'a, T> Iterator for StridedSplitIter<'a, T> {
    type Item = StridedViewMut<'a, T>;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.n == 0 || self.current_i >= self.n {
            None
        } else {
            let i = self.current_i;
            self.current_i += 1;
            let new_stride = self.base_stride.checked_mul(self.n)?;
            let sub_len = self.total_len.checked_sub(i).map_or(0, |rem| rem.div_ceil(self.n));
            unsafe {
                let start_ptr = NonNull::new_unchecked(offset_ptr(self.base_ptr, i, self.base_stride));
                Some(StridedViewMut::new_unchecked(start_ptr, new_stride, sub_len))
            }
        }
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.n == 0 || self.current_i >= self.n {
            (0, Some(0))
        } else {
            let rem = self.n - self.current_i;
            (rem, Some(rem))
        }
    }
}

impl<'a, T> ExactSizeIterator for StridedSplitIter<'a, T> {
    #[inline(always)]
    fn len(&self) -> usize {
        if self.n == 0 || self.current_i >= self.n {
            0
        } else {
            self.n - self.current_i
        }
    }
}

/// Trait for strided splitting of continuous mutable memory buffers and strided views.
pub trait SplitStrided<'a, T> {
    /// Attempts to split the memory into `N` strided mutable views.
    ///
    /// # Errors
    ///
    /// Returns [`StridedError::ZeroStride`] if `N == 0`, or [`StridedError::Overflow`] if stride multiplication overflows `usize`.
    fn try_split_strided<const N: usize>(self) -> Result<[StridedViewMut<'a, T>; N]>;

    /// Splitting method wrapper over `try_split_strided`.
    ///
    /// # Panics
    ///
    /// Panics if `N == 0` or if stride multiplication overflows `usize`.
    fn split_strided<const N: usize>(self) -> [StridedViewMut<'a, T>; N]
    where
        Self: Sized,
    {
        debug_assert!(N > 0, "N must be greater than zero");
        self.try_split_strided::<N>().expect("split_strided failed")
    }

    /// Creates a lazy iterator over `n` strided sub-views (Approach A).
    fn split_strided_iter(self, n: usize) -> StridedSplitIter<'a, T>;

    /// Splits the memory into up to `MAX_N` sub-views on the stack (Approach B).
    ///
    /// # Errors
    ///
    /// Returns [`StridedError::OutOfBounds`] if `n > MAX_N`.
    fn split_bounded<const MAX_N: usize>(
        self,
        n: usize,
    ) -> Result<([StridedViewMut<'a, T>; MAX_N], usize)>;
}

impl<'a, T> SplitStrided<'a, T> for &'a mut [T] {
    fn try_split_strided<const N: usize>(self) -> Result<[StridedViewMut<'a, T>; N]> {
        if N == 0 {
            return Err(StridedError::ZeroStride);
        }

        let total_len = self.len();
        let base_ptr = unsafe { NonNull::new_unchecked(self.as_mut_ptr()) };
        let base_stride = 1;
        let new_stride = N;

        let views = core::array::from_fn(|i| {
            let sub_len = total_len.checked_sub(i).map_or(0, |rem| rem.div_ceil(N));
            unsafe {
                let start_ptr = NonNull::new_unchecked(offset_ptr(base_ptr, i, base_stride));
                StridedViewMut::new_unchecked(start_ptr, new_stride, sub_len)
            }
        });

        Ok(views)
    }

    fn split_strided_iter(self, n: usize) -> StridedSplitIter<'a, T> {
        let total_len = self.len();
        let base_ptr = unsafe { NonNull::new_unchecked(self.as_mut_ptr()) };
        StridedSplitIter::new(base_ptr, 1, total_len, n)
    }

    fn split_bounded<const MAX_N: usize>(
        self,
        n: usize,
    ) -> Result<([StridedViewMut<'a, T>; MAX_N], usize)> {
        if n > MAX_N {
            #[cfg(feature = "verbose-errors")]
            {
                return Err(StridedError::OutOfBounds {
                    requested: n,
                    max: MAX_N,
                });
            }
            #[cfg(not(feature = "verbose-errors"))]
            {
                return Err(StridedError::OutOfBounds);
            }
        }

        let total_len = self.len();
        let base_ptr = unsafe { NonNull::new_unchecked(self.as_mut_ptr()) };

        if n == 0 {
            let arr = core::array::from_fn(|_| unsafe {
                StridedViewMut::new_unchecked(base_ptr, 1, 0)
            });
            return Ok((arr, 0));
        }

        let new_stride = n;

        let arr = core::array::from_fn(|i| {
            if i < n {
                let sub_len = total_len.checked_sub(i).map_or(0, |rem| rem.div_ceil(n));
                unsafe {
                    let start_ptr = NonNull::new_unchecked(offset_ptr(base_ptr, i, 1));
                    StridedViewMut::new_unchecked(start_ptr, new_stride, sub_len)
                }
            } else {
                unsafe { StridedViewMut::new_unchecked(base_ptr, new_stride, 0) }
            }
        });

        Ok((arr, n))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_try_split_strided_basic() {
        let mut data = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        let [mut v0, mut v1, mut v2] = data.try_split_strided::<3>().unwrap();

        assert_eq!(v0.len(), 4);
        assert_eq!(v1.len(), 3);
        assert_eq!(v2.len(), 3);

        v0[0] = 100;
        v1[0] = 101;
        v2[0] = 102;

        assert_eq!(data[0], 100);
        assert_eq!(data[1], 101);
        assert_eq!(data[2], 102);
    }

    #[test]
    fn test_try_split_strided_overflow_and_near_max() {
        let mut data = [1, 2, 3];
        let ptr = NonNull::new(data.as_mut_ptr()).unwrap();
        let huge_stride = usize::MAX / 2 + 10;
        let view_mut = unsafe { StridedViewMut::new_unchecked(ptr, huge_stride, 2) };

        let res = view_mut.try_split_strided::<2>();
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), StridedError::Overflow);
    }

    #[test]
    fn test_split_strided_iter() {
        let mut data = [0, 1, 2, 3, 4, 5, 6, 7];
        let mut split_iter = data.split_strided_iter(3);

        assert_eq!(split_iter.len(), 3);

        let mut v0 = split_iter.next().unwrap();
        let mut v1 = split_iter.next().unwrap();
        let mut v2 = split_iter.next().unwrap();

        assert!(split_iter.next().is_none());

        assert_eq!(v0.len(), 3);
        assert_eq!(v1.len(), 3);
        assert_eq!(v2.len(), 2);

        v0[0] = 90;
        v1[0] = 91;
        v2[0] = 92;

        assert_eq!(data[0], 90);
        assert_eq!(data[1], 91);
        assert_eq!(data[2], 92);
    }

    #[test]
    fn test_split_bounded() {
        let mut data = [10, 20, 30, 40, 50];

        let (views, n) = data.as_mut_slice().split_bounded::<4>(3).unwrap();
        assert_eq!(n, 3);
        assert_eq!(views[0].len(), 2);
        assert_eq!(views[1].len(), 2);
        assert_eq!(views[2].len(), 1);
        assert_eq!(views[3].len(), 0);

        let mut data2 = [1, 2, 3];
        let err_res = data2.as_mut_slice().split_bounded::<2>(3);
        assert!(err_res.is_err());
        #[cfg(feature = "verbose-errors")]
        assert!(matches!(err_res.unwrap_err(), StridedError::OutOfBounds { .. }));
        #[cfg(not(feature = "verbose-errors"))]
        assert_eq!(err_res.unwrap_err(), StridedError::OutOfBounds);
    }
}

impl<'a, T> SplitStrided<'a, T> for StridedViewMut<'a, T> {
    fn try_split_strided<const N: usize>(self) -> Result<[StridedViewMut<'a, T>; N]> {
        if N == 0 {
            return Err(StridedError::ZeroStride);
        }

        let total_len = self.len();
        let base_stride = self.stride();
        let base_ptr = self.as_ptr_non_null();
        let new_stride = base_stride.checked_mul(N).ok_or(StridedError::Overflow)?;

        let views = core::array::from_fn(|i| {
            let sub_len = total_len.checked_sub(i).map_or(0, |rem| rem.div_ceil(N));
            unsafe {
                let start_ptr = NonNull::new_unchecked(offset_ptr(base_ptr, i, base_stride));
                StridedViewMut::new_unchecked(start_ptr, new_stride, sub_len)
            }
        });

        Ok(views)
    }

    fn split_strided_iter(self, n: usize) -> StridedSplitIter<'a, T> {
        let total_len = self.len();
        let base_stride = self.stride();
        let base_ptr = self.as_ptr_non_null();
        StridedSplitIter::new(base_ptr, base_stride, total_len, n)
    }

    fn split_bounded<const MAX_N: usize>(
        self,
        n: usize,
    ) -> Result<([StridedViewMut<'a, T>; MAX_N], usize)> {
        if n > MAX_N {
            #[cfg(feature = "verbose-errors")]
            {
                return Err(StridedError::OutOfBounds {
                    requested: n,
                    max: MAX_N,
                });
            }
            #[cfg(not(feature = "verbose-errors"))]
            {
                return Err(StridedError::OutOfBounds);
            }
        }

        let total_len = self.len();
        let base_stride = self.stride();
        let base_ptr = self.as_ptr_non_null();

        if n == 0 {
            let arr = core::array::from_fn(|_| unsafe {
                StridedViewMut::new_unchecked(base_ptr, base_stride, 0)
            });
            return Ok((arr, 0));
        }

        let new_stride = base_stride.checked_mul(n).ok_or(StridedError::Overflow)?;

        let arr = core::array::from_fn(|i| {
            if i < n {
                let sub_len = total_len.checked_sub(i).map_or(0, |rem| rem.div_ceil(n));
                unsafe {
                    let start_ptr = NonNull::new_unchecked(offset_ptr(base_ptr, i, base_stride));
                    StridedViewMut::new_unchecked(start_ptr, new_stride, sub_len)
                }
            } else {
                unsafe { StridedViewMut::new_unchecked(base_ptr, new_stride, 0) }
            }
        });

        Ok((arr, n))
    }
}
