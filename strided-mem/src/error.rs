use core::fmt;

/// Errors that can occur during strided memory operations.
#[cfg(feature = "verbose-errors")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StridedError {
    /// Stride must be greater than zero.
    ZeroStride,
    /// Memory overlap detected between strided views.
    OverlapDetected {
        /// Base offset 1.
        offset1: usize,
        /// Base offset 2.
        offset2: usize,
    },
    /// Index or offset out of bounds.
    OutOfBounds {
        /// Attempted index.
        index: usize,
        /// Maximum length.
        len: usize,
    },
    /// Encountered a null or unaligned pointer.
    NullPointer,
}

/// Errors that can occur during strided memory operations.
#[cfg(not(feature = "verbose-errors"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StridedError {
    /// Stride must be greater than zero.
    ZeroStride,
    /// Memory overlap detected between strided views.
    OverlapDetected,
    /// Index or offset out of bounds.
    OutOfBounds,
    /// Encountered a null or unaligned pointer.
    NullPointer,
}

impl fmt::Display for StridedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            #[cfg(feature = "verbose-errors")]
            StridedError::ZeroStride => write!(f, "stride must be greater than zero"),
            #[cfg(not(feature = "verbose-errors"))]
            StridedError::ZeroStride => write!(f, "zero stride"),

            #[cfg(feature = "verbose-errors")]
            StridedError::OverlapDetected { offset1, offset2 } => {
                write!(f, "memory overlap detected between offsets {} and {}", offset1, offset2)
            }
            #[cfg(not(feature = "verbose-errors"))]
            StridedError::OverlapDetected => write!(f, "overlap detected"),

            #[cfg(feature = "verbose-errors")]
            StridedError::OutOfBounds { index, len } => {
                write!(f, "index {} out of bounds for strided view of length {}", index, len)
            }
            #[cfg(not(feature = "verbose-errors"))]
            StridedError::OutOfBounds => write!(f, "out of bounds"),

            StridedError::NullPointer => write!(f, "null pointer"),
        }
    }
}

impl core::error::Error for StridedError {}

/// A specialized [`Result`](core::result::Result) type for strided memory operations.
pub type Result<T> = core::result::Result<T, StridedError>;

#[cfg(test)]
mod tests {
    use super::*;
    extern crate std;
    use std::format;

    #[test]
    fn test_error_display() {
        #[cfg(feature = "verbose-errors")]
        {
            let err = StridedError::OutOfBounds { index: 10, len: 5 };
            let s = format!("{}", err);
            assert!(s.contains("10"));
            assert!(s.contains("5"));
        }

        #[cfg(not(feature = "verbose-errors"))]
        {
            let err = StridedError::OutOfBounds;
            let s = format!("{}", err);
            assert_eq!(s, "out of bounds");
        }
    }
}
