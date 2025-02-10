//! Events and event lists.

use std::{
    fmt::{Display, Formatter},
    ptr::slice_from_raw_parts,
};

use crate::ffi::{CLAP_EVENT_MIDI, clap_event_header, clap_event_midi};

#[derive(Debug, PartialEq)]
pub struct Header([u8]);

impl Header {
    pub const unsafe fn new(header: &clap_event_header) -> &Self {
        let len = header.size as usize;
        let data = &raw const *header as *const _;
        unsafe { &*(slice_from_raw_parts::<u8>(data, len) as *const _) }
    }

    pub const fn as_clap_event_header(&self) -> &clap_event_header {
        unsafe { &*(self.0.as_ptr() as *const _) }
    }

    const unsafe fn cast_unchecked<T>(&self) -> &T {
        unsafe { &*self.0.as_ptr().cast() }
    }

    const fn to_bytes(&self) -> &[u8] {
        &self.0
    }

    pub const fn flags(&self) -> u32 {
        self.as_clap_event_header().flags
    }

    pub const fn size(&self) -> u32 {
        self.0.len() as u32
    }

    pub const fn space_id(&self) -> u16 {
        self.as_clap_event_header().space_id
    }

    pub const fn time(&self) -> u32 {
        self.as_clap_event_header().time
    }

    pub const fn r#type(&self) -> u16 {
        self.as_clap_event_header().r#type
    }

    pub const unsafe fn midi_unchecked(&self) -> Midi {
        unsafe { Midi::new_unchecked(self) }
    }

    pub const fn midi(&self) -> Result<Midi, Error> {
        Midi::try_new(self)
    }
}

pub trait Event {
    fn header(&self) -> &Header;

    fn to_bytes(&self) -> &[u8] {
        self.header().to_bytes()
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Midi<'a> {
    header: &'a Header,
}

impl<'a> Midi<'a> {
    pub const unsafe fn new_unchecked(header: &'a Header) -> Self {
        Self { header }
    }

    pub const fn try_new(header: &'a Header) -> Result<Self, Error> {
        if header.size() != size_of::<clap_event_midi>() as u32 {
            return Err(Error::PayloadSize(header.size()));
        }
        if header.r#type() != CLAP_EVENT_MIDI as u16 {
            return Err(Error::OtherType(header.r#type()));
        }
        Ok(unsafe { Self::new_unchecked(header) })
    }

    const fn as_clap_event_midi(&self) -> &clap_event_midi {
        unsafe { self.header.cast_unchecked() }
    }

    pub const fn port_index(&self) -> u16 {
        self.as_clap_event_midi().port_index
    }

    pub const fn data(&self) -> &[u8; 3] {
        &self.as_clap_event_midi().data
    }
}

impl Event for Midi<'_> {
    fn header(&self) -> &Header {
        self.header
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Error {
    UnknownEvent(u16),
    UnknownExpression(i32),
    OtherType(u16),
    PayloadSize(u32),
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
            Error::PayloadSize(size) => {
                write!(f, "wrong payload size for the defined event type: {size}")
            }
            Error::OtherType(_) => todo!(),
        }
    }
}

impl std::error::Error for Error {}

impl From<Error> for crate::Error {
    fn from(value: Error) -> Self {
        crate::Error::Events(value)
    }
}
