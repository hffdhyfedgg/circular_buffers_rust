//! Ring buffer index arithmetic and slice calculation primitives.

/// Converts a relative index `rel` (which may be negative) to a 0-based non-negative relative index.
///
/// - `rel >= 0`: 0 is newest, 1 is second newest, etc.
/// - `rel < 0`: -1 is oldest, -2 is second oldest, etc.
///
/// # Panics
///
/// Этот метод никогда не паникует.
#[inline(always)]
pub fn rel_to_index(rel: isize, len: usize) -> Option<usize> {
    if rel >= 0 {
        let idx = rel as usize;
        if idx < len {
            Some(idx)
        } else {
            None
        }
    } else {
        let neg = rel.unsigned_abs();
        if neg > len || len == 0 {
            None
        } else {
            Some(len - neg)
        }
    }
}

/// Calculates the physical memory index for relative index `rel` (where 0 is newest)
/// for arbitrary capacity buffers.
///
/// # Invariants
///
/// `head < capacity`, `len <= capacity`, `rel < len` checked before calling, `phys` is guaranteed in `[0, capacity)`.
///
/// # Panics
///
/// Этот метод никогда не паникует.
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
    debug_assert!(capacity > 0);
    debug_assert!(head < capacity);
    debug_assert!(rel < capacity);
    (head + capacity - rel) % capacity
}

/// Calculates the physical memory index of the oldest element ($r = L - 1$)
/// for arbitrary capacity buffers.
///
/// # Invariants
///
/// `head < capacity`, `len <= capacity`, `phys` is guaranteed in `[0, capacity)`.
///
/// # Panics
///
/// Этот метод никогда не паникует.
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
    debug_assert!(capacity > 0);
    debug_assert!(head < capacity);
    debug_assert!(len <= capacity);
    debug_assert!(len > 0);
    (head + capacity + 1 - len) % capacity
}

/// Advances head pointer for arbitrary capacity buffers.
///
/// # Panics
///
/// Этот метод никогда не паникует.
#[inline(always)]
pub fn next_head(head: usize, capacity: usize) -> usize {
    debug_assert!(capacity > 0);
    debug_assert!(head < capacity);
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
/// # Panics
///
/// Этот метод никогда не паникует.
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
    if len == 0 || data.is_empty() {
        return (&[], &[]);
    }
    let oldest = oldest_index(head, capacity, len);
    if oldest + len <= capacity {
        (data.get(oldest..oldest + len).unwrap_or(&[]), &[])
    } else {
        let first_len = capacity - oldest;
        let second_len = len - first_len;
        (
            data.get(oldest..capacity).unwrap_or(&[]),
            data.get(..second_len).unwrap_or(&[]),
        )
    }
}

/// Returns the buffer data in logical order (oldest to newest) as up to two continuous mutable slices
/// for arbitrary capacity buffers.
///
/// # Panics
///
/// Этот метод никогда не паникует.
#[inline(always)]
pub fn as_slices_mut<T>(
    data: &mut [T],
    head: usize,
    len: usize,
    capacity: usize,
) -> (&mut [T], &mut [T]) {
    if len == 0 || data.is_empty() {
        return (&mut [], &mut []);
    }
    let oldest = oldest_index(head, capacity, len);
    if oldest + len <= capacity {
        let slice = data.get_mut(oldest..oldest + len).unwrap_or(&mut []);
        (slice, &mut [])
    } else {
        let first_len = capacity - oldest;
        let second_len = len - first_len;
        debug_assert!(oldest <= data.len());
        let (left, right) = data.split_at_mut(oldest);
        debug_assert!(first_len <= right.len());
        let (slice1, _) = right.split_at_mut(first_len);
        debug_assert!(second_len <= left.len());
        let (slice2, _) = left.split_at_mut(second_len);
        (slice1, slice2)
    }
}

