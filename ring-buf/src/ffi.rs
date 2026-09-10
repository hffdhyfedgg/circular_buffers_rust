use crate::error::RingBufError;
use strided_mem::StridedError;
use raw_storage::StorageError;

/// FFI error codes matching `references/dema_err.h`.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FfiError {
    /// Operation completed successfully (0).
    Ok = 0,
    /// Null pointer error (-1).
    NullPointer = -1,
    /// Invalid argument or dimensions (-2).
    InvalidArg = -2,
    /// Memory allocation or resize failure (-3).
    MemAlloc = -3,
    /// Element or resource not found (-4).
    NotFound = -4,
    /// Operation not supported (-7).
    NotSupported = -7,
    /// Ring buffer is empty (-10).
    BufferEmpty = -10,
    /// Ring buffer is full (-11).
    BufferFull = -11,
}

impl From<FfiError> for i32 {
    #[inline]
    fn from(err: FfiError) -> Self {
        err as i32
    }
}

impl From<RingBufError> for FfiError {
    fn from(err: RingBufError) -> Self {
        match err {
            RingBufError::CapacityZero => FfiError::InvalidArg,
            #[cfg(feature = "verbose-errors")]
            RingBufError::NotPowerOfTwo { .. } => FfiError::InvalidArg,
            #[cfg(not(feature = "verbose-errors"))]
            RingBufError::NotPowerOfTwo => FfiError::InvalidArg,

            #[cfg(feature = "verbose-errors")]
            RingBufError::StorageTooSmall { .. } => FfiError::InvalidArg,
            #[cfg(not(feature = "verbose-errors"))]
            RingBufError::StorageTooSmall => FfiError::InvalidArg,

            #[cfg(feature = "verbose-errors")]
            RingBufError::ChannelOutOfBounds { .. } => FfiError::InvalidArg,
            #[cfg(not(feature = "verbose-errors"))]
            RingBufError::ChannelOutOfBounds => FfiError::InvalidArg,

            #[cfg(feature = "verbose-errors")]
            RingBufError::InvalidTailLength { .. } => FfiError::InvalidArg,
            #[cfg(not(feature = "verbose-errors"))]
            RingBufError::InvalidTailLength => FfiError::InvalidArg,

            #[cfg(feature = "verbose-errors")]
            RingBufError::FilterLengthMismatch { .. } => FfiError::InvalidArg,
            #[cfg(not(feature = "verbose-errors"))]
            RingBufError::FilterLengthMismatch => FfiError::InvalidArg,

            RingBufError::BufferEmpty => FfiError::BufferEmpty,

            #[cfg(feature = "verbose-errors")]
            RingBufError::StorageError(StorageError::ResizeFailed { .. }) => FfiError::MemAlloc,
            #[cfg(not(feature = "verbose-errors"))]
            RingBufError::StorageError(StorageError::ResizeFailed) => FfiError::MemAlloc,

            RingBufError::StridedError(strided_err) => match strided_err {
                StridedError::NullPointer => FfiError::NullPointer,
                #[cfg(feature = "verbose-errors")]
                StridedError::StorageError(StorageError::ResizeFailed { .. }) => FfiError::MemAlloc,
                #[cfg(not(feature = "verbose-errors"))]
                StridedError::StorageError(StorageError::ResizeFailed) => FfiError::MemAlloc,
                _ => FfiError::InvalidArg,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ffi_error_conversions() {
        assert_eq!(i32::from(FfiError::Ok), 0);
        assert_eq!(i32::from(FfiError::NullPointer), -1);
        assert_eq!(i32::from(FfiError::InvalidArg), -2);
        assert_eq!(i32::from(FfiError::MemAlloc), -3);
        assert_eq!(i32::from(FfiError::NotFound), -4);
        assert_eq!(i32::from(FfiError::NotSupported), -7);
        assert_eq!(i32::from(FfiError::BufferEmpty), -10);
        assert_eq!(i32::from(FfiError::BufferFull), -11);

        let err: FfiError = RingBufError::BufferEmpty.into();
        assert_eq!(err, FfiError::BufferEmpty);

        let err: FfiError = RingBufError::CapacityZero.into();
        assert_eq!(err, FfiError::InvalidArg);

        #[cfg(feature = "verbose-errors")]
        let storage_err = StorageError::ResizeFailed { requested: 10, current: 5 };
        #[cfg(not(feature = "verbose-errors"))]
        let storage_err = StorageError::ResizeFailed;

        let err: FfiError = RingBufError::StorageError(storage_err).into();
        assert_eq!(err, FfiError::MemAlloc);
    }
}
