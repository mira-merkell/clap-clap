use std::ptr::null_mut;

use clap_clap::{
    events::{Event, InputEvents, Midi, Note},
    ffi::{
        CLAP_EVENT_MIDI, CLAP_EVENT_NOTE_OFF, CLAP_EVENT_NOTE_ON, clap_event_header,
        clap_event_midi, clap_event_note, clap_input_events,
    },
};

const EVENT_NOTE: [clap_event_note; 2] = [
    clap_event_note {
        header: clap_event_header {
            size: size_of::<clap_event_note>() as u32,
            time: 9,
            space_id: 0,
            r#type: CLAP_EVENT_NOTE_ON as u16,
            flags: 0,
        },
        note_id: -1,
        port_index: 0,
        channel: 0,
        key: 50,
        velocity: 12.0,
    },
    clap_event_note {
        header: clap_event_header {
            size: size_of::<clap_event_note>() as u32,
            time: 99,
            space_id: 0,
            r#type: CLAP_EVENT_NOTE_OFF as u16,
            flags: 1,
        },
        note_id: 0,
        port_index: 0,
        channel: 7,
        key: 60,
        velocity: 99.0,
    },
];

const EVENT_MIDI: [clap_event_midi; 1] = [clap_event_midi {
    header: clap_event_header {
        size: size_of::<clap_event_midi>() as u32,
        time: 23,
        space_id: 0,
        r#type: CLAP_EVENT_MIDI as u16,
        flags: 3,
    },
    port_index: 0,
    data: [1, 2, 3],
}];

static EVENT_HEADERS: [&clap_event_header; 3] = [
    &EVENT_NOTE[0].header,
    &EVENT_MIDI[0].header,
    &EVENT_NOTE[1].header,
];

extern "C-unwind" fn input_events_size(_: *const clap_input_events) -> u32 {
    EVENT_HEADERS.len() as u32
}

extern "C-unwind" fn input_events_get(
    _: *const clap_input_events,
    index: u32,
) -> *const clap_event_header {
    EVENT_HEADERS[index as usize]
}

const INPUT_EVENTS: clap_input_events = clap_input_events {
    ctx: null_mut(),
    size: Some(input_events_size),
    get: Some(input_events_get),
};

#[test]
fn input_events() {
    let input_events = unsafe { InputEvents::new(&INPUT_EVENTS) };
    assert_eq!(input_events.size(), 3);

    let ev = input_events.get(0).unwrap();
    let Event::NoteOn(note) = ev else {
        panic!("wrong event type")
    };
    assert_eq!(note, Note::from(EVENT_NOTE[0]));

    let ev = input_events.get(1).unwrap();
    let Event::Midi(midi) = ev else {
        panic!("wrong event type")
    };
    assert_eq!(midi, Midi::from(EVENT_MIDI[0]));

    let ev = input_events.get(2).unwrap();
    let Event::NoteOff(note) = ev else {
        panic!("wrong event type")
    };
    assert_eq!(note, Note::from(EVENT_NOTE[1]));
}

#[should_panic]
#[test]
fn input_events_wrong_index() {
    let input_events = unsafe { InputEvents::new(&INPUT_EVENTS) };
    assert_eq!(input_events.size(), 3);

    let _ = input_events.get(3);
}
