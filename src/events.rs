//! Events and event lists.

use std::{
    ffi::c_void,
    fmt::{Display, Formatter},
    ptr::{null_mut, slice_from_raw_parts},
};

use crate::{
    ffi::{
        CLAP_CORE_EVENT_SPACE_ID, CLAP_EVENT_MIDI, CLAP_EVENT_MIDI2, CLAP_EVENT_PARAM_VALUE,
        clap_event_header, clap_event_midi, clap_event_midi2, clap_event_param_value,
        clap_input_events,
    },
    id::ClapId,
};

macro_rules! impl_event_cast_methods {
    ($name:tt, $name_unchk:tt, $type:ty, $cast_type:ty, $clap_id:ident $(,)?) => {
        /// # Safety
        ///
        #[doc = concat!("The caller must ensure that this `Header` has correct size and type to contain the header and the payload of event of the returned type: `", stringify!($name), "`.")]
        pub const unsafe fn $name_unchk(&self) -> $type {
            unsafe { <$type>::new_unchecked(self) }
        }

        pub const fn $name(&self) -> Result<$type, Error> {
            if self.r#type() != $clap_id as u16 {
                return Err(Error::OtherType(self.r#type()));
            }
            if self.size() != size_of::<$cast_type>() as u32 {
                return Err(Error::PayloadSize(self.size()));
            }
            // SAFETY: We just checked if `self` is a event of type to be cast to.
            Ok(unsafe { <$type>::new_unchecked(self) })
        }
    };
}

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

    impl_event_cast_methods!(
        param_value,
        param_value_unchecked,
        ParamValue,
        clap_event_param_value,
        CLAP_EVENT_PARAM_VALUE
    );
    impl_event_cast_methods!(midi, midi_unchecked, Midi, clap_event_midi, CLAP_EVENT_MIDI);
    impl_event_cast_methods!(
        midi2,
        midi2_unchecked,
        Midi2,
        clap_event_midi2,
        CLAP_EVENT_MIDI2
    );
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

macro_rules! impl_event_const_getter {
    ($name:tt, $cast_method:ident, $type:ty $(,)?) => {
        pub const fn $name(&self) -> $type {
            self.$cast_method().$name
        }
    };
}

macro_rules! impl_event_builder_setter {
    ($name:tt, $type:ty $(,)*) => {
        pub const fn $name(self, value: $type) -> Self {
            let mut build = self;
            build.0.$name = value;
            build
        }
    };
}

macro_rules! impl_event_builder {
    ($type:ty, $event_type:ty, $cast_method:ident $(,)?) => {
        impl EventBuilder for $type {
            type Event<'a>
                = $event_type
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
                // SAFETY: By construction, `self.header` is a valid header of the event type.
                let header = unsafe { Header::new(&self.0.header) };
                unsafe { header.$cast_method() }
            }
        }
    };
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ParamValue<'a> {
    header: &'a Header,
}

impl<'a> ParamValue<'a> {
    /// # Safety
    ///
    /// The `header` must be a header of type: `clap_event_param_value`.
    pub const unsafe fn new_unchecked(header: &'a Header) -> Self {
        Self { header }
    }

    const fn as_clap_event_param_value(&self) -> &clap_event_param_value {
        // SAFETY: By construction, this cast is safe.
        unsafe { self.header.cast_unchecked() }
    }

    impl_event_const_getter!(channel, as_clap_event_param_value, i16);
    impl_event_const_getter!(cookie, as_clap_event_param_value, *mut c_void);
    impl_event_const_getter!(key, as_clap_event_param_value, i16);
    impl_event_const_getter!(note_id, as_clap_event_param_value, i32);

    pub fn param_id(&self) -> ClapId {
        self.as_clap_event_param_value()
            .param_id
            .try_into()
            .unwrap_or(ClapId::invalid_id())
    }

    impl_event_const_getter!(port_index, as_clap_event_param_value, i16);
    impl_event_const_getter!(value, as_clap_event_param_value, f64);

    pub const fn build() -> ParamValueBuilder {
        ParamValueBuilder::new()
    }

    pub fn update(&self) -> ParamValueBuilder {
        ParamValueBuilder::with_param_value(self)
    }
}

