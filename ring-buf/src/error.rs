use core::fmt;

/// Errors that can occur in ring buffer operations.
#[cfg(feature = "verbose-errors")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RingBufError {
    /// Capacity must be greater than zero.
    CapacityZero,
    /// Capacity must be a power of two for 2N variants.
    NotPowerOfTwo { capacity: usize },
    /// Storage size is insufficient for the requested dimensions.
    StorageTooSmall { required: usize, actual: usize },
    /// Requested channel index is out of bounds.
    ChannelOutOfBounds { channel: usize, max_channels: usize },
    /// Requested tail length is invalid or larger than capacity.
    InvalidTailLength { tail_len: usize, capacity: usize },
    /// Filter length mismatch during convolution or scaling.
    FilterLengthMismatch { expected: usize, actual: usize },
    /// Buffer is empty.
    BufferEmpty,
}

/// Errors that can occur in ring buffer operations.
#[cfg(not(feature = "verbose-errors"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RingBufError {
    /// Capacity must be greater than zero.
    CapacityZero,
    /// Capacity must be a power of two for 2N variants.
    NotPowerOfTwo,
    /// Storage size is insufficient for the requested dimensions.
    StorageTooSmall,
    /// Requested channel index is out of bounds.
    ChannelOutOfBounds,
    /// Requested tail length is invalid or larger than capacity.
    InvalidTailLength,
    /// Filter length mismatch during convolution or scaling.
    FilterLengthMismatch,
    /// Buffer is empty.
    BufferEmpty,
}

impl fmt::Display for RingBufError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            #[cfg(feature = "verbose-errors")]
            RingBufError::CapacityZero => write!(f, "capacity must be greater than zero"),
            #[cfg(not(feature = "verbose-errors"))]
            RingBufError::CapacityZero => write!(f, "capacity must be greater than zero"),

            #[cfg(feature = "verbose-errors")]
            RingBufError::NotPowerOfTwo { capacity } => {
                write!(f, "capacity {} is not a power of two", capacity)
            }
            #[cfg(not(feature = "verbose-errors"))]
            RingBufError::NotPowerOfTwo => write!(f, "capacity is not a power of two"),

            #[cfg(feature = "verbose-errors")]
            RingBufError::StorageTooSmall { required, actual } => {
                write!(f, "storage too small: required {}, actual {}", required, actual)
            }
            #[cfg(not(feature = "verbose-errors"))]
            RingBufError::StorageTooSmall => write!(f, "storage too small"),

            #[cfg(feature = "verbose-errors")]
            RingBufError::ChannelOutOfBounds { channel, max_channels } => {
                write!(f, "channel index {} out of bounds (max {})", channel, max_channels)
            }
            #[cfg(not(feature = "verbose-errors"))]
            RingBufError::ChannelOutOfBounds => write!(f, "channel index out of bounds"),

            #[cfg(feature = "verbose-errors")]
            RingBufError::InvalidTailLength { tail_len, capacity } => {
                write!(f, "invalid tail length {} for capacity {}", tail_len, capacity)
            }
            #[cfg(not(feature = "verbose-errors"))]
            RingBufError::InvalidTailLength => write!(f, "invalid tail length"),

            #[cfg(feature = "verbose-errors")]
            RingBufError::FilterLengthMismatch { expected, actual } => {
                write!(f, "filter length mismatch: expected {}, actual {}", expected, actual)
            }
            #[cfg(not(feature = "verbose-errors"))]
            RingBufError::FilterLengthMismatch => write!(f, "filter length mismatch"),

            RingBufError::BufferEmpty => write!(f, "buffer is empty"),
        }
    }
}

impl core::error::Error for RingBufError {}

/// Specialized Result type for ring buffer operations.
pub type Result<T> = core::result::Result<T, RingBufError>;

#[cfg(test)]
mod tests {
    use super::*;

    struct DummyBuf([u8; 64], usize);
    impl fmt::Write for DummyBuf {
        fn write_str(&mut self, s: &str) -> fmt::Result {
            let bytes = s.as_bytes();
            let rem = self.0.len() - self.1;
            let len = bytes.len().min(rem);
            self.0[self.1..self.1 + len].copy_from_slice(&bytes[..len]);
            self.1 += len;
            Ok(())
        }
    }

    #[test]
    fn test_error_display() {
        use core::fmt::Write;
        let err = RingBufError::CapacityZero;
        let mut buf = DummyBuf([0; 64], 0);
        write!(buf, "{}", err).unwrap();
        assert!(buf.1 > 0);
    }
}
