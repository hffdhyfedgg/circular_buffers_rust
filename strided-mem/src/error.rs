use core::fmt;

/// Errors that can occur during strided memory operations.
#[cfg(feature = "verbose-errors")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StridedError {
    /// Requested index or range is out of memory bounds.
    OutOfBounds { requested: usize, max: usize },
    /// Stride value cannot be zero.
    ZeroStride,
    /// Overlapping indices were detected during view splitting.
    OverlapDetected { index: usize },
    /// Pointer or length calculation overflowed usize.
    Overflow,
}

/// Errors that can occur during strided memory operations.
#[cfg(not(feature = "verbose-errors"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StridedError {
    /// Requested index or range is out of memory bounds.
    OutOfBounds,
    /// Stride value cannot be zero.
    ZeroStride,
    /// Overlapping indices were detected during view splitting.
    OverlapDetected,
    /// Pointer or length calculation overflowed usize.
    Overflow,
}

impl fmt::Display for StridedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            #[cfg(feature = "verbose-errors")]
            StridedError::OutOfBounds { requested, max } => {
                write!(f, "out of bounds: requested {}, max {}", requested, max)
            }
            #[cfg(not(feature = "verbose-errors"))]
            StridedError::OutOfBounds => write!(f, "out of bounds"),

            #[cfg(feature = "verbose-errors")]
            StridedError::ZeroStride => write!(f, "stride must be greater than zero"),
            #[cfg(not(feature = "verbose-errors"))]
            StridedError::ZeroStride => write!(f, "zero stride"),

            #[cfg(feature = "verbose-errors")]
            StridedError::OverlapDetected { index } => {
                write!(f, "overlap detected at index {}", index)
            }
            #[cfg(not(feature = "verbose-errors"))]
            StridedError::OverlapDetected => write!(f, "overlap detected"),

            #[cfg(feature = "verbose-errors")]
            StridedError::Overflow => write!(f, "arithmetic overflow in stride computation"),
            #[cfg(not(feature = "verbose-errors"))]
            StridedError::Overflow => write!(f, "stride overflow"),
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
            let err = StridedError::OutOfBounds { requested: 10, max: 5 };
            assert!(format!("{}", err).contains("10"));
        }

        #[cfg(not(feature = "verbose-errors"))]
        {
            let err = StridedError::OutOfBounds;
            assert_eq!(format!("{}", err), "out of bounds");
        }
    }
}
