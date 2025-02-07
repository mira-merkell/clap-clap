use std::fmt::{Display, Formatter};

use crate::clap::{self, CLAP_INVALID_ID};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Error {
    InvalidId,
    Overflow,
    Underflow,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidId => write!(f, "invalid ID"),
            Error::Overflow => write!(f, "overflow during type conversion"),
            Error::Underflow => write!(f, "underflow during type conversion"),
        }
    }
}

impl std::error::Error for Error {}

/// A type that corresponds to CLAP's `clap_id`.
///
/// It is either a `u32` value less that [`u32::MAX`], or a value that
/// represents an invalid id.
///
/// # Example
///
/// ```rust
/// # use clap_clap::id::ClapId;
/// let id = ClapId::from(3);
///
/// assert!(id.is_valid());
/// assert_ne!(id, ClapId::invalid_id());
/// ```
#[derive(Default, Debug, Copy, Clone, Hash, PartialEq)]
pub struct ClapId(Option<u32>);

impl ClapId {
    /// The value representing an invalid id.
    pub const fn invalid_id() -> Self {
        Self(None)
    }

    pub const fn is_valid(&self) -> bool {
        self.0.is_some()
    }
}

impl TryFrom<u32> for ClapId {
    type Error = Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        (value != CLAP_INVALID_ID)
            .then_some(Self(Some(value)))
            .ok_or(Error::InvalidId)
    }
}

impl TryFrom<i32> for ClapId {
    type Error = Error;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        if value >= 0 {
            (value as u32).try_into()
        } else {
            Err(Error::Underflow)
        }
    }
}

impl From<u16> for ClapId {
    fn from(value: u16) -> Self {
        Self(Some(value as u32))
    }
}

impl TryFrom<usize> for ClapId {
    type Error = Error;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match u32::try_from(value).map_err(|_| Error::Overflow) {
            Ok(v) => v.try_into(),
            Err(e) => Err(e),
        }
    }
}

impl From<ClapId> for clap::clap_id {
    fn from(value: ClapId) -> Self {
        value.0.unwrap_or(clap::CLAP_INVALID_ID)
    }
}
