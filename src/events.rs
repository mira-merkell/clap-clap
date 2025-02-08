//! CLAP events and event lists.

use core::fmt::{Display, Formatter};
use std::ffi::c_uint;

use crate::{
    ffi::{
        self, CLAP_EVENT_DONT_RECORD, CLAP_EVENT_IS_LIVE, CLAP_TRANSPORT_HAS_BEATS_TIMELINE,
        CLAP_TRANSPORT_HAS_SECONDS_TIMELINE, CLAP_TRANSPORT_HAS_TEMPO,
        CLAP_TRANSPORT_HAS_TIME_SIGNATURE, CLAP_TRANSPORT_IS_LOOP_ACTIVE,
        CLAP_TRANSPORT_IS_PLAYING, CLAP_TRANSPORT_IS_RECORDING, CLAP_TRANSPORT_IS_WITHIN_PRE_ROLL,
        clap_event_header, clap_event_midi, clap_event_midi_sysex, clap_event_midi2,
        clap_event_note, clap_event_note_expression, clap_event_param_gesture,
        clap_event_param_mod, clap_event_param_value, clap_event_transport, clap_input_events,
        clap_output_events,
    },
    fixedpoint::{BeatTime, SecTime},
    id::ClapId,
};

macro_rules! impl_flags_set_clear {
    ($Typ:ty, $flag_repr:ty) => {
        impl $Typ {
            pub const fn is_set(&self, flags: $flag_repr) -> bool {
                *self as $flag_repr & flags != 0
            }

            pub const fn set(&self, flags: $flag_repr) -> $flag_repr {
                *self as $flag_repr | flags
            }

            pub const fn clear(&self, flags: $flag_repr) -> $flag_repr {
                !(*self as $flag_repr) & flags
            }
        }
    };
}

#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(u32)]
pub enum EventFlags {
    IsLive = CLAP_EVENT_IS_LIVE,
    DontRecord = CLAP_EVENT_DONT_RECORD,
}

impl_flags_set_clear!(EventFlags, u32);

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Event {
    NoteOn(Note),
    NoteOff(Note),
    NoteChoke(Note),
    NoteEnd(Note),
    NoteExpression(NoteExpression),
    ParamValue(ParamValue),
    ParamMod(ParamMod),
    ParamGestureBegin(ParamGesture),
    ParamGestureEnd(ParamGesture),
    Transport(Transport),
    Midi(Midi),
    Midi2(Midi2),
    MidiSysex(MidiSysex),
}

impl Event {
    /// # Safety
    ///
    /// The pointer `header` must point to a full CLAP event of some type, i.e.
    /// it must be followed be the event 'body', so that the cast is safe.
    /// In particular, header must be non-null.
    const unsafe fn cast_and_copy_clap_event(
        header: *const clap_event_header,
    ) -> Result<Self, Error> {
        let ev_type = unsafe { *header }.r#type;

        match ev_type as c_uint {
            ffi::CLAP_EVENT_NOTE_ON => Ok(Self::NoteOn(Note(unsafe { *header.cast() }))),
            ffi::CLAP_EVENT_NOTE_OFF => Ok(Self::NoteOff(Note(unsafe { *header.cast() }))),
            ffi::CLAP_EVENT_NOTE_CHOKE => Ok(Self::NoteChoke(Note(unsafe { *header.cast() }))),
            ffi::CLAP_EVENT_NOTE_END => Ok(Self::NoteEnd(Note(unsafe { *header.cast() }))),

            ffi::CLAP_EVENT_NOTE_EXPRESSION => Ok(Event::NoteExpression(
                // SAFETY: the header is valid, and we just checked if the event type is correct.
                match unsafe { NoteExpression::cast_and_copy_clap_event(header) } {
                    // We don't call `unwrap()` here because the function is `const`.
                    Ok(note_expr) => note_expr,
                    Err(e) => {
                        return Err(e);
                    }
                },
            )),

            ffi::CLAP_EVENT_PARAM_VALUE => {
                Ok(Self::ParamValue(ParamValue(unsafe { *header.cast() })))
            }

            ffi::CLAP_EVENT_PARAM_MOD => Ok(Self::ParamMod(ParamMod(unsafe { *header.cast() }))),

            ffi::CLAP_EVENT_PARAM_GESTURE_BEGIN => {
                Ok(Self::ParamGestureBegin(ParamGesture(unsafe {
                    *header.cast()
                })))
            }
            ffi::CLAP_EVENT_PARAM_GESTURE_END => Ok(Self::ParamGestureEnd(ParamGesture(unsafe {
                *header.cast()
            }))),

            ffi::CLAP_EVENT_TRANSPORT => Ok(Self::Transport(Transport(unsafe { *header.cast() }))),

            ffi::CLAP_EVENT_MIDI => Ok(Self::Midi(Midi(unsafe { *header.cast() }))),
            ffi::CLAP_EVENT_MIDI2 => Ok(Self::Midi2(Midi2(unsafe { *header.cast() }))),
            ffi::CLAP_EVENT_MIDI_SYSEX => Ok(Self::MidiSysex(MidiSysex(unsafe { *header.cast() }))),

            _ => Err(Error::UnknownEvent(ev_type)),
        }
    }

