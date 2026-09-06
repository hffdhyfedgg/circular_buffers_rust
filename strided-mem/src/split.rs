use core::ptr::NonNull;

use crate::pointer::offset_ptr;
use crate::views::{StridedView, StridedViewMut};

impl<'a, T> StridedViewMut<'a, T> {
    /// Splits a mutable strided view into `N` disjoint mutable sub-views.
    ///
    /// # Math & Soundness
    ///
    /// Sub-view `i` ($0 \le i < N$) contains elements at original indices $i + k \cdot N$.
    /// Since $i_1 \not\equiv i_2 \pmod N$ for distinct $i_1, i_2 < N$, the set of memory
    /// locations accessed by each sub-view is strictly disjoint. No two sub-views can
    /// alias the same memory location, preserving Rust's exclusive mutability invariants.
    #[inline]
    pub fn split_strided<const N: usize>(self) -> [StridedViewMut<'a, T>; N] {
        const { assert!(N > 0, "Stride/split count N must be greater than zero") };

        let len = self.len();
        let stride = self.stride();
        let ptr = unsafe { NonNull::new_unchecked(self.as_ptr() as *mut T) };
        let new_stride = stride * N;

        core::array::from_fn(|i| {
            if len > i {
                let sub_len = (len - i + N - 1) / N;
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
        })
    }
}

impl<'a, T> StridedView<'a, T> {
    /// Splits an immutable strided view into `N` disjoint immutable sub-views.
    #[inline]
    pub fn split_strided<const N: usize>(self) -> [StridedView<'a, T>; N] {
        const { assert!(N > 0, "Stride/split count N must be greater than zero") };

        let len = self.len();
        let stride = self.stride();
        let ptr = unsafe { NonNull::new_unchecked(self.as_ptr() as *mut T) };
        let new_stride = stride * N;

        core::array::from_fn(|i| {
            if len > i {
                let sub_len = (len - i + N - 1) / N;
                // SAFETY: Elements accessed by sub-view i are a subset of the valid original view.
                unsafe {
                    let sub_ptr = offset_ptr(ptr, stride, i);
                    StridedView::new_unchecked(sub_ptr, new_stride, sub_len)
                }
            } else {
                // SAFETY: sub_len is 0, so no memory will ever be accessed.
                unsafe { StridedView::new_unchecked(ptr, new_stride, 0) }
            }
        })
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
}
