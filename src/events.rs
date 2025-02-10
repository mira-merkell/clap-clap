//! Events and event lists.

use std::{
    fmt::{Display, Formatter},
    ptr::slice_from_raw_parts,
};

use crate::ffi::{CLAP_CORE_EVENT_SPACE_ID, CLAP_EVENT_MIDI, clap_event_header, clap_event_midi};

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
        if self.size() != size_of::<clap_event_midi>() as u32 {
            return Err(Error::PayloadSize(self.size()));
        }
        if self.r#type() != CLAP_EVENT_MIDI as u16 {
            return Err(Error::OtherType(self.r#type()));
        }
        Ok(unsafe { Midi::new_unchecked(self) })
    }
}

pub trait Event {
    fn header(&self) -> &Header;

    fn to_bytes(&self) -> &[u8] {
        self.header().to_bytes()
    }
}

pub trait EventBuilder {
    type Event<'a>: Event
    where
        Self: 'a;

    fn time(&mut self, value: u32);
    fn space_id(&mut self, value: u16);
    fn flags(&mut self, value: u32);

    fn event(&self) -> Self::Event<'_>;

    fn event_at(&mut self, time: u32) -> Self::Event<'_> {
        self.time(time);
        self.event()
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

    const fn as_clap_event_midi(&self) -> &clap_event_midi {
        unsafe { self.header.cast_unchecked() }
    }

    pub const fn port_index(&self) -> u16 {
        self.as_clap_event_midi().port_index
    }

    pub const fn data(&self) -> &[u8; 3] {
        &self.as_clap_event_midi().data
    }

    /// # Example
    ///
    /// ```rust
    /// # use clap_clap::events::{Event, EventBuilder,Midi};
    /// let mut midi = Midi::build().port_index(1);
    /// let event = midi.event_at(3);
    ///
    /// assert_eq!(event.port_index(), 1);
    /// assert_eq!(event.header().time(), 3);
    /// ```
    pub const fn build() -> MidiBuilder {
        MidiBuilder::new()
    }

    /// # Example
    ///
    /// ```rust
    /// # use clap_clap::events::{Event, EventBuilder,Midi};
    /// let midi = Midi::build().port_index(1).data([1, 2, 3]);
    /// let event = midi.event();
    ///
    /// let other_midi = event.update().data([4, 5, 6]);
    /// let other_event = other_midi.event();
    ///
    /// assert_eq!(event.port_index(), 1);
    /// assert_eq!(event.data(), &[1, 2, 3]);
    ///
    /// assert_eq!(other_event.port_index(), 1);
    /// assert_eq!(other_event.data(), &[4, 5, 6]);
    /// ```
    pub fn update(&self) -> MidiBuilder {
        MidiBuilder::with_midi(self)
    }
}

impl Event for Midi<'_> {
    fn header(&self) -> &Header {
        self.header
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct MidiBuilder(clap_event_midi);

impl MidiBuilder {
    pub const fn new() -> Self {
        Self(clap_event_midi {
            header: clap_event_header {
                size: size_of::<clap_event_midi>() as u32,
                time: 0,
                space_id: CLAP_CORE_EVENT_SPACE_ID,
                r#type: CLAP_EVENT_MIDI as u16,
                flags: 0,
            },
            port_index: 0,
            data: [0; 3],
        })
    }

    pub fn with_midi(midi: &Midi<'_>) -> Self {
        Self(*unsafe { midi.header().cast_unchecked() })
    }

    pub const fn port_index(self, value: u16) -> Self {
        let mut builder = self;
        builder.0.port_index = value;
        builder
    }

    pub const fn data(self, value: [u8; 3]) -> Self {
        let mut builder = self;
        builder.0.data = value;
        builder
    }
}

impl Default for MidiBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl EventBuilder for MidiBuilder {
    type Event<'a>
        = Midi<'a>
    where
        Self: 'a;

    fn time(&mut self, value: u32) {
        self.0.header.time = value;
    }

    fn space_id(&mut self, value: u16) {
        self.0.header.space_id = value;
    }

    fn flags(&mut self, value: u32) {
        self.0.header.flags = value;
    }

    fn event(&self) -> Self::Event<'_> {
        let header = unsafe { Header::new(&self.0.header) };
        unsafe { header.midi_unchecked() }
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
