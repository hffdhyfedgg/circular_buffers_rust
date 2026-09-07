use core::marker::PhantomData;
use core::ops::{Add, Mul};
use raw_storage::traits::StorageMut;
use crate::error::{Result, RingBufError};

/// Owning filter bank storing `num_filters × filter_length` coefficients.
#[derive(Debug)]
pub struct FilterBank<T, S> {
    storage: S,
    num_filters: usize,
    filter_length: usize,
    _marker: PhantomData<T>,
}

impl<T, S: StorageMut<Item = T>> FilterBank<T, S> {
    /// Creates a new `FilterBank` with `num_filters` filters of length `filter_length`.
    ///
    /// # Errors
    /// Returns [`RingBufError::CapacityZero`] if `num_filters == 0` or `filter_length == 0`, or
    /// [`RingBufError::StorageTooSmall`] if storage length is less than `num_filters * filter_length`.
    pub fn try_new(mut storage: S, num_filters: usize, filter_length: usize) -> Result<Self> {
        if num_filters == 0 || filter_length == 0 {
            return Err(RingBufError::CapacityZero);
        }
        let required = num_filters * filter_length;
        let actual = storage.as_mut_slice().len();
        if actual < required {
            #[cfg(feature = "verbose-errors")]
            return Err(RingBufError::StorageTooSmall { required, actual });
            #[cfg(not(feature = "verbose-errors"))]
            return Err(RingBufError::StorageTooSmall);
        }
        Ok(Self {
            storage,
            num_filters,
            filter_length,
            _marker: PhantomData,
        })
    }

    /// Returns the number of filters.
    #[inline(always)]
    pub fn num_filters(&self) -> usize {
        self.num_filters
    }

    /// Returns the filter length.
    #[inline(always)]
    pub fn filter_length(&self) -> usize {
        self.filter_length
    }

    /// Returns a slice of coefficients for filter `filter_idx`.
    #[inline(always)]
    pub fn filter_coefficients(&self, filter_idx: usize) -> Option<&[T]> {
        if filter_idx >= self.num_filters {
            None
        } else {
            let start = filter_idx * self.filter_length;
            Some(&self.storage.as_slice()[start..start + self.filter_length])
        }
    }

    /// Returns a mutable slice of coefficients for filter `filter_idx`.
    #[inline(always)]
    pub fn filter_coefficients_mut(&mut self, filter_idx: usize) -> Option<&mut [T]> {
        if filter_idx >= self.num_filters {
            None
        } else {
            let start = filter_idx * self.filter_length;
            Some(&mut self.storage.as_mut_slice()[start..start + self.filter_length])
        }
    }

