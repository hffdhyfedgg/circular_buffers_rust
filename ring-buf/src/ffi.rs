use crate::error::CBufError;
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
}

impl From<FfiError> for i32 {
    #[inline]
    fn from(err: FfiError) -> Self {
        err as i32
    }
}

impl From<CBufError> for FfiError {
    fn from(err: CBufError) -> Self {
        match err {
            CBufError::CapacityZero => FfiError::InvalidArg,
            #[cfg(feature = "verbose-errors")]
            CBufError::NotPowerOfTwo { .. } => FfiError::InvalidArg,
            #[cfg(not(feature = "verbose-errors"))]
            CBufError::NotPowerOfTwo => FfiError::InvalidArg,

            #[cfg(feature = "verbose-errors")]
            CBufError::StorageTooSmall { .. } => FfiError::InvalidArg,
            #[cfg(not(feature = "verbose-errors"))]
            CBufError::StorageTooSmall => FfiError::InvalidArg,

            #[cfg(feature = "verbose-errors")]
            CBufError::ChannelOutOfBounds { .. } => FfiError::InvalidArg,
            #[cfg(not(feature = "verbose-errors"))]
            CBufError::ChannelOutOfBounds => FfiError::InvalidArg,

            #[cfg(feature = "verbose-errors")]
            CBufError::InvalidTailLength { .. } => FfiError::InvalidArg,
            #[cfg(not(feature = "verbose-errors"))]
            CBufError::InvalidTailLength => FfiError::InvalidArg,

            #[cfg(feature = "verbose-errors")]
            CBufError::FilterLengthMismatch { .. } => FfiError::InvalidArg,
            #[cfg(not(feature = "verbose-errors"))]
            CBufError::FilterLengthMismatch => FfiError::InvalidArg,

            CBufError::BufferEmpty => FfiError::BufferEmpty,

            #[cfg(feature = "verbose-errors")]
            CBufError::StorageError(StorageError::ResizeFailed { .. }) => FfiError::MemAlloc,
            #[cfg(not(feature = "verbose-errors"))]
            CBufError::StorageError(StorageError::ResizeFailed) => FfiError::MemAlloc,

            CBufError::StridedError(strided_err) => match strided_err {
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

        let err: FfiError = CBufError::BufferEmpty.into();
        assert_eq!(err, FfiError::BufferEmpty);

        let err: FfiError = CBufError::CapacityZero.into();
        assert_eq!(err, FfiError::InvalidArg);

        #[cfg(feature = "verbose-errors")]
        let storage_err = StorageError::ResizeFailed { requested: 10, current: 5 };
        #[cfg(not(feature = "verbose-errors"))]
        let storage_err = StorageError::ResizeFailed;

        let err: FfiError = CBufError::StorageError(storage_err).into();
        assert_eq!(err, FfiError::MemAlloc);
    }
}
