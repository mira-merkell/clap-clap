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

mod param_value {
    use std::ptr::null_mut;

    use clap_clap::{
        events,
        events::{EventBuilder, Header, ParamValue},
        ffi::{
            CLAP_EVENT_NOTE_CHOKE, CLAP_EVENT_PARAM_VALUE, clap_event_header,
            clap_event_param_value,
        },
    };

    #[test]
    fn try_01() {
        let event = clap_event_param_value {
            header: clap_event_header {
                size: 33,
                time: 0,
                space_id: 0,
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
        };

        let header = unsafe { Header::new(&event.header) };
        assert_eq!(Err(events::Error::PayloadSize(33)), header.param_value())
    }

    #[test]
    fn try_02() {
        let event = clap_event_param_value {
            header: clap_event_header {
                size: size_of::<clap_event_param_value>() as u32,
                time: 0,
                space_id: 0,
                r#type: CLAP_EVENT_NOTE_CHOKE as u16,
                flags: 0,
            },
            param_id: 0,
            cookie: null_mut(),
            note_id: 0,
            port_index: 0,
            channel: 0,
            key: 0,
            value: 0.0,
        };

        let header = unsafe { Header::new(&event.header) };
        assert_eq!(
            Err(events::Error::OtherType(CLAP_EVENT_NOTE_CHOKE as u16)),
            header.midi()
        )
    }

    #[test]
    fn try_03() {
        let event = clap_event_param_value {
            header: clap_event_header {
                size: size_of::<clap_event_param_value>() as u32,
                time: 0,
                space_id: 0,
                r#type: CLAP_EVENT_PARAM_VALUE as u16,
                flags: 0,
            },
            param_id: 0,
            cookie: null_mut(),
            note_id: 0,
            port_index: 87,
            channel: 1,
            key: 0,
            value: 123.456,
        };

        let header = unsafe { Header::new(&event.header) };
        let _ = header.midi().unwrap_err();
        let event = header.param_value().unwrap();
        assert_eq!(event.port_index(), 87);
        assert_eq!(event.value(), 123.456);
    }

    #[test]
    fn build() {
        let param_value1 = ParamValue::build().port_index(1).value(123.456);
        let param_value2 = param_value1.port_index(3);

        let event1 = param_value1.event();
        let event2 = param_value2.event();

        assert_eq!(event1.value(), 123.456);
        assert_eq!(event2.value(), 123.456);

        assert_eq!(event1.port_index(), 1);
        assert_eq!(event2.port_index(), 3);
    }
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

mod midi2 {
    use clap_clap::{
        events,
        events::{EventBuilder, Header, Midi2},
        ffi::{CLAP_EVENT_MIDI2, CLAP_EVENT_NOTE_CHOKE, clap_event_header, clap_event_midi2},
    };

    #[test]
    fn try_midi2_01() {
        let event = clap_event_midi2 {
            header: clap_event_header {
                size: 3,
                time: 0,
                space_id: 0,
                r#type: CLAP_EVENT_MIDI2 as u16,
                flags: 0,
            },
            port_index: 0,
            data: [0, 0, 0, 0],
        };

        let header = unsafe { Header::new(&event.header) };
        assert_eq!(Err(events::Error::PayloadSize(3)), header.midi2())
    }

    #[test]
    fn try_midi2_02() {
        let event = clap_event_midi2 {
            header: clap_event_header {
                size: size_of::<clap_event_midi2>() as u32,
                time: 0,
                space_id: 0,
                r#type: CLAP_EVENT_NOTE_CHOKE as u16,
                flags: 0,
            },
            port_index: 0,
            data: [0, 0, 0, 0],
        };

        let header = unsafe { Header::new(&event.header) };
        assert_eq!(
            Err(events::Error::OtherType(CLAP_EVENT_NOTE_CHOKE as u16)),
            header.midi2()
        )
    }

    #[test]
    fn try_midi_03() {
        let event = clap_event_midi2 {
            header: clap_event_header {
                size: size_of::<clap_event_midi2>() as u32,
                time: 0,
                space_id: 0,
                r#type: CLAP_EVENT_MIDI2 as u16,
                flags: 0,
            },
            port_index: 87,
            data: [1, 2, 3, 4],
        };

        let header = unsafe { Header::new(&event.header) };
        let midi = header.midi2().unwrap();
        assert_eq!(midi.data(), &[1, 2, 3, 4]);
        assert_eq!(midi.port_index(), 87);
    }

    #[test]
    fn build() {
        let midi1 = Midi2::build().port_index(1).data([5, 6, 7, 8]);
        let midi2 = midi1.port_index(3);

        let event1 = midi1.event();
        let event2 = midi2.event();

        assert_eq!(event1.data(), &[5, 6, 7, 8]);
        assert_eq!(event2.data(), &[5, 6, 7, 8]);

        assert_eq!(event1.port_index(), 1);
        assert_eq!(event2.port_index(), 3);
    }
}

mod input_events {
    use std::ptr::null_mut;

    use clap_clap::{
        events::InputEvents,
        ffi::{
            CLAP_CORE_EVENT_SPACE_ID, CLAP_EVENT_MIDI, CLAP_EVENT_PARAM_VALUE, clap_event_header,
            clap_event_midi, clap_event_param_value, clap_input_events,
        },
    };

    static EVENT1: clap_event_midi = clap_event_midi {
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

    const EVENT2: clap_event_param_value = clap_event_param_value {
        header: clap_event_header {
            size: size_of::<clap_event_param_value>() as u32,
            time: 2,
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
        value: 123.456,
    };

    extern "C-unwind" fn test_size(_: *const clap_input_events) -> u32 {
        2
    }

    extern "C-unwind" fn test_get(
        _: *const clap_input_events,
        index: u32,
    ) -> *const clap_event_header {
        if index == 0 {
            &EVENT1.header
        } else {
            &EVENT2.header
        }
    }

    const INPUT_EVENTS: clap_input_events = clap_input_events {
        ctx: null_mut(),
        size: Some(test_size),
        get: Some(test_get),
    };

    #[test]
    fn input_events() {
        let input_events = unsafe { InputEvents::new_unchecked(&INPUT_EVENTS) };
        assert_eq!(input_events.size(), 2);

        let header = input_events.get(0);
        let _ = header.midi2().unwrap_err();
        let event = header.midi().unwrap();
        assert_eq!(event.data(), &[1, 2, 3]);

        let header = input_events.get(1);
        let _ = header.midi().unwrap_err();
        let event = header.param_value().unwrap();
        assert_eq!(event.value(), 123.456);
    }

    #[should_panic(expected = "index out of bounds")]
    #[test]
    fn input_events_index_oob() {
        let input_events = unsafe { InputEvents::new_unchecked(&INPUT_EVENTS) };
        assert_eq!(input_events.size(), 2);

        let _ = input_events.get(2);
    }
}