/// Calculates the physical memory index for relative index `rel` (where 0 is newest)
/// for power-of-two capacity buffers.
///
/// # Invariants
///
/// `head < capacity`, `len <= capacity`, `rel < len` checked before calling, `phys` is guaranteed in `[0, capacity)`.
///
/// # Panics
///
/// Этот метод никогда не паникует.
///
/// # Math
///
/// $$P(\text{rel}) = (H + C - \text{rel}) \ \& \ M$$
///
/// where $M = C - 1$ is the bitmask. Since $C = 2^k$, $x \pmod C \equiv x \ \& \ M$ for all non-negative $x$.
#[inline(always)]
pub fn phys_index_2n(head: usize, capacity: usize, rel: usize, mask: usize) -> usize {
    debug_assert!(capacity > 0);
    debug_assert!(head < capacity);
    debug_assert!(rel < capacity);
    (head + capacity - rel) & mask
}

/// Calculates the physical memory index of the oldest element ($r = L - 1$)
/// for power-of-two capacity buffers using bitwise AND indexing.
///
/// # Invariants
///
/// `head < capacity`, `len <= capacity`, `phys` is guaranteed in `[0, capacity)`.
///
/// # Panics
///
/// Этот метод никогда не паникует.
///
/// # Math
///
/// $$P_{\text{oldest}} = (H + C + 1 - L) \ \& \ M$$
///
/// where $M = C - 1$.
#[inline(always)]
pub fn oldest_index_2n(head: usize, capacity: usize, len: usize, mask: usize) -> usize {
    debug_assert!(capacity > 0);
    debug_assert!(head < capacity);
    debug_assert!(len <= capacity);
    debug_assert!(len > 0);
    (head + capacity + 1 - len) & mask
}

/// Advances head pointer for power-of-two capacity buffers.
///
/// # Panics
///
/// Этот метод никогда не паникует.
#[inline(always)]
pub fn next_head_2n(head: usize, mask: usize) -> usize {
    (head + 1) & mask
}

/// Returns the buffer data in logical order (oldest to newest) as up to two continuous slices
/// for power-of-two capacity buffers.
///
/// # Panics
///
/// Этот метод никогда не паникует.
#[inline(always)]
pub fn as_slices_2n<T>(
    data: &[T],
    head: usize,
    len: usize,
    capacity: usize,
    mask: usize,
) -> (&[T], &[T]) {
    if len == 0 || data.is_empty() {
        return (&[], &[]);
    }
    let oldest = oldest_index_2n(head, capacity, len, mask);
    if oldest + len <= capacity {
        (data.get(oldest..oldest + len).unwrap_or(&[]), &[])
    } else {
        let first_len = capacity - oldest;
        let second_len = len - first_len;
        (
            data.get(oldest..capacity).unwrap_or(&[]),
            data.get(..second_len).unwrap_or(&[]),
        )
    }
}

/// Returns the buffer data in logical order (oldest to newest) as up to two continuous mutable slices
/// for power-of-two capacity buffers.
///
/// # Panics
///
/// Этот метод никогда не паникует.
#[inline(always)]
pub fn as_slices_mut_2n<T>(
    data: &mut [T],
    head: usize,
    len: usize,
    capacity: usize,
    mask: usize,
) -> (&mut [T], &mut [T]) {
    if len == 0 || data.is_empty() {
        return (&mut [], &mut []);
    }
    let oldest = oldest_index_2n(head, capacity, len, mask);
    if oldest + len <= capacity {
        let slice = data.get_mut(oldest..oldest + len).unwrap_or(&mut []);
        (slice, &mut [])
    } else {
        let first_len = capacity - oldest;
        let second_len = len - first_len;
        debug_assert!(oldest <= data.len());
        let (left, right) = data.split_at_mut(oldest);
        debug_assert!(first_len <= right.len());
        let (slice1, _) = right.split_at_mut(first_len);
        debug_assert!(second_len <= left.len());
        let (slice2, _) = left.split_at_mut(second_len);
        (slice1, slice2)
    }
}