    const fn clap_event_header(&self) -> &clap_event_header {
        match self {
            Event::NoteOn(ev) => &ev.0.header,
            Event::NoteOff(ev) => &ev.0.header,
            Event::NoteChoke(ev) => &ev.0.header,
            Event::NoteEnd(ev) => &ev.0.header,
            Event::NoteExpression(ev) => ev.clap_event_header(),
            Event::ParamValue(ev) => &ev.0.header,
            Event::ParamMod(ev) => &ev.0.header,
            Event::ParamGestureBegin(ev) => &ev.0.header,
            Event::ParamGestureEnd(ev) => &ev.0.header,
            Event::Transport(ev) => &ev.0.header,
            Event::Midi(ev) => &ev.0.header,
            Event::Midi2(ev) => &ev.0.header,
            Event::MidiSysex(ev) => &ev.0.header,
        }
    }
}

macro_rules! impl_event_get_attr {
    ($Typ:ty, $($attr:tt:$attr_typ:ty),*) => {
        impl $Typ {
            $(
                pub const fn $attr(&self) -> $attr_typ {
                    self.0.$attr
                }
            )*

            pub const fn time(&self) -> u32 {
                self.0.header.time
            }

            pub const fn space_id(&self) -> u16 {
                self.0.header.space_id
            }

            pub const fn flags(&self) -> u32 {
                self.0.header.flags
            }
        }
    };
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Note(clap_event_note);

impl_event_get_attr!(Note,
    note_id:i32, port_index:i16, channel:i16, key:i16, velocity:f64
);

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum NoteExpression {
    Volume(Expression),
    Pan(Expression),
    Tuning(Expression),
    Vibrato(Expression),
    Expression(Expression),
    Brightness(Expression),
    Pressure(Expression),
}

impl NoteExpression {
    /// # Safety
    ///
    /// 1. The pointer `header` must point to a full CLAP event of some type,
    ///    i.e. it must be followed be the event 'body', so that the cast is
    ///    safe. In particular, header must be non-null.
    /// 2. The event type must be CLAP_EVENT_NOTE_EXPRESSION.
    const unsafe fn cast_and_copy_clap_event(
        header: *const clap_event_header,
    ) -> Result<Self, Error> {
        let ev = unsafe { *header.cast::<clap_event_note_expression>() };
        match ev.expression_id {
            ffi::CLAP_NOTE_EXPRESSION_VOLUME => Ok(Self::Volume(Expression(ev))),
            ffi::CLAP_NOTE_EXPRESSION_PAN => Ok(Self::Pan(Expression(ev))),
            ffi::CLAP_NOTE_EXPRESSION_TUNING => Ok(Self::Tuning(Expression(ev))),
            ffi::CLAP_NOTE_EXPRESSION_VIBRATO => Ok(Self::Vibrato(Expression(ev))),
            ffi::CLAP_NOTE_EXPRESSION_EXPRESSION => Ok(Self::Expression(Expression(ev))),
            ffi::CLAP_NOTE_EXPRESSION_BRIGHTNESS => Ok(Self::Brightness(Expression(ev))),
            ffi::CLAP_NOTE_EXPRESSION_PRESSURE => Ok(Self::Pressure(Expression(ev))),
            _ => Err(Error::UnknownExpression(ev.expression_id)),
        }
    }

    const fn expression(&self) -> &Expression {
        match self {
            NoteExpression::Volume(e) => e,
            NoteExpression::Pan(e) => e,
            NoteExpression::Tuning(e) => e,
            NoteExpression::Vibrato(e) => e,
            NoteExpression::Expression(e) => e,
            NoteExpression::Brightness(e) => e,
            NoteExpression::Pressure(e) => e,
        }
    }

    const fn clap_event_header(&self) -> &clap_event_header {
        &self.expression().0.header
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Expression(clap_event_note_expression);

impl_event_get_attr!(Expression,
    note_id:i32, port_index:i16, channel:i16, key:i16, value:f64
);

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ParamValue(clap_event_param_value);

impl_event_get_attr!(ParamValue,
    note_id:i32, port_index:i16, channel:i16, key:i16, value:f64
);

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ParamMod(clap_event_param_mod);

impl_event_get_attr!(ParamMod,
    note_id:i32, port_index:i16, channel:i16, key:i16, amount:f64
);

impl ParamMod {
    pub fn param_id(&self) -> ClapId {
        self.0.param_id.try_into().unwrap_or(ClapId::invalid_id())
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ParamGesture(clap_event_param_gesture);

impl_event_get_attr!(ParamGesture,);

impl ParamGesture {
    pub fn param_id(&self) -> ClapId {
        self.0.param_id.try_into().unwrap_or(ClapId::invalid_id())
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
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

impl_flags_set_clear!(TransportFlags, u32);

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Transport(clap_event_transport);

impl_event_get_attr!(Transport,
    tempo:f64, tempo_inc:f64,
     bar_number:i32,
    tsig_num:u16, tsig_denom:u16
);

macro_rules! impl_transport_get_attr {
    ($Typ:ty, $($attr:tt:$attr_typ:ty),*) => {
        impl $Typ {
            $(
                pub const fn $attr(&self) -> $attr_typ {
                    <$attr_typ>::new(self.0.$attr)
                }
            )*
        }
    };
}

impl_transport_get_attr!(Transport,
    song_pos_beats:BeatTime, song_pos_seconds:SecTime,
    loop_start_beats:BeatTime, loop_end_beats:BeatTime,
    loop_start_seconds:SecTime, loop_end_seconds:SecTime,
    bar_start:BeatTime
);

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Midi(clap_event_midi);

impl_event_get_attr!(Midi, port_index:u16);

impl Midi {
    pub const fn data(&self) -> &[u8; 3] {
        &self.0.data
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Midi2(clap_event_midi2);

impl_event_get_attr!(Midi2, port_index:u16);

impl Midi2 {
    pub const fn data(&self) -> &[u32; 4] {
        &self.0.data
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct MidiSysex(clap_event_midi_sysex);

#[derive(Debug)]
pub struct InputEvents(*const clap_input_events);

impl InputEvents {
    /// # Safety:
    ///
    /// 1. `clap_input_events` must be non-null
    /// 2. `clap_input_events.size` must be non-null
    /// 3. `clap_input_events.get` must be non-null
    pub(crate) const unsafe fn new(clap_input_events: *const clap_input_events) -> Self {
        Self(clap_input_events)
    }

    pub fn size(&self) -> u32 {
        // SAFETY: By construction, the call is safe.
        unsafe { (*self.0).size.unwrap()(self.0) }
    }

    /// # Safety
    ///
    /// The value of `index` must be less than `self.size()`.
    pub unsafe fn get_unchecked(&self, index: u32) -> Result<Event, Error> {
        // SAFETY: By construction, and by the fact that `index < self.size()`,
        // the call to `get()` is safe.
        let header = unsafe { (*self.0).get.unwrap()(self.0, index) };
        if header.is_null() {
            return Err(Error::Null);
        }
        // SAFETY: the header ev is followed by the 'body' of the event, because
        // the header points to a genuine event obtained from the host.
        unsafe { Event::cast_and_copy_clap_event(header) }
    }

    /// # Panic
    ///
    /// This function will panic if `index >= self.size()`.
    pub fn get(&self, index: u32) -> Result<Event, Error> {
        (index < self.size())
            .then_some(unsafe { self.get_unchecked(index) })
            .expect("index out of bound")
    }
}

pub struct OutputEvents {
    clap_output_events: *const clap_output_events,
    last_time: u32,
}

impl OutputEvents {
    /// # Safety:
    ///
    /// 1. `clap_output_events` must be non-null
    /// 2. `clap_output_events.try_push` must be non-null
    pub(crate) const unsafe fn new(clap_output_events: *const clap_output_events) -> Self {
        Self {
            clap_output_events,
            last_time: 0,
        }
    }

    /// # Safety
    ///
    /// Events must be pushed into the list in the sample order.
    pub unsafe fn try_push_unchecked(&mut self, event: Event) -> Result<(), Error> {
        let header = event.clap_event_header();
        // SAFETY: By construction, `try_push()` is non-null.
        unsafe { (*self.clap_output_events).try_push.unwrap()(self.clap_output_events, header) }
            .then_some(())
            .ok_or(Error::TryPush(event))
    }

    pub fn try_push(&mut self, event: Event) -> Result<(), Error> {
        let header = event.clap_event_header();
        let time = header.time;
        if self.last_time <= time {
            let res = unsafe { self.try_push_unchecked(event) };
            self.last_time = time;
            res
        } else {
            Err(Error::OutOfOrder(event))
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Error {
    Null,
    UnknownEvent(u16),
    UnknownExpression(i32),
    OutOfOrder(Event),
    TryPush(Event),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::Null => write!(f, "null pointer"),
            Error::UnknownEvent(id) => write!(f, "unknown event type: {id}"),
            Error::UnknownExpression(id) => write!(f, "unknown note expression: {id}"),
            Error::OutOfOrder(ev) => {
                write!(f, "events must be inserted in the sample order: {ev:?}")
            }
            Error::TryPush(ev) => {
                write!(
                    f,
                    "event could not be pushed to the queue (out of memory?): {ev:?}"
                )
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
