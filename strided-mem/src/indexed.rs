use core::marker::PhantomData;
use core::ptr::NonNull;

#[cfg(feature = "alloc")]
extern crate alloc;

use crate::error::{Result, StridedError};

/// A mutable view over arbitrary non-overlapping indexed elements.
#[derive(Debug)]
pub struct IndexedViewMut<'a, T> {
    ptr: NonNull<T>,
    indices: &'a [usize],
    _marker: PhantomData<&'a mut T>,
}

unsafe impl<'a, T: Send> Send for IndexedViewMut<'a, T> {}
unsafe impl<'a, T: Sync> Sync for IndexedViewMut<'a, T> {}

impl<'a, T> IndexedViewMut<'a, T> {
    /// Creates a new `IndexedViewMut` unchecked.
    ///
    /// # Safety
    ///
    /// - `ptr` must point to valid memory for elements up to `max(indices)`.
    /// - All indices in `indices` must be valid, unique, and disjoint from any other active mutable references.
    #[inline(always)]
    pub(crate) unsafe fn new_unchecked(ptr: NonNull<T>, indices: &'a [usize]) -> Self {
        Self {
            ptr,
            indices,
            _marker: PhantomData,
        }
    }

    /// Returns the number of indices in this view.
    ///
    /// # Panics
    ///
    /// Этот метод никогда не паникует.
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.indices.len()
    }

    /// Returns `true` if this view contains no indices.
    ///
    /// # Panics
    ///
    /// Этот метод никогда не паникует.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.indices.is_empty()
    }

    /// Returns the slice of indices represented by this view.
    ///
    /// # Panics
    ///
    /// Этот метод никогда не паникует.
    #[inline(always)]
    pub fn indices(&self) -> &'a [usize] {
        self.indices
    }

    /// Returns an immutable reference to the element at view `index`, or `None` if out of bounds.
    ///
    /// # Panics
    ///
    /// Этот метод никогда не паникует.
    #[inline(always)]
    pub fn get(&self, index: usize) -> Option<&T> {
        if index >= self.indices.len() {
            None
        } else {
            let target_idx = *self.indices.get(index)?;
            unsafe { Some(&*self.ptr.as_ptr().add(target_idx)) }
        }
    }

    /// Returns a mutable reference to the element at view `index`, or `None` if out of bounds.
    ///
    /// # Panics
    ///
    /// Этот метод никогда не паникует.
    #[inline(always)]
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index >= self.indices.len() {
            None
        } else {
            let target_idx = *self.indices.get(index)?;
            unsafe { Some(&mut *self.ptr.as_ptr().add(target_idx)) }
        }
    }
}

