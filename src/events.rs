//! Events and event lists.

use std::{
    ffi::c_void,
    fmt::{Display, Formatter},
    ptr::{null_mut, slice_from_raw_parts},
};

use crate::{
    ffi::{
        CLAP_CORE_EVENT_SPACE_ID, CLAP_EVENT_MIDI, CLAP_EVENT_MIDI2, CLAP_EVENT_PARAM_MOD,
        CLAP_EVENT_PARAM_VALUE, CLAP_EVENT_TRANSPORT, CLAP_TRANSPORT_HAS_BEATS_TIMELINE,
        CLAP_TRANSPORT_HAS_SECONDS_TIMELINE, CLAP_TRANSPORT_HAS_TEMPO,
        CLAP_TRANSPORT_HAS_TIME_SIGNATURE, CLAP_TRANSPORT_IS_LOOP_ACTIVE,
        CLAP_TRANSPORT_IS_PLAYING, CLAP_TRANSPORT_IS_RECORDING, CLAP_TRANSPORT_IS_WITHIN_PRE_ROLL,
        clap_event_header, clap_event_midi, clap_event_midi2, clap_event_param_mod,
        clap_event_param_value, clap_event_transport, clap_input_events, clap_output_events,
    },
    fixedpoint::{BeatTime, SecTime},
    id::ClapId,
    impl_flags_u32,
};

