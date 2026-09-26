use core::marker::PhantomData;

use crate::error::{Result, StridedError};
use crate::traits::{View, ViewMut};

#[inline(always)]
fn out_of_bounds_err(_index: usize, _len: usize) -> StridedError {
    #[cfg(feature = "verbose-errors")]
    {
        StridedError::OutOfBounds {
            index: _index,
            len: _len,
        }
    }
    #[cfg(not(feature = "verbose-errors"))]
    {
        StridedError::OutOfBounds
    }
}

/// Представление, ограничивающее видимую длину внутреннего
/// представления значением `effective_len`.
///
/// Элементы за пределами `effective_len` физически существуют,
/// но недоступны через данное представление.
pub struct TailView<'a, T, V: View<'a, Item = T>> {
    inner: V,
    effective_len: usize,
    _marker: PhantomData<&'a T>,
}

impl<'a, T, V: View<'a, Item = T>> TailView<'a, T, V> {
    /// Создаёт представление с ограничением.
    ///
    /// # Errors
    /// `StridedError::OutOfBounds` если `effective_len > inner.len()`.
    pub fn try_new(inner: V, effective_len: usize) -> Result<Self> {
        if effective_len > inner.len() {
            return Err(out_of_bounds_err(effective_len, inner.len()));
        }
        Ok(Self {
            inner,
            effective_len,
            _marker: PhantomData,
        })
    }

    /// Изменяет видимую длину.
    ///
    /// # Errors
    /// `StridedError::OutOfBounds` если `new_len > self.inner.len()`.
    pub fn resize(&mut self, new_len: usize) -> Result<()> {
        if new_len > self.inner.len() {
            return Err(out_of_bounds_err(new_len, self.inner.len()));
        }
        self.effective_len = new_len;
        Ok(())
    }

    /// Возвращает текущую видимую длину.
    #[inline(always)]
    pub fn effective_len(&self) -> usize {
        self.effective_len
    }

    #[inline(always)]
    pub fn inner(&self) -> &V {
        &self.inner
    }
}

impl<'a, T, V: View<'a, Item = T>> View<'a> for TailView<'a, T, V> {
    type Item = T;

    #[inline(always)]
    fn len(&self) -> usize {
        self.effective_len
    }

    #[inline(always)]
    fn get(&self, index: usize) -> Option<&'a T> {
        if index >= self.effective_len {
            return None;
        }
        self.inner.get(index)
    }
}

/// Мутабельное представление с ограничением длины.
pub struct TailViewMut<'a, T, V: ViewMut<'a, Item = T>> {
    inner: V,
    effective_len: usize,
    _marker: PhantomData<&'a mut T>,
}

impl<'a, T, V: ViewMut<'a, Item = T>> TailViewMut<'a, T, V> {
    /// Создаёт представление с ограничением.
    ///
    /// # Errors
    /// `StridedError::OutOfBounds` если `effective_len > inner.len()`.
    pub fn try_new(inner: V, effective_len: usize) -> Result<Self> {
        if effective_len > inner.len() {
            return Err(out_of_bounds_err(effective_len, inner.len()));
        }
        Ok(Self {
            inner,
            effective_len,
            _marker: PhantomData,
        })
    }

    /// Изменяет видимую длину.
    ///
    /// # Errors
    /// `StridedError::OutOfBounds` если `new_len > self.inner.len()`.
    pub fn resize(&mut self, new_len: usize) -> Result<()> {
        if new_len > self.inner.len() {
            return Err(out_of_bounds_err(new_len, self.inner.len()));
        }
        self.effective_len = new_len;
        Ok(())
    }

    /// Возвращает текущую видимую длину.
    #[inline(always)]
    pub fn effective_len(&self) -> usize {
        self.effective_len
    }

    #[inline(always)]
    pub fn inner(&self) -> &V {
        &self.inner
    }
}

impl<'a, T, V: ViewMut<'a, Item = T>> View<'a> for TailViewMut<'a, T, V> {
    type Item = T;

    #[inline(always)]
    fn len(&self) -> usize {
        self.effective_len
    }

    #[inline(always)]
    fn get(&self, index: usize) -> Option<&'a T> {
        if index >= self.effective_len {
            return None;
        }
        self.inner.get(index)
    }
}

impl<'a, T, V: ViewMut<'a, Item = T>> ViewMut<'a> for TailViewMut<'a, T, V> {
    #[inline(always)]
    fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index >= self.effective_len {
            return None;
        }
        self.inner.get_mut(index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::views::{StridedView, StridedViewMut};

    #[test]
    fn test_tail_view() {
        let data = [10, 20, 30, 40, 50];
        let sv = StridedView::from_slice(&data);

        let mut tail = TailView::try_new(sv, 3).unwrap();
        assert_eq!(tail.len(), 3);
        assert_eq!(tail.get(0), Some(&10));
        assert_eq!(tail.get(2), Some(&30));
        assert_eq!(tail.get(3), None);

        tail.resize(4).unwrap();
        assert_eq!(tail.len(), 4);
        assert_eq!(tail.get(3), Some(&40));

        assert!(tail.resize(6).is_err());
    }

    #[test]
    fn test_tail_view_mut() {
        let mut data = [10, 20, 30, 40, 50];
        let sv = StridedViewMut::from_mut_slice(&mut data);

        let mut tail_mut = TailViewMut::try_new(sv, 2).unwrap();
        assert_eq!(tail_mut.len(), 2);
        if let Some(v) = tail_mut.get_mut(1) {
            *v = 200;
        }
        assert_eq!(tail_mut.get(1), Some(&200));
        assert_eq!(tail_mut.get(2), None);
        drop(tail_mut);
        assert_eq!(data[1], 200);
    }
}
