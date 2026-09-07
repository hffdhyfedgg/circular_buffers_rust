use core::ptr::NonNull;

/// Computes the pointer offset for an element at `index` given a constant `stride`.
///
/// # Safety
///
/// The caller must guarantee that `ptr` points to a valid allocation and that
/// `index * stride` does not overflow or access memory outside the allocation bounds.
#[inline(always)]
pub(crate) unsafe fn offset_ptr<T>(ptr: NonNull<T>, stride: usize, index: usize) -> NonNull<T> {
    debug_assert!(
        index.checked_mul(stride).is_some(),
        "index * stride overflowed"
    );
    let offset = index * stride;
    debug_assert!(
        offset
            .checked_mul(core::mem::size_of::<T>())
            .map_or(false, |bytes| bytes <= (isize::MAX as usize)),
        "offset in bytes exceeds isize::MAX"
    );
    // SAFETY: The caller guarantees that ptr + offset is within bounds of the allocated buffer.
    unsafe { NonNull::new_unchecked(ptr.as_ptr().add(offset)) }
}