macro_rules! impl_event_cast_methods {
    ($name:tt, $name_unchk:tt, $type:ty, $cast_type:ty, $clap_id:ident $(,)?) => {
        /// # Safety
        #[doc = concat!("The caller must ensure that this `Header` has correct \
            size and type to contain the header and the payload of event of the \
            returned type: `", stringify!($name), "`.")
                                                                                                ]
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

    #[doc(hidden)]
    pub const fn to_bytes(&self) -> &[u8] {
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

    impl_event_cast_methods!(
        param_mod,
        param_mod_unchecked,
        ParamMod,
        clap_event_param_mod,
        CLAP_EVENT_PARAM_MOD
    );

    impl_event_cast_methods!(
        transport,
        transport_unchecked,
        Transport,
        clap_event_transport,
        CLAP_EVENT_TRANSPORT
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

impl From<clap_event_param_value> for ParamValueBuilder {
    fn from(value: clap_event_param_value) -> Self {
        Self(value)
    }
}

impl_event_builder!(ParamValueBuilder, ParamValue<'a>, param_value_unchecked);

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ParamMod<'a> {
    header: &'a Header,
}

impl<'a> ParamMod<'a> {
    /// # Safety
    ///
    /// The `header` must be a header of type: `clap_event_param_mod`.
    pub const unsafe fn new_unchecked(header: &'a Header) -> Self {
        Self { header }
    }

    const fn as_clap_event_param_mod(&self) -> &clap_event_param_mod {
        // SAFETY: By construction, this cast is safe.
        unsafe { self.header.cast_unchecked() }
    }

    impl_event_const_getter!(channel, as_clap_event_param_mod, i16);
    impl_event_const_getter!(cookie, as_clap_event_param_mod, *mut c_void);
    impl_event_const_getter!(key, as_clap_event_param_mod, i16);
    impl_event_const_getter!(note_id, as_clap_event_param_mod, i32);

    pub fn param_id(&self) -> ClapId {
        self.as_clap_event_param_mod()
            .param_id
            .try_into()
            .unwrap_or(ClapId::invalid_id())
    }

    impl_event_const_getter!(port_index, as_clap_event_param_mod, i16);
    impl_event_const_getter!(amount, as_clap_event_param_mod, f64);

    pub const fn build() -> ParamModBuilder {
        ParamModBuilder::new()
    }

    pub fn update(&self) -> ParamModBuilder {
        ParamModBuilder::with_param_mod(self)
    }
}

impl Event for ParamMod<'_> {
    fn header(&self) -> &Header {
        self.header
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ParamModBuilder(clap_event_param_mod);

impl ParamModBuilder {
    pub const fn new() -> Self {
        Self(clap_event_param_mod {
            header: clap_event_header {
                size: size_of::<clap_event_param_mod>() as u32,
                time: 0,
                space_id: CLAP_CORE_EVENT_SPACE_ID,
                r#type: CLAP_EVENT_PARAM_MOD as u16,
                flags: 0,
            },
            param_id: 0,
            cookie: null_mut(),
            note_id: 0,
            port_index: 0,
            channel: 0,
            key: 0,
            amount: 0.0,
        })
    }

    pub fn with_param_mod(param_mod: &ParamMod<'_>) -> Self {
        // SAFETY: ParamValue constructor guarantees that this cast is safe, and we can
        // copy the object of type: `clap_event_param_mod`.
        Self(*unsafe { param_mod.header().cast_unchecked() })
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

    impl_event_builder_setter!(amount, f64);
}

impl Default for ParamModBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl From<clap_event_param_mod> for ParamModBuilder {
    fn from(value: clap_event_param_mod) -> Self {
        Self(value)
    }
}

impl_event_builder!(ParamModBuilder, ParamMod<'a>, param_mod_unchecked);

/// Transport flags.
///
/// # Example
///
/// ```rust
/// # use clap_clap::events::TransportFlags;
/// assert_eq!(TransportFlags::HasTempo as u32, 0b1);
/// assert!(TransportFlags::HasTempo.is_set(0b101));
/// assert_eq!(TransportFlags::HasTempo.set(0b100), 0b101);
/// assert_eq!(TransportFlags::HasTempo.clear(0b101), 0b100);
/// ```
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u32)]
pub enum TransportFlags {
    HasTempo = CLAP_TRANSPORT_HAS_TEMPO,
    HasBeatsTimeline = CLAP_TRANSPORT_HAS_BEATS_TIMELINE,
    HasSecondsTimeline = CLAP_TRANSPORT_HAS_SECONDS_TIMELINE,
    HasTimeSignature = CLAP_TRANSPORT_HAS_TIME_SIGNATURE,
    IsPlaying = CLAP_TRANSPORT_IS_PLAYING,
    IsRecording = CLAP_TRANSPORT_IS_RECORDING,
    IsLoopActive = CLAP_TRANSPORT_IS_LOOP_ACTIVE,
    IsWithinPreRoll = CLAP_TRANSPORT_IS_WITHIN_PRE_ROLL,
}

impl_flags_u32!(TransportFlags);

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Transport<'a> {
    header: &'a Header,
}

macro_rules! impl_transport_time_getter {
    ($name:tt, $type:tt $(,)?) => {
        pub const fn $name(&self) -> $type {
            $type(self.as_clap_event_transport().$name)
        }
    };
}

impl<'a> Transport<'a> {
    /// # Safety
    ///
    /// The `header` must be a header of type: `clap_event_transport`.
    pub const unsafe fn new_unchecked(header: &'a Header) -> Self {
        Self { header }
    }

    const fn as_clap_event_transport(&self) -> &clap_event_transport {
        // SAFETY: By construction, this cast is safe.
        unsafe { self.header.cast_unchecked() }
    }

    impl_transport_time_getter!(song_pos_seconds, SecTime);
    impl_transport_time_getter!(song_pos_beats, BeatTime);
    impl_transport_time_getter!(loop_start_beats, BeatTime);
    impl_transport_time_getter!(loop_end_beats, BeatTime);
    impl_transport_time_getter!(loop_start_seconds, SecTime);
    impl_transport_time_getter!(loop_end_seconds, SecTime);
    impl_transport_time_getter!(bar_start, BeatTime);

    impl_event_const_getter!(flags, as_clap_event_transport, u32);
    impl_event_const_getter!(tempo, as_clap_event_transport, f64);
    impl_event_const_getter!(tempo_inc, as_clap_event_transport, f64);
    impl_event_const_getter!(bar_number, as_clap_event_transport, i32);
    impl_event_const_getter!(tsig_num, as_clap_event_transport, u16);
    impl_event_const_getter!(tsig_denom, as_clap_event_transport, u16);

    pub const fn build() -> TransportBuilder {
        TransportBuilder::new()
    }

    pub fn update(&self) -> TransportBuilder {
        TransportBuilder::with_transport(self)
    }
}

impl Event for Transport<'_> {
    fn header(&self) -> &Header {
        self.header
    }
}

macro_rules! impl_transport_time_setter {
    ($name:tt, $type:tt $(,)?) => {
        pub const fn $name(self, value: $type) -> Self {
            let mut build = self;
            build.0.$name = value.0;
            build
        }
    };
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct TransportBuilder(clap_event_transport);

impl TransportBuilder {
    pub const fn new() -> Self {
        Self(clap_event_transport {
            header: clap_event_header {
                size: size_of::<clap_event_transport>() as u32,
                time: 0,
                space_id: CLAP_CORE_EVENT_SPACE_ID,
                r#type: CLAP_EVENT_TRANSPORT as u16,
                flags: 0,
            },
            flags: 0,
            song_pos_beats: 0,
            song_pos_seconds: 0,
            tempo: 0.0,
            tempo_inc: 0.0,
            loop_start_beats: 0,
            loop_end_beats: 0,
            loop_start_seconds: 0,
            loop_end_seconds: 0,
            bar_start: 0,
            bar_number: 0,
            tsig_num: 0,
            tsig_denom: 0,
        })
    }

    pub fn with_transport(param_value: &Transport<'_>) -> Self {
        // SAFETY: Transport constructor guarantees that this cast is safe, and we can
        // copy the object of type: `clap_event_transport`.
        Self(*unsafe { param_value.header().cast_unchecked() })
    }

    impl_transport_time_setter!(song_pos_seconds, SecTime);
    impl_transport_time_setter!(song_pos_beats, BeatTime);
    impl_transport_time_setter!(loop_start_beats, BeatTime);
    impl_transport_time_setter!(loop_end_beats, BeatTime);
    impl_transport_time_setter!(loop_start_seconds, SecTime);
    impl_transport_time_setter!(loop_end_seconds, SecTime);
    impl_transport_time_setter!(bar_start, BeatTime);

    impl_event_builder_setter!(flags, u32);
    impl_event_builder_setter!(tempo, f64);
    impl_event_builder_setter!(tempo_inc, f64);
    impl_event_builder_setter!(bar_number, i32);
    impl_event_builder_setter!(tsig_num, u16);
    impl_event_builder_setter!(tsig_denom, u16);
}

impl Default for TransportBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl From<clap_event_transport> for TransportBuilder {
    fn from(value: clap_event_transport) -> Self {
        Self(value)
    }
}

impl_event_builder!(TransportBuilder, Transport<'a>, transport_unchecked);

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

impl From<clap_event_midi2> for Midi2Builder {
    fn from(value: clap_event_midi2) -> Self {
        Self(value)
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

impl From<clap_event_midi> for MidiBuilder {
    fn from(value: clap_event_midi) -> Self {
        Self(value)
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

pub struct OutputEvents<'a> {
    list: &'a clap_output_events,
    last_time: u32,
}

impl<'a> OutputEvents<'a> {
    /// # Safety
    ///
    /// The pointer: `list.try_push` must be Some.
    #[doc(hidden)]
    pub const unsafe fn new_unchecked(list: &'a clap_output_events) -> Self {
        Self { list, last_time: 0 }
    }

    /// # Safety
    ///
    /// Events must be pushed in sample time order, i.e. `event.header().time()`
    /// must not be smaller than the time of the previous event successfully
    /// pushed.
    pub unsafe fn try_push_unchecked(&mut self, event: impl Event) -> Result<(), Error> {
        let time = event.header().time();

        if unsafe { self.list.try_push.unwrap()(self.list, event.header().as_clap_event_header()) }
        {
            self.last_time = time;
            Ok(())
        } else {
            Err(Error::TryPush)
        }
    }

    pub fn try_push(&mut self, event: impl Event) -> Result<(), Error> {
        if event.header().time() >= self.last_time {
            unsafe { self.try_push_unchecked(event) }
        } else {
            Err(Error::OutOfOrder {
                last_time: self.last_time,
            })
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Error {
    OtherType(u16),
    PayloadSize(u32),
    TryPush,
    OutOfOrder { last_time: u32 },
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::PayloadSize(size) => {
                write!(f, "payload size for the defined event type: {size}")
            }
            Error::OtherType(id) => write!(f, "other type, id: {id}"),

            Error::TryPush => write!(f, "pushing event failed"),

            Error::OutOfOrder { last_time } => {
                write!(f, "event out of order, last event's time: {last_time}")
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
