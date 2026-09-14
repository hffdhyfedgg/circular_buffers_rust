use core::marker::PhantomData;

use crate::traits::{View, ViewMut};

/// Инвертирующее представление: меняет порядок элементов
/// внутреннего представления на противоположный.
///
/// `invert.get(i)` эквивалентно `inner.get(inner.len() - 1 - i)`.
pub struct InvertView<'a, T, V: View<'a, Item = T>> {
    inner: V,
    _marker: PhantomData<&'a T>,
}

impl<'a, T, V: View<'a, Item = T>> InvertView<'a, T, V> {
    #[inline(always)]
    pub fn new(inner: V) -> Self {
        Self {
            inner,
            _marker: PhantomData,
        }
    }

    #[inline(always)]
    pub fn inner(&self) -> &V {
        &self.inner
    }
}

impl<'a, T, V: View<'a, Item = T>> View<'a> for InvertView<'a, T, V> {
    type Item = T;

    #[inline(always)]
    fn len(&self) -> usize {
        self.inner.len()
    }

    #[inline(always)]
    fn get(&self, index: usize) -> Option<&'a T> {
        let len = self.inner.len();
        if index >= len {
            return None;
        }
        self.inner.get(len - 1 - index)
    }
}

/// Мутабельный вариант `InvertView`.
pub struct InvertViewMut<'a, T, V: ViewMut<'a, Item = T>> {
    inner: V,
    _marker: PhantomData<&'a mut T>,
}

impl<'a, T, V: ViewMut<'a, Item = T>> InvertViewMut<'a, T, V> {
    #[inline(always)]
    pub fn new(inner: V) -> Self {
        Self {
            inner,
            _marker: PhantomData,
        }
    }

    #[inline(always)]
    pub fn inner(&self) -> &V {
        &self.inner
    }
}

impl<'a, T, V: ViewMut<'a, Item = T>> View<'a> for InvertViewMut<'a, T, V> {
    type Item = T;

    #[inline(always)]
    fn len(&self) -> usize {
        self.inner.len()
    }

    #[inline(always)]
    fn get(&self, index: usize) -> Option<&'a T> {
        let len = self.inner.len();
        if index >= len {
            return None;
        }
        self.inner.get(len - 1 - index)
    }
}

impl<'a, T, V: ViewMut<'a, Item = T>> ViewMut<'a> for InvertViewMut<'a, T, V> {
    #[inline(always)]
    fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        let len = self.inner.len();
        if index >= len {
            return None;
        }
        self.inner.get_mut(len - 1 - index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::views::{StridedView, StridedViewMut};

    #[test]
    fn test_invert_view() {
        let data = [10, 20, 30, 40];
        let sv = StridedView::from_slice(&data);
        let inv = InvertView::new(sv);

        assert_eq!(inv.len(), 4);
        assert_eq!(inv.get(0), Some(&40));
        assert_eq!(inv.get(1), Some(&30));
        assert_eq!(inv.get(2), Some(&20));
        assert_eq!(inv.get(3), Some(&10));
        assert_eq!(inv.get(4), None);
    }

    #[test]
    fn test_invert_view_mut() {
        let mut data = [10, 20, 30, 40];
        let sv = StridedViewMut::from_mut_slice(&mut data);
        let mut inv_mut = InvertViewMut::new(sv);

        assert_eq!(inv_mut.get(0), Some(&40));
        if let Some(v) = inv_mut.get_mut(0) {
            *v = 400;
        }
        assert_eq!(data[3], 400);
    }
}
