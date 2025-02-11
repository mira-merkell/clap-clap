//! Events and event lists.

use std::{
    fmt::{Display, Formatter},
    ptr::slice_from_raw_parts,
};

use crate::ffi::{
    CLAP_CORE_EVENT_SPACE_ID, CLAP_EVENT_MIDI, clap_event_header, clap_event_midi,
    clap_input_events,
};

#[derive(Debug, PartialEq)]
pub struct Header([u8]);

impl Header {
    /// # Safety
    ///
    /// The reference must point to a valid header of a CLAP event. i.e.
    ///
    /// 1. The header's field: `size` must indicate the correct size of the
    ///    event.
    /// 2. The header's field: `type` must indicate the correct type of the
    ///    event.
    /// 3. The entire memory block of size `header.size`, starting from the
    ///    address of `header` must hold a properly initialized and aligned
    ///    object whose type is inferred from `header.type`.
    ///
    /// This is to make possible to cast `&raw const *header` pointer to
    /// a pointer to the concrete event type.
    pub const unsafe fn new(header: &clap_event_header) -> &Self {
        let len = header.size as usize;
        let data = &raw const *header as *const _;
        unsafe { &*(slice_from_raw_parts::<u8>(data, len) as *const _) }
    }

    /// # Safety
    ///
    /// The caller must ensure that the cast to a reference of type `T` is safe,
    /// i.e. that the header and the payload hold together a properly
    /// initialized and aligned object of type `T`.
    const unsafe fn cast_unchecked<T>(&self) -> &T {
        unsafe { &*self.0.as_ptr().cast() }
    }

    pub const fn as_clap_event_header(&self) -> &clap_event_header {
        // SAFETY: By construction, a cast to `clap_event_header` from with `self` was
        // created is safe.
        unsafe { self.cast_unchecked() }
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

    /// # Safety
    ///
    /// The caller must ensure that this `Header` has correct size and type to
    /// contain the header and the payload of event of type: `clap_event_midi`.
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
        // SAFETY: We just checked if `self` is a event of type: `clap_event_midi`.
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

    fn time(self, value: u32) -> Self;
    fn space_id(self, value: u16) -> Self;
    fn flags(self, value: u32) -> Self;

    fn event(&self) -> Self::Event<'_>;
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Midi<'a> {
    header: &'a Header,
}

impl<'a> Midi<'a> {
    /// # Safety
    ///
    /// The `header` must be a header of type: `clap_event_midi`.
    pub const unsafe fn new_unchecked(header: &'a Header) -> Self {
        Self { header }
    }

    const fn as_clap_event_midi(&self) -> &clap_event_midi {
        // SAFETY: By construction, this cast is safe.
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
    /// let midi = Midi::build().port_index(1).time(3);
    /// let event = midi.event();
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
        // SAFETY: Midi constructor guarantees that this cast is safe, and we can copy
        // the object of type: `clap_event_midi`.
        Self(*unsafe { midi.header().cast_unchecked() })
    }

    pub const fn port_index(self, value: u16) -> Self {
        let mut build = self;
        build.0.port_index = value;
        build
    }

    pub const fn data(self, value: [u8; 3]) -> Self {
        let mut build = self;
        build.0.data = value;
        build
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

    fn time(self, value: u32) -> Self {
        let mut build = self;
        build.0.header.time = value;
        build
    }

    fn space_id(self, value: u16) -> Self {
        let mut build = self;
        build.0.header.space_id = value;
        build
    }

    fn flags(self, value: u32) -> Self {
        let mut build = self;
        build.0.header.flags = value;
        build
    }

    fn event(&self) -> Self::Event<'_> {
        // SAFETY: By construction, `self.header` is a valid header of type:
        // `clap_event_midi`.
        let header = unsafe { Header::new(&self.0.header) };
        unsafe { header.midi_unchecked() }
    }
}

pub struct InputEvents<'a>(&'a clap_input_events);

impl<'a> InputEvents<'a> {
    /// # Safety
    ///
    /// The pointers: `list.size` and `list.get` must be Some.
    #[doc(hidden)]
    pub const unsafe fn new_unchecked(list: &'a clap_input_events) -> Self {
        Self(list)
    }

    pub fn size(&self) -> u32 {
        // SAFETY: By construction, the pointer `self.size` is Some.
        unsafe { self.0.size.unwrap()(self.0) }
    }

    /// # Safety
    ///
    /// The value of `index` must be less than `self.size()`.
    pub unsafe fn get_unchecked(&self, index: u32) -> &Header {
        // SAFETY: By construction, the pointer `self.get` is Some.
        let header = unsafe { &*self.0.get.unwrap()(self.0, index) };
        unsafe { Header::new(header) }
    }

    /// # Panic
    ///
    /// Panic if `index` greater or equal than `self.size()`.
    pub fn get(&self, index: u32) -> &Header {
        assert!(index < self.size(), "index out of bounds");
        unsafe { self.get_unchecked(index) }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Error {
    OtherType(u16),
    PayloadSize(u32),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::PayloadSize(size) => {
                write!(f, "payload size for the defined event type: {size}")
            }
            Error::OtherType(id) => write!(f, "other type, id: {id}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<Error> for crate::Error {
    fn from(value: Error) -> Self {
        crate::Error::Events(value)
    }
}
