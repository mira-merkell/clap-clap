//! Events and event lists.

use std::fmt::{Display, Formatter};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Error {
    UnknownEvent(u16),
    UnknownExpression(i32),
    OutOfOrder,
    TryPush,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::UnknownEvent(id) => {
                write!(f, "unknown event type: {id}")
            }
            Error::UnknownExpression(id) => {
                write!(f, "unknown note expression: {id}")
            }
            Error::OutOfOrder => {
                write!(f, "events must be inserted in the sample order")
            }
            Error::TryPush => {
                write!(f, "event could not be pushed to the queue")
            }
        }
    }
}

impl std::error::Error for Error {}

impl From<Error> for crate::Error {
    fn from(value: Error) -> Self {
        crate::Error::Events(value)
    }
}
