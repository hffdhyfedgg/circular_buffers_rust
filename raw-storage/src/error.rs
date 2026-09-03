use core::fmt;

/// Errors that can occur during storage operations.
#[cfg(feature = "verbose-errors")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageError {
    /// Failed to resize storage to the requested capacity.
    ResizeFailed {
        /// The requested capacity that could not be allocated or reserved.
        requested: usize,
        /// The current capacity of the storage before the failed attempt.
        current: usize,
    },
}

/// Errors that can occur during storage operations.
#[cfg(not(feature = "verbose-errors"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageError {
    /// Failed to resize storage to the requested capacity.
    ResizeFailed,
}

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            #[cfg(feature = "verbose-errors")]
            StorageError::ResizeFailed { requested, current } => {
                write!(
                    f,
                    "failed to resize storage to capacity {} (current capacity: {})",
                    requested, current
                )
            }
            #[cfg(not(feature = "verbose-errors"))]
            StorageError::ResizeFailed => write!(f, "failed to resize storage"),
        }
    }
}

impl core::error::Error for StorageError {}

/// A specialized [`Result`](core::result::Result) type for storage operations.
pub type Result<T> = core::result::Result<T, StorageError>;

#[cfg(test)]
mod tests {
    use super::*;
    extern crate std;
    use std::format;

    #[test]
    fn test_error_display() {
        #[cfg(feature = "verbose-errors")]
        {
            let err = StorageError::ResizeFailed {
                requested: 100,
                current: 50,
            };
            let s = format!("{}", err);
            assert!(s.contains("100"));
            assert!(s.contains("50"));
        }

        #[cfg(not(feature = "verbose-errors"))]
        {
            let err = StorageError::ResizeFailed;
            let s = format!("{}", err);
            assert_eq!(s, "failed to resize storage");
        }
    }
}
