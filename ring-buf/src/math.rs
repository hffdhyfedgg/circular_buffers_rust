//! Ring buffer index arithmetic and slice calculation primitives.

/// Calculates the physical memory index for relative index `rel` (where 0 is newest)
/// for arbitrary capacity buffers.
///
/// # Math
///
/// $$P(\text{rel}) = (H + C - \text{rel}) \pmod C$$
///
/// where $H = \text{head} \in [0, C-1]$, $C = \text{capacity} > 0$, and $\text{rel} \in [0, L-1]$.
/// Since $H \in [0, C-1]$ and $\text{rel} < C$, $H + C - \text{rel} \ge 1 > 0$, guaranteeing
/// no underflow.
#[inline(always)]
pub fn phys_index(head: usize, capacity: usize, rel: usize) -> usize {
    (head + capacity - rel) % capacity
}

/// Calculates the physical memory index of the oldest element ($r = L - 1$)
/// for arbitrary capacity buffers.
///
/// # Math
///
/// $$P_{\text{oldest}} = (H + C + 1 - L) \pmod C$$
///
/// where $H = \text{head} \in [0, C-1]$, $C = \text{capacity} > 0$, and $L = \text{len} \in [1, C]$.
///
/// Since $H \ge 0$ and $1 \le L \le C$:
/// $$1 \le H + C + 1 - L \le 2C - 1$$
///
/// Taking $\pmod C$ yields a valid physical index in $[0, C-1]$, guaranteeing that
/// memory accesses remain strictly within buffer bounds.
#[inline(always)]
pub fn oldest_index(head: usize, capacity: usize, len: usize) -> usize {
    (head + capacity + 1 - len) % capacity
}

/// Advances head pointer for arbitrary capacity buffers.
#[inline(always)]
pub fn next_head(head: usize, capacity: usize) -> usize {
    let next = head + 1;
    if next == capacity {
        0
    } else {
        next
    }
}

/// Returns the buffer data in logical order (oldest to newest) as up to two continuous slices
/// for arbitrary capacity buffers.
///
/// # Math
///
/// When $P_{\text{oldest}} + L \le C$, elements occupy a single contiguous slice:
/// $$\text{data}[P_{\text{oldest}} \dots P_{\text{oldest}} + L - 1]$$
///
/// When $P_{\text{oldest}} + L > C$, elements wrap around the buffer boundary into two slices:
/// $$\text{slice}_1 = \text{data}[P_{\text{oldest}} \dots C - 1]$$
/// $$\text{slice}_2 = \text{data}[0 \dots L - (C - P_{\text{oldest}}) - 1]$$
#[inline(always)]
pub fn as_slices<T>(data: &[T], head: usize, len: usize, capacity: usize) -> (&[T], &[T]) {
    if len == 0 {
        return (&[], &[]);
    }
    let oldest = oldest_index(head, capacity, len);
    if oldest + len <= capacity {
        (&data[oldest..oldest + len], &[])
    } else {
        let first_len = capacity - oldest;
        let second_len = len - first_len;
        (&data[oldest..capacity], &data[..second_len])
    }
}

/// Returns the buffer data in logical order (oldest to newest) as up to two continuous mutable slices
/// for arbitrary capacity buffers.
#[inline(always)]
pub fn as_slices_mut<T>(
    data: &mut [T],
    head: usize,
    len: usize,
    capacity: usize,
) -> (&mut [T], &mut [T]) {
    if len == 0 {
        return (&mut [], &mut []);
    }
    let oldest = oldest_index(head, capacity, len);
    if oldest + len <= capacity {
        let slice = &mut data[oldest..oldest + len];
        (slice, &mut [])
    } else {
        let first_len = capacity - oldest;
        let second_len = len - first_len;
        let (left, right) = data.split_at_mut(oldest);
        let (slice1, _) = right.split_at_mut(first_len);
        let (slice2, _) = left.split_at_mut(second_len);
        (slice1, slice2)
    }
}

/// Calculates the physical memory index for relative index `rel` (where 0 is newest)
/// for power-of-two capacity buffers.
///
/// # Math
///
/// $$P(\text{rel}) = (H + C - \text{rel}) \ \& \ M$$
///
/// where $M = C - 1$ is the bitmask. Since $C = 2^k$, $x \pmod C \equiv x \ \& \ M$ for all non-negative $x$.
#[inline(always)]
pub fn phys_index_2n(head: usize, capacity: usize, rel: usize, mask: usize) -> usize {
    (head + capacity - rel) & mask
}

/// Calculates the physical memory index of the oldest element ($r = L - 1$)
/// for power-of-two capacity buffers using bitwise AND indexing.
///
/// # Math
///
/// $$P_{\text{oldest}} = (H + C + 1 - L) \ \& \ M$$
///
/// where $M = C - 1$.
#[inline(always)]
pub fn oldest_index_2n(head: usize, capacity: usize, len: usize, mask: usize) -> usize {
    (head + capacity + 1 - len) & mask
}

/// Advances head pointer for power-of-two capacity buffers.
#[inline(always)]
pub fn next_head_2n(head: usize, mask: usize) -> usize {
    (head + 1) & mask
}

/// Returns the buffer data in logical order (oldest to newest) as up to two continuous slices
/// for power-of-two capacity buffers.
#[inline(always)]
pub fn as_slices_2n<T>(
    data: &[T],
    head: usize,
    len: usize,
    capacity: usize,
    mask: usize,
) -> (&[T], &[T]) {
    if len == 0 {
        return (&[], &[]);
    }
    let oldest = oldest_index_2n(head, capacity, len, mask);
    if oldest + len <= capacity {
        (&data[oldest..oldest + len], &[])
    } else {
        let first_len = capacity - oldest;
        let second_len = len - first_len;
        (&data[oldest..capacity], &data[..second_len])
    }
}

/// Returns the buffer data in logical order (oldest to newest) as up to two continuous mutable slices
/// for power-of-two capacity buffers.
#[inline(always)]
pub fn as_slices_mut_2n<T>(
    data: &mut [T],
    head: usize,
    len: usize,
    capacity: usize,
    mask: usize,
) -> (&mut [T], &mut [T]) {
    if len == 0 {
        return (&mut [], &mut []);
    }
    let oldest = oldest_index_2n(head, capacity, len, mask);
    if oldest + len <= capacity {
        let slice = &mut data[oldest..oldest + len];
        (slice, &mut [])
    } else {
        let first_len = capacity - oldest;
        let second_len = len - first_len;
        let (left, right) = data.split_at_mut(oldest);
        let (slice1, _) = right.split_at_mut(first_len);
        let (slice2, _) = left.split_at_mut(second_len);
        (slice1, slice2)
    }
}
