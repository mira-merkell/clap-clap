use clap_clap::{
    events::Header,
    ffi::{CLAP_EVENT_MIDI, clap_event_header, clap_event_midi},
};

mod cast_event_ids {
    macro_rules! check_cast_event_ids_u16 {
        ($($name:ident),* $(,)?) => { $(
            #[allow(non_snake_case)]
            #[test]
            fn $name() {
                assert!((clap_clap::ffi::$name as u64) < u16::MAX as u64);
                let _ = u16::try_from( clap_clap::ffi::$name ).unwrap();
            }
        )*};
    }
    check_cast_event_ids_u16!(
        CLAP_EVENT_NOTE_ON,
        CLAP_EVENT_NOTE_OFF,
        CLAP_EVENT_NOTE_CHOKE,
        CLAP_EVENT_NOTE_END,
        CLAP_EVENT_NOTE_EXPRESSION,
        CLAP_EVENT_PARAM_VALUE,
        CLAP_EVENT_PARAM_MOD,
        CLAP_EVENT_PARAM_GESTURE_BEGIN,
        CLAP_EVENT_PARAM_GESTURE_END,
        CLAP_EVENT_TRANSPORT,
        CLAP_EVENT_MIDI,
        CLAP_EVENT_MIDI_SYSEX,
        CLAP_EVENT_MIDI2,
    );
}

#[test]
fn cast_header() {
    let event = clap_event_midi {
        header: clap_event_header {
            size: size_of::<clap_event_midi>() as u32,
            time: 1,
            space_id: 2,
            r#type: CLAP_EVENT_MIDI as u16,
            flags: 3,
        },
        port_index: 0,
        data: [0, 0, 0],
    };

    let header = unsafe { Header::new(&event.header) };
    assert_eq!(header.time(), 1);
    assert_eq!(header.space_id(), 2);
    assert_eq!(header.flags(), 3);

    assert_eq!(header.r#type(), CLAP_EVENT_MIDI as u16);
}

mod midi {
    use clap_clap::{
        events,
        events::{EventBuilder, Header, Midi},
        ffi::{CLAP_EVENT_MIDI, CLAP_EVENT_NOTE_CHOKE, clap_event_header, clap_event_midi},
    };

    #[test]
    fn try_midi_01() {
        let event = clap_event_midi {
            header: clap_event_header {
                size: 3,
                time: 0,
                space_id: 0,
                r#type: CLAP_EVENT_MIDI as u16,
                flags: 0,
            },
            port_index: 0,
            data: [0, 0, 0],
        };

        let header = unsafe { Header::new(&event.header) };
        assert_eq!(Err(events::Error::PayloadSize(3)), header.midi())
    }

    #[test]
    fn try_midi_02() {
        let event = clap_event_midi {
            header: clap_event_header {
                size: size_of::<clap_event_midi>() as u32,
                time: 0,
                space_id: 0,
                r#type: CLAP_EVENT_NOTE_CHOKE as u16,
                flags: 0,
            },
            port_index: 0,
            data: [0, 0, 0],
        };

        let header = unsafe { Header::new(&event.header) };
        assert_eq!(
            Err(events::Error::OtherType(CLAP_EVENT_NOTE_CHOKE as u16)),
            header.midi()
        )
    }

    #[test]
    fn try_midi_03() {
        let event = clap_event_midi {
            header: clap_event_header {
                size: size_of::<clap_event_midi>() as u32,
                time: 0,
                space_id: 0,
                r#type: CLAP_EVENT_MIDI as u16,
                flags: 0,
            },
            port_index: 87,
            data: [1, 2, 3],
        };

        let header = unsafe { Header::new(&event.header) };
        let midi = header.midi().unwrap();
        assert_eq!(midi.data(), &[1, 2, 3]);
        assert_eq!(midi.port_index(), 87);
    }

    #[test]
    fn build() {
        let midi1 = Midi::build().port_index(1).data([5, 6, 7]);
        let midi2 = midi1.port_index(3);

        let event1 = midi1.event();
        let event2 = midi2.event();

        assert_eq!(event1.data(), &[5, 6, 7]);
        assert_eq!(event2.data(), &[5, 6, 7]);

        assert_eq!(event1.port_index(), 1);
        assert_eq!(event2.port_index(), 3);
    }
}

mod input_events {
    use std::ptr::null_mut;

    use clap_clap::{
        events::InputEvents,
        ffi::{
            CLAP_CORE_EVENT_SPACE_ID, CLAP_EVENT_MIDI, clap_event_header, clap_event_midi,
            clap_input_events,
        },
    };

    static MIDI_EVENT: clap_event_midi = clap_event_midi {
        header: clap_event_header {
            size: size_of::<clap_event_midi>() as u32,
            time: 1,
            space_id: CLAP_CORE_EVENT_SPACE_ID,
            r#type: CLAP_EVENT_MIDI as u16,
            flags: 0,
        },
        port_index: 0,
        data: [1, 2, 3],
    };

    // TODO: once other event types are implemented, change this to some other type.
    static MIDI_EVENT2: clap_event_midi = clap_event_midi {
        header: clap_event_header {
            size: size_of::<clap_event_midi>() as u32,
            time: 2,
            space_id: CLAP_CORE_EVENT_SPACE_ID,
            r#type: CLAP_EVENT_MIDI as u16,
            flags: 0,
        },
        port_index: 0,
        data: [5, 6, 7],
    };

    extern "C-unwind" fn test_size(_: *const clap_input_events) -> u32 {
        2
    }

    extern "C-unwind" fn test_get(
        _: *const clap_input_events,
        index: u32,
    ) -> *const clap_event_header {
        if index == 0 {
            &MIDI_EVENT.header
        } else {
            &MIDI_EVENT2.header
        }
    }

    const INPUT_EVENTS: clap_input_events = clap_input_events {
        ctx: null_mut(),
        size: Some(test_size),
        get: Some(test_get),
    };

    #[test]
    fn input_events() {
        let input_events = unsafe { InputEvents::new(&INPUT_EVENTS) };
        assert_eq!(input_events.size(), 2);

        let header = input_events.get(0);
        let event = header.midi().unwrap();
        assert_eq!(event.data(), &[1, 2, 3]);

        let header = input_events.get(1);
        let event = header.midi().unwrap();
        assert_eq!(event.data(), &[5, 6, 7]);
    }

    #[should_panic(expected = "index out of bounds")]
    #[test]
    fn input_events_index_oob() {
        let input_events = unsafe { InputEvents::new(&INPUT_EVENTS) };
        assert_eq!(input_events.size(), 2);

        let _ = input_events.get(2);
    }
}
