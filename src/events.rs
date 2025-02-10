//! Events and event lists.

use std::{
    fmt::{Display, Formatter},
    ptr::slice_from_raw_parts,
};

use crate::ffi::{CLAP_EVENT_MIDI, clap_event_header, clap_event_midi, clap_input_events};

#[repr(transparent)]
pub struct Event([u8]);

impl Event {
    const unsafe fn new(header: &clap_event_header) -> &Self {
        let len = header.size as usize;
        let data = &raw const *header as *const _;
        unsafe { &*(slice_from_raw_parts::<u8>(data, len) as *const _) }
    }

    const fn as_clap_event_header(&self) -> &clap_event_header {
        unsafe { &*(self.0.as_ptr() as *const _) }
    }

    pub const fn midi(&self) -> Result<Midi, Error> {
        let ev_type = self.as_clap_event_header().r#type;

        if ev_type == CLAP_EVENT_MIDI as u16 {
            let midi: &clap_event_midi = unsafe { &*(self.0.as_ptr() as *const _) };
            Ok(Midi::from_clap_event(midi))
        } else {
            Err(Error::OtherType(ev_type))
        }
    }
}

pub struct Midi {
    pub port_index: u16,
    pub data: [u8; 3],
}

impl Midi {
    const fn from_clap_event(event: &clap_event_midi) -> Self {
        Self {
            port_index: event.port_index,
            data: event.data,
        }
    }
}

pub struct InputEvents(*const clap_input_events);

impl InputEvents {}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Error {
    UnknownEvent(u16),
    UnknownExpression(i32),
    OutOfOrder,
    TryPush,
    OtherType(u16),
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

#[test]
fn cast_midi_event() {
    let ev = clap_event_midi {
        header: clap_event_header {
            size: size_of::<clap_event_midi>() as u32,
            time: 0,
            space_id: 0,
            r#type: CLAP_EVENT_MIDI as u16,
            flags: 0,
        },
        port_index: 7,
        data: [1, 2, 3],
    };

    let event = unsafe { Event::new(&ev.header) };

    let midi = event.midi().unwrap();

    assert_eq!(midi.port_index, ev.port_index);
    assert_eq!(midi.data, ev.data);
}
