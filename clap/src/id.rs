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

impl ClapId {
    pub fn invalid_id() -> Self {
        Self(None)
    }

    pub fn is_valid(&self) -> bool {
        self.0.is_some()
    }
}

impl TryFrom<u32> for ClapId {
    type Error = Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        (value != clap_sys::CLAP_INVALID_ID)
            .then_some(Self(Some(value)))
            .ok_or(Error::InvalidId)
    }
}

impl TryFrom<i32> for ClapId {
    type Error = Error;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        if (value >= 0) {
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

impl From<ClapId> for clap_sys::clap_id {
    fn from(value: ClapId) -> Self {
        value.0.unwrap_or(clap_sys::CLAP_INVALID_ID)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid() {
        let _ = ClapId::try_from(0).unwrap();
        let _ = ClapId::try_from(10).unwrap();
        let _ = ClapId::try_from(1000).unwrap();
        let _ = ClapId::try_from(10000000).unwrap();
        let _ = ClapId::try_from(u32::MAX - 1).unwrap();
    }

    #[test]
    fn invalid() {
        let _ = ClapId::try_from(u32::MAX).unwrap_err();
        let _ = ClapId::try_from(usize::try_from(1u64 << 33).unwrap()).unwrap_err();
        let _ = ClapId::try_from(-1).unwrap_err();
        let _ = ClapId::try_from(-10).unwrap_err();
    }

    #[test]
    fn is_valid() {
        assert!(ClapId::from(0).is_valid());
        assert!(ClapId::from(10).is_valid());
        assert!(ClapId::from(100).is_valid());
        assert!(ClapId::from(1000).is_valid());
        assert!(ClapId::from(10000).is_valid());

        assert!(!ClapId::invalid_id().is_valid());
    }

    #[test]
    fn invalid_is_max() {
        assert_eq!(clap_sys::CLAP_INVALID_ID, ClapId::invalid_id().into());
        assert_eq!(u32::MAX, ClapId::invalid_id().into());
    }
}