    /// Applies filter `filter_idx` to input sequence `input` using purely iterative inner loop.
    ///
    /// # FIR Convolution Order
    ///
    /// `input` must yield elements in newest-first order ($x_0 = x_{\text{newest}}, x_1 = x_{\text{newest}-1}, \dots$),
    /// evaluating the causal FIR filter formula:
    ///
    /// $$y = \sum_{i=0}^{N-1} h_i \cdot x_{\text{newest}-i}$$
    ///
    /// # Errors
    /// Returns [`RingBufError::ChannelOutOfBounds`] if `filter_idx >= num_filters`.
    pub fn filter_signal<'a>(
        &self,
        filter_idx: usize,
        input: impl IntoIterator<Item = &'a T>,
    ) -> Result<T>
    where
        T: 'a + Copy + Add<Output = T> + Mul<Output = T> + Default,
    {
        let coeffs = self.filter_coefficients(filter_idx).ok_or_else(|| {
            #[cfg(feature = "verbose-errors")]
            {
                RingBufError::ChannelOutOfBounds {
                    channel: filter_idx,
                    max_channels: self.num_filters,
                }
            }
            #[cfg(not(feature = "verbose-errors"))]
            {
                RingBufError::ChannelOutOfBounds
            }
        })?;

        let mut acc = T::default();
        for (&c, &x) in coeffs.iter().zip(input) {
            acc = acc + c * x;
        }
        Ok(acc)
    }

    /// Scales all filter coefficients in the filter bank by `scale`.
    pub fn scale_filter_bank(&mut self, scale: T)
    where
        T: Copy + Mul<Output = T>,
    {
        let len = self.num_filters * self.filter_length;
        let data = &mut self.storage.as_mut_slice()[..len];
        for coeff in data {
            *coeff = *coeff * scale;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use raw_storage::impls::ArrayStorage;
    use crate::cbuf::CBuf;
    use crate::cbuf2n::CBuf2N;
    use crate::stack::CBufStack;
    use crate::tail::CBufTail;
    use crate::view2n::CBuf2NViewMut;

    #[test]
    fn test_filter_bank() {
        let storage = ArrayStorage::<f32, 6>::default(); // 2 filters x 3 length
        let mut fb = FilterBank::try_new(storage, 2, 3).unwrap();

        assert_eq!(fb.num_filters(), 2);
        assert_eq!(fb.filter_length(), 3);

        if let Some(f0) = fb.filter_coefficients_mut(0) {
            f0.copy_from_slice(&[1.0, 2.0, 3.0]);
        }
        if let Some(f1) = fb.filter_coefficients_mut(1) {
            f1.copy_from_slice(&[0.5, 0.5, 0.5]);
        }

        let mut mem = [0f32; 4];
        let mut view = CBuf2NViewMut::try_new(&mut mem).unwrap();
        view.push(10.0);
        view.push(20.0);
        view.push(30.0);

        // FIR convolution order (newest first):
        // newest = 30.0 (rel 0), 20.0 (rel 1), oldest = 10.0 (rel 2)
        // coeffs 0: [1.0, 2.0, 3.0]
        // y = 1.0 * 30.0 + 2.0 * 20.0 + 3.0 * 10.0 = 30 + 40 + 30 = 100.0
        let res0 = fb.filter_signal(0, view.iter_newest_first()).unwrap();
        assert_eq!(res0, 100.0);

        fb.scale_filter_bank(2.0);
        let res0_scaled = fb.filter_signal(0, view.iter_newest_first()).unwrap();
        assert_eq!(res0_scaled, 200.0);
    }

    #[test]
    fn test_filter_signal_across_buffer_types() {
        let storage = ArrayStorage::<f32, 3>::default();
        let mut fb = FilterBank::try_new(storage, 1, 3).unwrap();
        fb.filter_coefficients_mut(0).unwrap().copy_from_slice(&[1.0, 0.5, 0.25]);

        // CBuf
        let mut cbuf = CBuf::try_new(ArrayStorage::<f32, 3>::default()).unwrap();
        cbuf.push(10.0);
        cbuf.push(20.0);
        cbuf.push(40.0); // newest: 40, rel 1: 20, rel 2: 10
        // y = 1.0*40 + 0.5*20 + 0.25*10 = 40 + 10 + 2.5 = 52.5
        let res_cbuf = fb.filter_signal(0, cbuf.iter_newest_first()).unwrap();
        assert_eq!(res_cbuf, 52.5);

        // CBuf2N
        let mut cbuf2n = CBuf2N::try_new(ArrayStorage::<f32, 4>::default()).unwrap();
        cbuf2n.push(10.0);
        cbuf2n.push(20.0);
        cbuf2n.push(40.0);
        let res_cbuf2n = fb.filter_signal(0, cbuf2n.iter_newest_first()).unwrap();
        assert_eq!(res_cbuf2n, 52.5);

        // CBufTail
        let mut tail_buf = CBufTail::try_new(ArrayStorage::<f32, 4>::default(), 1).unwrap();
        tail_buf.push(5.0);  // truncated by tail_len = 1
        tail_buf.push(10.0);
        tail_buf.push(20.0);
        tail_buf.push(40.0); // newest: 40, rel 1: 20, rel 2: 10
        let res_tail = fb.filter_signal(0, tail_buf.iter_newest_first()).unwrap();
        assert_eq!(res_tail, 52.5);

        // CBufStack
        let mut stack = CBufStack::try_new(ArrayStorage::<f32, 6>::default(), 2, 3).unwrap();
        stack.push_all(&[10.0, 0.0]);
        stack.push_all(&[20.0, 0.0]);
        stack.push_all(&[40.0, 0.0]);
        let res_stack = fb.filter_signal(0, stack.channel_iter_newest_first(0).unwrap()).unwrap();
        assert_eq!(res_stack, 52.5);
    }
}
