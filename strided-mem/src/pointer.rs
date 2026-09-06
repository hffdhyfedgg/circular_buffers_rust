use core::ptr::NonNull;

/// Calculates element pointer with stride offset.
///
/// # Safety
///
/// Caller must ensure that `ptr` points to valid memory and that offset `index * stride`
/// stays within allocated memory bounds without aliasing violations.
#[inline(always)]
pub(crate) unsafe fn offset_ptr<T>(ptr: NonNull<T>, index: usize, stride: usize) -> *mut T {
    let offset = index
        .checked_mul(stride)
        .expect("stride index multiplication overflowed usize");

    debug_assert!(
        offset
            .checked_mul(core::mem::size_of::<T>())
            .map_or(false, |byte_offset| byte_offset <= isize::MAX as usize),
        "offset in bytes exceeds isize::MAX"
    );

    ptr.as_ptr().add(offset)
}
