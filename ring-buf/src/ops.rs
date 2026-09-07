use core::ops::{Add, Mul};

/// Calculates the sum of all elements in `iter`.
pub fn sum<'a, T: 'a + Copy + Add<Output = T> + Default>(
    iter: impl IntoIterator<Item = &'a T>,
) -> T {
    let mut acc = T::default();
    for &val in iter {
        acc = acc + val;
    }
    acc
}

/// Calculates the dot product of two sequences `a` and `b`.
///
/// Iteration stops as soon as either sequence runs out of elements.
pub fn dot_prod<'a, T: 'a + Copy + Add<Output = T> + Mul<Output = T> + Default>(
    a: impl IntoIterator<Item = &'a T>,
    b: impl IntoIterator<Item = &'a T>,
) -> T {
    let mut acc = T::default();
    for (&x, &y) in a.into_iter().zip(b) {
        acc = acc + x * y;
    }
    acc
}

/// Returns the index and reference to the maximum element in `iter`.
///
/// Returns `None` if the sequence is empty.
pub fn argmax<'a, T: 'a + PartialOrd>(
    iter: impl IntoIterator<Item = &'a T>,
) -> Option<(usize, &'a T)> {
    let mut it = iter.into_iter().enumerate();
    let (first_idx, first_val) = it.next()?;
    let mut max_idx = first_idx;
    let mut max_val = first_val;

    for (idx, val) in it {
        if val > max_val {
            max_idx = idx;
            max_val = val;
        }
    }
    Some((max_idx, max_val))
}

/// Returns the index and reference to the minimum element in `iter`.
///
/// Returns `None` if the sequence is empty.
pub fn argmin<'a, T: 'a + PartialOrd>(
    iter: impl IntoIterator<Item = &'a T>,
) -> Option<(usize, &'a T)> {
    let mut it = iter.into_iter().enumerate();
    let (first_idx, first_val) = it.next()?;
    let mut min_idx = first_idx;
    let mut min_val = first_val;

    for (idx, val) in it {
        if val < min_val {
            min_idx = idx;
            min_val = val;
        }
    }
    Some((min_idx, min_val))
}

/// Applies a closure `f` to each element in `iter`.
pub fn foreach<'a, T: 'a>(iter: impl IntoIterator<Item = &'a T>, mut f: impl FnMut(&'a T)) {
    for val in iter {
        f(val);
    }
}

/// Accumulates values in `iter` starting with `init` and applying `f`.
pub fn accumulate<'a, T: 'a, R>(
    iter: impl IntoIterator<Item = &'a T>,
    init: R,
    mut f: impl FnMut(R, &'a T) -> R,
) -> R {
    let mut acc = init;
    for val in iter {
        acc = f(acc, val);
    }
    acc
}

/// Returns the first element in `iter` matching `predicate`, along with its index.
pub fn select<'a, T: 'a>(
    iter: impl IntoIterator<Item = &'a T>,
    mut predicate: impl FnMut(&'a T) -> bool,
) -> Option<(usize, &'a T)> {
    for (idx, val) in iter.into_iter().enumerate() {
        if predicate(val) {
            return Some((idx, val));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::view2n::CBuf2NViewMut;

    #[test]
    fn test_ops_on_view() {
        let mut mem1 = [0f32; 4];
        let mut mem2 = [0f32; 4];

        let mut v1 = CBuf2NViewMut::try_new(&mut mem1).unwrap();
        let mut v2 = CBuf2NViewMut::try_new(&mut mem2).unwrap();

        v1.push(1.0);
        v1.push(2.0);
        v1.push(3.0);

        v2.push(10.0);
        v2.push(20.0);
        v2.push(30.0);

        // v1 logical: [1.0, 2.0, 3.0]
        // v2 logical: [10.0, 20.0, 30.0]

        assert_eq!(sum(&v1), 6.0);
        assert_eq!(dot_prod(&v1, &v2), 10.0 + 40.0 + 90.0);

        assert_eq!(argmax(&v1), Some((2, &3.0)));
        assert_eq!(argmin(&v1), Some((0, &1.0)));

        let mut visited = 0;
        foreach(&v1, |_| visited += 1);
        assert_eq!(visited, 3);

        let acc = accumulate(&v1, 0.0, |a, &b| a + b * 2.0);
        assert_eq!(acc, 12.0);

        assert_eq!(select(&v1, |&x| x > 1.5), Some((1, &2.0)));
    }
}
