#[derive(Debug, Default, Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct BeatTime(pub i64);

impl BeatTime {
    const FACTOR: i64 = 1i64 << 31;

    pub const fn new(value: i64) -> Self {
        Self(value)
    }
}

#[derive(Debug, Default, Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct SecTime(pub i64);

impl SecTime {
    const FACTOR: i64 = 1i64 << 31;

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
                (value.0 / <$Typ>::FACTOR) as _
            }
        }
    };
}

impl_to_from_f64!(BeatTime);
impl_to_from_f64!(SecTime);
