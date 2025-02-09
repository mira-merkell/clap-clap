//! Timestamp: the number of seconds since [`UNIX_EPOCH`].
//!
//! [`UNIX_EPOCH`]: std::time::UNIX_EPOCH

use std::num::NonZeroU64;

use crate::ffi::clap_timestamp;

/// Timestamp: the number of seconds since [`UNIX_EPOCH`].
///
/// # Example
///
/// ```rust
/// # use clap_clap::timestamp::TimeStamp;
/// assert_ne!(TimeStamp::from(1), TimeStamp::Unknown);
/// assert!(TimeStamp::from(2) > TimeStamp::from(1));
/// ```
///
/// [`UNIX_EPOCH`]: std::time::UNIX_EPOCH
#[derive(Debug, Copy, Clone, PartialOrd, PartialEq)]
pub enum TimeStamp {
    Seconds(NonZeroU64),
    /// Value for unknown timestamp.
    ///
    /// In CLAP, this value is represented by the constant:
    /// [`CLAP_TIMESTAMP_UNKNOWN`] and is set to zero.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use clap_clap::timestamp::TimeStamp;
    /// use clap_clap::ffi::CLAP_TIMESTAMP_UNKNOWN;
    ///
    /// assert_eq!(TimeStamp::from(CLAP_TIMESTAMP_UNKNOWN), TimeStamp::Unknown);
    /// ```
    ///
    /// [`CLAP_TIMESTAMP_UNKNOWN`]: crate::ffi::CLAP_TIMESTAMP_UNKNOWN
    Unknown,
}

impl From<clap_timestamp> for TimeStamp {
    fn from(value: clap_timestamp) -> Self {
        match value {
            seconds if seconds > 0 => Self::Seconds(unsafe { NonZeroU64::new_unchecked(seconds) }),
            crate::ffi::CLAP_TIMESTAMP_UNKNOWN => Self::Unknown,
            // CLAP_TIMESTAMP_UNKNOWN is zero.
            _ => unreachable!(),
        }
    }
}

impl From<TimeStamp> for clap_timestamp {
    fn from(value: TimeStamp) -> Self {
        match value {
            TimeStamp::Seconds(seconds) => seconds.get(),
            TimeStamp::Unknown => crate::ffi::CLAP_TIMESTAMP_UNKNOWN,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ffi::{CLAP_TIMESTAMP_UNKNOWN, clap_timestamp};

    #[test]
    fn clap_timestamp_unknown_is_zero() {
        assert_eq!(CLAP_TIMESTAMP_UNKNOWN, 0);
    }

    #[test]
    fn clap_timestamp_is_u64() {
        let x: clap_timestamp = u64::MAX;
        assert!(x > 0)
    }
}
