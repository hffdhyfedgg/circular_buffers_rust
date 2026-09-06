use core::ptr::NonNull;

/// Computes the pointer offset for an element at `index` given a constant `stride`.
///
/// # Safety
///
/// The caller must guarantee that `ptr` points to a valid allocation and that
/// `index * stride` does not overflow or access memory outside the allocation bounds.
#[inline(always)]
pub unsafe fn offset_ptr<T>(ptr: NonNull<T>, stride: usize, index: usize) -> NonNull<T> {
    // SAFETY: The caller guarantees that ptr + index * stride is within bounds of the allocated buffer.
    unsafe { NonNull::new_unchecked(ptr.as_ptr().add(index * stride)) }
}