/// Splits a mutable slice into multiple `IndexedViewMut` views after validating index bounds and non-overlap at init-time.
///
/// # Errors
///
/// Returns [`StridedError::OutOfBounds`] if any index is `>= slice.len()`.
/// Returns [`StridedError::OverlapDetected`] if any index appears more than once across all index sets.
pub fn split_indexed_mut<'a, T, const K: usize>(
    slice: &'a mut [T],
    index_sets: [&'a [usize]; K],
) -> Result<[IndexedViewMut<'a, T>; K]> {
    let slice_len = slice.len();

    // 1. Check bounds
    for set in &index_sets {
        for &idx in *set {
            if idx >= slice_len {
                #[cfg(feature = "verbose-errors")]
                {
                    return Err(StridedError::OutOfBounds {
                        index: idx,
                        len: slice_len,
                    });
                }
                #[cfg(not(feature = "verbose-errors"))]
                {
                    return Err(StridedError::OutOfBounds);
                }
            }
        }
    }

    // 2. Check for non-overlapping indices across all sets
    let total_m: usize = index_sets.iter().map(|s| s.len()).sum();

    #[cfg(feature = "alloc")]
    {
        use alloc::vec::Vec;
        let mut all_indices = Vec::with_capacity(total_m);
        for set in &index_sets {
            all_indices.extend_from_slice(set);
        }
        all_indices.sort_unstable();
        for i in 1..all_indices.len() {
            if all_indices.get(i) == all_indices.get(i - 1) && all_indices.get(i).is_some() {
                #[cfg(feature = "verbose-errors")]
                {
                    return Err(StridedError::OverlapDetected {
                        offset1: all_indices[i - 1],
                        offset2: all_indices[i],
                    });
                }
                #[cfg(not(feature = "verbose-errors"))]
                {
                    return Err(StridedError::OverlapDetected);
                }
            }
        }
    }

    #[cfg(not(feature = "alloc"))]
    {
        if total_m <= 128 {
            let mut buf = [0usize; 128];
            let mut offset = 0;
            for set in &index_sets {
                if let Some(target) = buf.get_mut(offset..offset + set.len()) {
                    target.copy_from_slice(set);
                }
                offset += set.len();
            }
            if let Some(active_buf) = buf.get_mut(..total_m) {
                active_buf.sort_unstable();
                for i in 1..active_buf.len() {
                    if active_buf.get(i) == active_buf.get(i - 1) && active_buf.get(i).is_some() {
                        #[cfg(feature = "verbose-errors")]
                        {
                            return Err(StridedError::OverlapDetected {
                                offset1: active_buf[i - 1],
                                offset2: active_buf[i],
                            });
                        }
                        #[cfg(not(feature = "verbose-errors"))]
                        {
                            return Err(StridedError::OverlapDetected);
                        }
                    }
                }
            }
        } else {
            for i in 0..index_sets.len() {
                if let Some(set_i) = index_sets.get(i) {
                    for (elem_idx, &idx1) in set_i.iter().enumerate() {
                        if set_i.get(elem_idx + 1..).map_or(false, |s| s.contains(&idx1)) {
                            #[cfg(feature = "verbose-errors")]
                            {
                                return Err(StridedError::OverlapDetected {
                                    offset1: idx1,
                                    offset2: idx1,
                                });
                            }
                            #[cfg(not(feature = "verbose-errors"))]
                            {
                                return Err(StridedError::OverlapDetected);
                            }
                        }
                        for j in (i + 1)..index_sets.len() {
                            if index_sets.get(j).map_or(false, |s| s.contains(&idx1)) {
                                #[cfg(feature = "verbose-errors")]
                                {
                                    return Err(StridedError::OverlapDetected {
                                        offset1: idx1,
                                        offset2: idx1,
                                    });
                                }
                                #[cfg(not(feature = "verbose-errors"))]
                                {
                                    return Err(StridedError::OverlapDetected);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // 3. Construct views safely
    let base_ptr = unsafe { NonNull::new_unchecked(slice.as_mut_ptr()) };
    let views = core::array::from_fn(|k| unsafe {
        IndexedViewMut::new_unchecked(base_ptr, index_sets[k])
    });

    Ok(views)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_indexed_view_mut_success() {
        let mut data = [10, 20, 30, 40, 50, 60];

        let i1 = [0usize, 3usize];
        let i2 = [1usize, 4usize, 5usize];

        let [mut v1, mut v2] = split_indexed_mut(&mut data, [&i1, &i2]).unwrap();

        assert_eq!(v1.len(), 2);
        assert_eq!(v2.len(), 3);

        *v1.get_mut(1).unwrap() = 99;
        *v2.get_mut(2).unwrap() = 88;

        assert_eq!(v1.get(0), Some(&10));
        assert_eq!(v1.get(2), None);

        drop(v1);
        drop(v2);

        assert_eq!(data[3], 99);
        assert_eq!(data[5], 88);
    }

    #[test]
    fn test_split_indexed_mut_out_of_bounds() {
        let mut data = [10, 20, 30];
        let i1 = [0usize, 3usize];

        let res = split_indexed_mut(&mut data, [&i1[..]]);
        assert!(res.is_err());
        #[cfg(feature = "verbose-errors")]
        assert!(matches!(res.unwrap_err(), StridedError::OutOfBounds { .. }));
        #[cfg(not(feature = "verbose-errors"))]
        assert_eq!(res.unwrap_err(), StridedError::OutOfBounds);
    }

    #[test]
    fn test_split_indexed_mut_overlapping_indices() {
        let mut data = [10, 20, 30, 40];

        let i1 = [0usize, 2usize];
        let i2 = [1usize, 2usize];

        let res = split_indexed_mut(&mut data, [&i1[..], &i2[..]]);
        assert!(res.is_err());
        #[cfg(feature = "verbose-errors")]
        assert!(matches!(res.unwrap_err(), StridedError::OverlapDetected { .. }));
        #[cfg(not(feature = "verbose-errors"))]
        assert_eq!(res.unwrap_err(), StridedError::OverlapDetected);

        let i3 = [1usize, 1usize];
        let res2 = split_indexed_mut(&mut data, [&i3[..]]);
        assert!(res2.is_err());
        #[cfg(feature = "verbose-errors")]
        assert!(matches!(res2.unwrap_err(), StridedError::OverlapDetected { .. }));
        #[cfg(not(feature = "verbose-errors"))]
        assert_eq!(res2.unwrap_err(), StridedError::OverlapDetected);
    }
}
