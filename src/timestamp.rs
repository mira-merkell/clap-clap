use std::num::{NonZero, NonZeroU64};

use crate::ffi::clap_timestamp;

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq)]
pub enum TimeStamp {
    Seconds(NonZeroU64),
    Unknown,
}

impl From<clap_timestamp> for TimeStamp {
    fn from(value: clap_timestamp) -> Self {
        match value {
            seconds if seconds > 0 => Self::Seconds(unsafe { NonZero::new_unchecked(seconds) }),
            crate::ffi::CLAP_TIMESTAMP_UNKNOWN => Self::Unknown,
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
