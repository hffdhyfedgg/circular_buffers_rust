use crate::error::{CBufError, CBufResult};
use core::marker::PhantomData;
use core::ops::{Add, Mul};
use raw_storage::traits::StorageMut;

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
    /// Returns [`CBufError::CapacityZero`] if `num_filters == 0` or `filter_length == 0`, or
    /// [`CBufError::StorageTooSmall`] if storage length is less than `num_filters * filter_length`.
    pub fn try_new(mut storage: S, num_filters: usize, filter_length: usize) -> CBufResult<Self> {
        if num_filters == 0 || filter_length == 0 {
            return Err(CBufError::CapacityZero);
        }
        let required = num_filters * filter_length;
        let actual = storage.as_mut_slice().len();
        if actual < required {
            #[cfg(feature = "verbose-errors")]
            return Err(CBufError::StorageTooSmall { required, actual });
            #[cfg(not(feature = "verbose-errors"))]
            return Err(CBufError::StorageTooSmall);
        }
        Ok(Self {
            storage,
            num_filters,
            filter_length,
            _marker: PhantomData,
        })
    }

    /// Returns the number of filters.
    ///
    /// # Panics
    ///
    /// Этот метод никогда не паникует.
    #[inline(always)]
    pub fn num_filters(&self) -> usize {
        self.num_filters
    }

    /// Returns the filter length.
    ///
    /// # Panics
    ///
    /// Этот метод никогда не паникует.
    #[inline(always)]
    pub fn filter_length(&self) -> usize {
        self.filter_length
    }

    /// Returns a slice of coefficients for filter `filter_idx`.
    ///
    /// # Panics
    ///
    /// Этот метод никогда не паникует.
    #[inline(always)]
    pub fn filter_coefficients(&self, filter_idx: usize) -> Option<&[T]> {
        if filter_idx >= self.num_filters {
            None
        } else {
            let start = filter_idx * self.filter_length;
            debug_assert!(start + self.filter_length <= self.storage.as_slice().len());
            self.storage
                .as_slice()
                .get(start..start + self.filter_length)
        }
    }

    /// Returns a mutable slice of coefficients for filter `filter_idx`.
    ///
    /// # Panics
    ///
    /// Этот метод никогда не паникует.
    #[inline(always)]
    pub fn filter_coefficients_mut(&mut self, filter_idx: usize) -> Option<&mut [T]> {
        if filter_idx >= self.num_filters {
            None
        } else {
            let start = filter_idx * self.filter_length;
            debug_assert!(start + self.filter_length <= self.storage.as_mut_slice().len());
            self.storage
                .as_mut_slice()
                .get_mut(start..start + self.filter_length)
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
    /// Returns [`CBufError::ChannelOutOfBounds`] if `filter_idx >= num_filters`.
    pub fn filter_signal<'a>(
        &self,
        filter_idx: usize,
        input: impl IntoIterator<Item = &'a T>,
    ) -> CBufResult<T>
    where
        T: 'a + Copy + Add<Output = T> + Mul<Output = T> + Default,
    {
        let coeffs = self.filter_coefficients(filter_idx).ok_or_else(|| {
            #[cfg(feature = "verbose-errors")]
            {
                CBufError::ChannelOutOfBounds {
                    channel: filter_idx,
                    max_channels: self.num_filters,
                }
            }
            #[cfg(not(feature = "verbose-errors"))]
            {
                CBufError::ChannelOutOfBounds
            }
        })?;

        let mut acc = T::default();
        for (&c, &x) in coeffs.iter().zip(input) {
            acc = acc + c * x;
        }
        Ok(acc)
    }

    /// Применяет фильтр к данным из представления.
    ///
    /// `input` должен отдавать элементы в порядке от НОВЫХ к СТАРЫМ
    /// (индекс 0 = самый новый).
    ///
    /// Принимает любое представление, реализующее `View`.
    pub fn filter_view<'a, V>(&self, filter_idx: usize, input: &V) -> CBufResult<T>
    where
        V: strided_mem::traits::View<'a, Item = T>,
        T: 'a + Copy + Add<Output = T> + Mul<Output = T> + Default,
    {
        let coeffs = self.filter_coefficients(filter_idx).ok_or_else(|| {
            #[cfg(feature = "verbose-errors")]
            {
                CBufError::ChannelOutOfBounds {
                    channel: filter_idx,
                    max_channels: self.num_filters,
                }
            }
            #[cfg(not(feature = "verbose-errors"))]
            {
                CBufError::ChannelOutOfBounds
            }
        })?;

        let mut acc = T::default();
        let len = coeffs.len().min(input.len());
        for i in 0..len {
            if let (Some(&c), Some(&x)) = (coeffs.get(i), input.get(i)) {
                acc = acc + c * x;
            }
        }
        Ok(acc)
    }

    /// Scales all filter coefficients in the filter bank by `scale`.
    ///
    /// # Panics
    ///
    /// Этот метод никогда не паникует.
    pub fn scale_filter_bank(&mut self, scale: T)
    where
        T: Copy + Mul<Output = T>,
    {
        let len = self.num_filters * self.filter_length;
        if let Some(data) = self.storage.as_mut_slice().get_mut(..len) {
            for coeff in data {
                *coeff = *coeff * scale;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cbuf::CBuf;
    use raw_storage::impls::ArrayStorage;

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

        let mut cbuf = CBuf::try_new(ArrayStorage::<f32, 4>::default()).unwrap();
        cbuf.push(10.0);
        cbuf.push(20.0);
        cbuf.push(30.0);

        let view = cbuf.as_ring_view();
        let res0 = fb.filter_view(0, &view).unwrap();
        assert_eq!(res0, 100.0);

        fb.scale_filter_bank(2.0);
        let res0_scaled = fb.filter_view(0, &view).unwrap();
        assert_eq!(res0_scaled, 200.0);
    }
}
