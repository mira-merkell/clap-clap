//! Fixed point representation of beat time and seconds time.
//!
//! The representation uses 31 least significant bits for the fractional part.
//!
//! # Example
//!
//! ```rust
//! # use clap_clap::fixedpoint::BeatTime;
//! let x = 1.330f64;
//! let beats = BeatTime::from(x);
//!
//! assert_eq!(beats.0, (x * BeatTime::FACTOR as f64).round() as i64);
//! ```
//!
//! The constants: [`BeatTime::FACTOR`] and [`SecTime::FACTOR`] are set to:
//! `2^31`:
//!
//! ```rust
//! # use clap_clap::fixedpoint::{BeatTime, SecTime};
//! assert_eq!(BeatTime::FACTOR, 1i64 << 31);
//! assert_eq!(SecTime::FACTOR, 1i64 << 31);
//! ```
//!
//! Hence, the error of converting a `f64` to `BeatTime` or `SecTime` and back
//! is not going to exceed `2^(-31) ~ 0.0000000004656613`.
//!
//! ```rust
//! # use clap_clap::fixedpoint::SecTime;
//! let x = 17.932f64;
//! let y = f64::from(SecTime::from(x));
//!
//! assert!(f64::abs(x - y) < (SecTime::FACTOR as f64).recip());
//! ```
//!
//! See also: <https://github.com/free-audio/clap/blob/main/include/clap/fixedpoint.h>.
//!
//! [`BeatTime::FACTOR`]: crate::fixedpoint::BeatTime::FACTOR
//! [`SecTime::FACTOR`]: crate::fixedpoint::SecTime::FACTOR

/// Time in beats.
///
/// # Example
///
/// ```rust
/// # use clap_clap::fixedpoint::BeatTime;
/// let beats = BeatTime::from(8.23);
///
/// assert!(beats > BeatTime::from(1.034));
/// ```
#[derive(Debug, Default, Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct BeatTime(pub i64);

impl BeatTime {
    pub const FACTOR: i64 = 1i64 << 31;

    pub const fn new(value: i64) -> Self {
        Self(value)
    }
}

/// Time in seconds.
///
/// # Example
///
/// ```rust
/// # use clap_clap::fixedpoint::SecTime;
/// let secs = SecTime::from(11.03);
///
/// assert!(secs < SecTime::from(99.7));
/// ```
#[derive(Debug, Default, Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct SecTime(pub i64);

impl SecTime {
    pub const FACTOR: i64 = 1i64 << 31;

    pub const fn new(value: i64) -> Self {
        Self(value)
    }
}

macro_rules! impl_to_from_f64 {
    ($Typ:ty) => {
        impl From<f64> for $Typ {
            fn from(value: f64) -> Self {
                Self((value * Self::FACTOR as f64).round() as _)
            }
        }

        impl From<$Typ> for f64 {
            fn from(value: $Typ) -> Self {
                (value.0 as f64) / (<$Typ>::FACTOR as f64)
            }
        }
    };
}

impl_to_from_f64!(BeatTime);
impl_to_from_f64!(SecTime);
