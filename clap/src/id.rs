use std::fmt::{Display, Formatter};

#[derive(Debug, Copy, Clone)]
pub enum Error {
    InvalidId,
    Overflow,
    Underflow,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidId => write!(f, "Invalid ID"),
            Error::Overflow => write!(f, "Overflow"),
            Error::Underflow => write!(f, "Underflow"),
        }
    }
}

impl std::error::Error for Error {}

#[derive(Default, Debug, Copy, Clone, Hash, PartialEq)]
pub struct ClapId(Option<u32>);

impl TryFrom<u32> for ClapId {
    type Error = Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        (value != clap_sys::CLAP_INVALID_ID)
            .then_some(Self(Some(value)))
            .ok_or(Self::Error::InvalidId)
    }
}

impl TryFrom<i32> for ClapId {
    type Error = Error;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        (value >= 0)
            .then_some((value as u32).try_into())
            .unwrap_or(Err(Error::Underflow))
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

impl From<ClapId> for clap_sys::clap_id {
    fn from(value: ClapId) -> Self {
        value.0.unwrap_or(clap_sys::CLAP_INVALID_ID)
    }
}
