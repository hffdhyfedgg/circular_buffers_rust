use core::slice;

/// Iterator over immutable references to ring buffer elements in logical order (oldest -> newest).
#[derive(Debug, Clone)]
pub struct CBufIter<'a, T> {
    first: slice::Iter<'a, T>,
    second: slice::Iter<'a, T>,
}

impl<'a, T> CBufIter<'a, T> {
    /// Creates a new iterator from two continuous slices.
    #[inline(always)]
    pub fn new(slice1: &'a [T], slice2: &'a [T]) -> Self {
        Self {
            first: slice1.iter(),
            second: slice2.iter(),
        }
    }
}

impl<'a, T> Iterator for CBufIter<'a, T> {
    type Item = &'a T;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(item) = self.first.next() {
            Some(item)
        } else {
            self.second.next()
        }
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.first.len() + self.second.len();
        (len, Some(len))
    }
}

impl<'a, T> ExactSizeIterator for CBufIter<'a, T> {
    #[inline(always)]
    fn len(&self) -> usize {
        self.first.len() + self.second.len()
    }
}

impl<'a, T> DoubleEndedIterator for CBufIter<'a, T> {
    #[inline(always)]
    fn next_back(&mut self) -> Option<Self::Item> {
        if let Some(item) = self.second.next_back() {
            Some(item)
        } else {
            self.first.next_back()
        }
    }
}

/// Iterator over mutable references to ring buffer elements in logical order (oldest -> newest).
#[derive(Debug)]
pub struct CBufIterMut<'a, T> {
    first: slice::IterMut<'a, T>,
    second: slice::IterMut<'a, T>,
}

impl<'a, T> CBufIterMut<'a, T> {
    /// Creates a new mutable iterator from two continuous mutable slices.
    #[inline(always)]
    pub fn new(slice1: &'a mut [T], slice2: &'a mut [T]) -> Self {
        Self {
            first: slice1.iter_mut(),
            second: slice2.iter_mut(),
        }
    }
}

impl<'a, T> Iterator for CBufIterMut<'a, T> {
    type Item = &'a mut T;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(item) = self.first.next() {
            Some(item)
        } else {
            self.second.next()
        }
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.first.len() + self.second.len();
        (len, Some(len))
    }
}

impl<'a, T> ExactSizeIterator for CBufIterMut<'a, T> {
    #[inline(always)]
    fn len(&self) -> usize {
        self.first.len() + self.second.len()
    }
}

impl<'a, T> DoubleEndedIterator for CBufIterMut<'a, T> {
    #[inline(always)]
    fn next_back(&mut self) -> Option<Self::Item> {
        if let Some(item) = self.second.next_back() {
            Some(item)
        } else {
            self.first.next_back()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iter_forward_and_backward() {
        let s1 = [10, 20, 30];
        let s2 = [40, 50];
        let mut iter = CBufIter::new(&s1, &s2);

        assert_eq!(iter.len(), 5);
        assert_eq!(iter.next(), Some(&10));
        assert_eq!(iter.next_back(), Some(&50));
        assert_eq!(iter.next(), Some(&20));
        assert_eq!(iter.next_back(), Some(&40));
        assert_eq!(iter.next(), Some(&30));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next_back(), None);
    }

    #[test]
    fn test_iter_mut() {
        let mut s1 = [1, 2];
        let mut s2 = [3, 4];
        let mut iter = CBufIterMut::new(&mut s1, &mut s2);

        while let Some(val) = iter.next() {
            *val *= 10;
        }

        assert_eq!(s1, [10, 20]);
        assert_eq!(s2, [30, 40]);
    }
}