impl Event for ParamValue<'_> {
    fn header(&self) -> &Header {
        self.header
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ParamValueBuilder(clap_event_param_value);

impl ParamValueBuilder {
    pub const fn new() -> Self {
        Self(clap_event_param_value {
            header: clap_event_header {
                size: size_of::<clap_event_param_value>() as u32,
                time: 0,
                space_id: CLAP_CORE_EVENT_SPACE_ID,
                r#type: CLAP_EVENT_PARAM_VALUE as u16,
                flags: 0,
            },
            param_id: 0,
            cookie: null_mut(),
            note_id: 0,
            port_index: 0,
            channel: 0,
            key: 0,
            value: 0.0,
        })
    }

    pub fn with_param_value(param_value: &ParamValue<'_>) -> Self {
        // SAFETY: ParamValue constructor guarantees that this cast is safe, and we can
        // copy the object of type: `clap_event_param_value`.
        Self(*unsafe { param_value.header().cast_unchecked() })
    }

    impl_event_builder_setter!(port_index, i16);
    impl_event_builder_setter!(channel, i16);
    impl_event_builder_setter!(cookie, *mut c_void);
    impl_event_builder_setter!(key, i16);
    impl_event_builder_setter!(note_id, i32);

    pub fn param_id(self, value: ClapId) -> Self {
        let mut builder = self;
        builder.0.param_id = value.into();
        builder
    }

    impl_event_builder_setter!(value, f64);
}

impl Default for ParamValueBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl_event_builder!(ParamValueBuilder, ParamValue<'a>, param_value_unchecked);

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Midi2<'a> {
    header: &'a Header,
}

impl<'a> Midi2<'a> {
    /// # Safety
    ///
    /// The `header` must be a header of type: `clap_event_midi2`.
    pub const unsafe fn new_unchecked(header: &'a Header) -> Self {
        Self { header }
    }

    const fn as_clap_event_midi2(&self) -> &clap_event_midi2 {
        // SAFETY: By construction, this cast is safe.
        unsafe { self.header.cast_unchecked() }
    }

    impl_event_const_getter!(port_index, as_clap_event_midi2, u16);

    pub const fn data(&self) -> &[u32; 4] {
        &self.as_clap_event_midi2().data
    }

    /// # Example
    ///
    /// ```rust
    /// # use clap_clap::events::{Event, EventBuilder,Midi2};
    /// let midi = Midi2::build().port_index(1).time(3);
    /// let event = midi.event();
    ///
    /// assert_eq!(event.port_index(), 1);
    /// assert_eq!(event.header().time(), 3);
    /// ```
    pub const fn build() -> Midi2Builder {
        Midi2Builder::new()
    }

    /// # Example
    ///
    /// ```rust
    /// # use clap_clap::events::{Event, EventBuilder,Midi2};
    /// let midi = Midi2::build().port_index(1).data([1, 2, 3, 4]);
    /// let event = midi.event();
    ///
    /// let other_midi = event.update().data([4, 5, 6, 7]);
    /// let other_event = other_midi.event();
    ///
    /// assert_eq!(event.port_index(), 1);
    /// assert_eq!(event.data(), &[1, 2, 3, 4]);
    ///
    /// assert_eq!(other_event.port_index(), 1);
    /// assert_eq!(other_event.data(), &[4, 5, 6, 7]);
    /// ```
    pub fn update(&self) -> Midi2Builder {
        Midi2Builder::with_midi2(self)
    }
}

impl Event for Midi2<'_> {
    fn header(&self) -> &Header {
        self.header
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Midi2Builder(clap_event_midi2);

impl Midi2Builder {
    pub const fn new() -> Self {
        Self(clap_event_midi2 {
            header: clap_event_header {
                size: size_of::<clap_event_midi2>() as u32,
                time: 0,
                space_id: CLAP_CORE_EVENT_SPACE_ID,
                r#type: CLAP_EVENT_MIDI2 as u16,
                flags: 0,
            },
            port_index: 0,
            data: [0; 4],
        })
    }

    pub fn with_midi2(midi: &Midi2<'_>) -> Self {
        // SAFETY: Midi constructor guarantees that this cast is safe, and we can copy
        // the object of type: `clap_event_midi2`.
        Self(*unsafe { midi.header().cast_unchecked() })
    }

    impl_event_builder_setter!(port_index, u16);
    impl_event_builder_setter!(data, [u32; 4]);
}

impl Default for Midi2Builder {
    fn default() -> Self {
        Self::new()
    }
}

impl_event_builder!(Midi2Builder, Midi2<'a>, midi2_unchecked);

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

    impl_event_const_getter!(port_index, as_clap_event_midi, u16);

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

    impl_event_builder_setter!(port_index, u16);
    impl_event_builder_setter!(data, [u8; 3]);
}

impl Default for MidiBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl_event_builder!(MidiBuilder, Midi<'a>, midi_unchecked);

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
