pub use crate::ffi::{
    CLAP_BEATTIME_FACTOR as BEATTIME_FACTOR, CLAP_SECTIME_FACTOR as SECTIME_FACTOR,
};
use crate::ffi::{clap_beattime, clap_sectime};

#[repr(transparent)]
pub struct BeatTime(clap_beattime);

impl BeatTime {
    pub const fn new(value: i64) -> Self {
        Self(value)
    }
}

#[repr(transparent)]
pub struct SecTime(clap_sectime);

impl SecTime {
    pub const fn new(value: i64) -> Self {
        Self(value)
    }
}
