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
            header.param_value()
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
        let value1 = ParamValue::build().port_index(1).value(123.456);
        let value2 = value1.port_index(3);

        let event1 = value1.event();
        let event2 = value2.event();

        assert_eq!(event1.value(), 123.456);
        assert_eq!(event2.value(), 123.456);

        assert_eq!(event1.port_index(), 1);
        assert_eq!(event2.port_index(), 3);
    }

    #[test]
    fn update() {
        let value1 = ParamValue::build().port_index(1).value(123.456);
        let event1 = value1.event();

        let value2 = ParamValue::update(&event1).port_index(3);
        let event2 = value2.event();

        assert_eq!(event1.value(), 123.456);
        assert_eq!(event2.value(), 123.456);

        assert_eq!(event1.port_index(), 1);
        assert_eq!(event2.port_index(), 3);
    }
}

mod param_mod {
    use std::ptr::null_mut;

    use clap_clap::{
        events,
        events::{EventBuilder, Header, ParamMod},
        ffi::{
            CLAP_EVENT_NOTE_CHOKE, CLAP_EVENT_PARAM_MOD, clap_event_header, clap_event_param_mod,
        },
    };

    #[test]
    fn try_01() {
        let event = clap_event_param_mod {
            header: clap_event_header {
                size: 33,
                time: 0,
                space_id: 0,
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
        };

        let header = unsafe { Header::new(&event.header) };
        assert_eq!(Err(events::Error::PayloadSize(33)), header.param_mod())
    }

    #[test]
    fn try_02() {
        let event = clap_event_param_mod {
            header: clap_event_header {
                size: size_of::<clap_event_param_mod>() as u32,
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
            amount: 0.0,
        };

        let header = unsafe { Header::new(&event.header) };
        assert_eq!(
            Err(events::Error::OtherType(CLAP_EVENT_NOTE_CHOKE as u16)),
            header.param_mod()
        )
    }

    #[test]
    fn try_03() {
        let event = clap_event_param_mod {
            header: clap_event_header {
                size: size_of::<clap_event_param_mod>() as u32,
                time: 0,
                space_id: 0,
                r#type: CLAP_EVENT_PARAM_MOD as u16,
                flags: 0,
            },
            param_id: 0,
            cookie: null_mut(),
            note_id: 0,
            port_index: 87,
            channel: 1,
            key: 0,
            amount: 123.456,
        };

        let header = unsafe { Header::new(&event.header) };
        let _ = header.midi().unwrap_err();
        let event = header.param_mod().unwrap();
        assert_eq!(event.port_index(), 87);
        assert_eq!(event.amount(), 123.456);
    }

    #[test]
    fn build() {
        let value1 = ParamMod::build().port_index(1).amount(123.456);
        let value2 = value1.port_index(3);

        let event1 = value1.event();
        let event2 = value2.event();

        assert_eq!(event1.amount(), 123.456);
        assert_eq!(event2.amount(), 123.456);

        assert_eq!(event1.port_index(), 1);
        assert_eq!(event2.port_index(), 3);
    }

    #[test]
    fn update() {
        let value1 = ParamMod::build().port_index(1).amount(123.456);
        let event1 = value1.event();

        let value2 = ParamMod::update(&event1).port_index(3);
        let event2 = value2.event();

        assert_eq!(event1.amount(), 123.456);
        assert_eq!(event2.amount(), 123.456);

        assert_eq!(event1.port_index(), 1);
        assert_eq!(event2.port_index(), 3);
    }
}

mod transport {
    use clap_clap::{
        events,
        events::{EventBuilder, Header, Transport},
        ffi::{
            CLAP_EVENT_NOTE_CHOKE, CLAP_EVENT_TRANSPORT, clap_event_header, clap_event_transport,
        },
        fixedpoint::{BeatTime, SecTime},
    };

    #[test]
    fn try_01() {
        let event = clap_event_transport {
            header: clap_event_header {
                size: 33,
                time: 0,
                space_id: 0,
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
        };

        let header = unsafe { Header::new(&event.header) };
        assert_eq!(Err(events::Error::PayloadSize(33)), header.transport())
    }

    #[test]
    fn try_02() {
        let event = clap_event_transport {
            header: clap_event_header {
                size: size_of::<clap_event_transport>() as u32,
                time: 0,
                space_id: 0,
                r#type: CLAP_EVENT_NOTE_CHOKE as u16,
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
        };

        let header = unsafe { Header::new(&event.header) };
        assert_eq!(
            Err(events::Error::OtherType(CLAP_EVENT_NOTE_CHOKE as u16)),
            header.transport()
        )
    }

    #[test]
    fn try_03() {
        let event = clap_event_transport {
            header: clap_event_header {
                size: size_of::<clap_event_transport>() as u32,
                time: 0,
                space_id: 0,
                r#type: CLAP_EVENT_TRANSPORT as u16,
                flags: 0,
            },
            flags: 0,
            song_pos_beats: 0,
            song_pos_seconds: 0,
            tempo: 0.0,
            tempo_inc: 0.0,
            loop_start_beats: 0,
            loop_end_beats: 110011,
            loop_start_seconds: 0,
            loop_end_seconds: 0,
            bar_start: 0,
            bar_number: 12345,
            tsig_num: 0,
            tsig_denom: 0,
        };

        let header = unsafe { Header::new(&event.header) };
        let _ = header.param_value().unwrap_err();
        let event = header.transport().unwrap();
        assert_eq!(event.loop_end_beats(), BeatTime(110011));
        assert_eq!(event.bar_number(), 12345);
    }

    #[test]
    fn build() {
        let value1 = Transport::build()
            .tempo(1.11)
            .song_pos_seconds(SecTime::from(12.34));
        let value2 = value1.tempo(3.3);

        let event1 = value1.event();
        let event2 = value2.event();

        assert_eq!(event1.song_pos_seconds(), SecTime::from(12.34));
        assert_eq!(event2.song_pos_seconds(), SecTime::from(12.34));

        assert_eq!(event1.tempo(), 1.11);
        assert_eq!(event2.tempo(), 3.3);
    }

    #[test]
    fn update() {
        let value1 = Transport::build()
            .tempo(1.11)
            .song_pos_seconds(SecTime::from(12.34));
        let event1 = value1.event();

        let value2 = Transport::update(&event1).tempo(3.3);
        let event2 = value2.event();

        assert_eq!(event1.song_pos_seconds(), SecTime::from(12.34));
        assert_eq!(event2.song_pos_seconds(), SecTime::from(12.34));

        assert_eq!(event1.tempo(), 1.11);
        assert_eq!(event2.tempo(), 3.3);
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

    #[test]
    fn update() {
        let midi1 = Midi::build().port_index(1).data([5, 6, 7]);
        let event1 = midi1.event();

        let midi2 = Midi::update(&event1).port_index(3);
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

    #[test]
    fn update() {
        let midi1 = Midi2::build().port_index(1).data([5, 6, 7, 8]);
        let event1 = midi1.event();

        let midi2 = Midi2::update(&event1).port_index(3);
        let event2 = midi2.event();

        assert_eq!(event1.data(), &[5, 6, 7, 8]);
        assert_eq!(event2.data(), &[5, 6, 7, 8]);

        assert_eq!(event1.port_index(), 1);
        assert_eq!(event2.port_index(), 3);
    }
}

pub mod input_events {
    use std::{
        marker::PhantomPinned,
        pin::Pin,
        ptr::{NonNull, null, null_mut},
    };

    use clap_clap::{
        events::{
            Event, EventBuilder, InputEvents, Midi, Midi2, Midi2Builder, MidiBuilder, ParamMod,
            ParamModBuilder, ParamValue, ParamValueBuilder, Transport, TransportBuilder,
        },
        ffi::{
            CLAP_CORE_EVENT_SPACE_ID, CLAP_EVENT_MIDI, CLAP_EVENT_MIDI2, CLAP_EVENT_PARAM_VALUE,
            clap_event_header, clap_event_midi, clap_event_param_value, clap_input_events,
            clap_process,
        },
        prelude::Process,
    };

    use crate::shims::events::output_events::SHIM_CLAP_OUTPUT_EVENTS;

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

    struct TestBed<'a> {
        events: Vec<Box<dyn Event + 'a>>,
        clap_input_events: clap_input_events,
        // clap_input_events list will be passed the pointer to events,
        // which makes is a self-referential struct.  We need to make sure the
        // TestBed is not moved.
        _marker: PhantomPinned,
    }

    impl<'a> TestBed<'a> {
        fn new() -> Pin<Box<Self>> {
            extern "C-unwind" fn size(list: *const clap_input_events) -> u32 {
                let events: &Vec<Box<dyn Event>> = unsafe { &*((*list).ctx as *const _) };
                events.len() as u32
            }

            extern "C-unwind" fn get(
                list: *const clap_input_events,
                index: u32,
            ) -> *const clap_event_header {
                let events: &Vec<Box<dyn Event>> = unsafe { &*((*list).ctx as *const _) };
                let index = index as usize;
                if index < events.len() {
                    events[index].header().as_clap_event_header()
                } else {
                    null()
                }
            }
            let mut bed = Box::new(Self {
                events: Vec::new(),
                clap_input_events: clap_input_events {
                    ctx: null_mut(),
                    size: Some(size),
                    get: Some(get),
                },
                _marker: PhantomPinned,
            });
            bed.clap_input_events.ctx = &raw const bed.events as *mut _;
            Box::into_pin(bed)
        }

        fn push_event<E: Event + 'a>(self: Pin<&mut Self>, event: E) {
            unsafe { Pin::get_unchecked_mut(self) }
                .events
                .push(Box::new(event))
        }

        fn input_events(&'a self) -> InputEvents<'a> {
            unsafe { InputEvents::new_unchecked(&self.clap_input_events) }
        }

        fn retrieve_events(&self, input_events: InputEvents<'a>) {
            for (i, known) in self.events.iter().enumerate() {
                let ev = input_events.get(i as u32);
                let type_id = ev.r#type() as u32;

                if type_id == CLAP_EVENT_MIDI {
                    assert_eq!(known.header().midi().unwrap(), ev.midi().unwrap());
                }
                if type_id == CLAP_EVENT_MIDI2 {
                    assert_eq!(known.header().midi2().unwrap(), ev.midi2().unwrap());
                }
                if type_id == CLAP_EVENT_PARAM_VALUE {
                    assert_eq!(
                        known.header().param_value().unwrap(),
                        ev.param_value().unwrap()
                    );
                }
            }
        }
    }

    #[test]
    fn testbed_self_test_01() {
        let midi = Midi::build().port_index(1);

        let mut bed = TestBed::new();
        bed.as_mut().push_event(midi.event());
        let input_events = bed.input_events();

        assert_eq!(input_events.size(), 1);
    }

    #[test]
    fn testbed_self_test_02() {
        let midi = Midi::build().port_index(1);
        let midi2 = Midi2::build().port_index(3);
        let mut bed = TestBed::new();

        bed.as_mut().push_event(midi.event());
        bed.as_mut().push_event(midi2.event());
        let input_events = bed.input_events();
        assert_eq!(input_events.size(), 2);

        let ev = input_events.get(0);
        let retrieved = ev.midi().unwrap();
        assert_eq!(retrieved.port_index(), 1);

        let ev = input_events.get(1);
        let retrieved = ev.midi2().unwrap();
        assert_eq!(retrieved.port_index(), 3);
    }

    #[derive(Debug, Copy, Clone, PartialEq)]
    pub enum KnownEvent {
        Midi(MidiBuilder),
        Midi2(Midi2Builder),
        ParamMod(ParamModBuilder),
        ParamValue(ParamValueBuilder),
        Transport(TransportBuilder),
    }

    fn check_input_events(events: &[KnownEvent]) {
        let mut bed = TestBed::new();
        for known in events {
            match known {
                KnownEvent::Midi(ev) => bed.as_mut().push_event(ev.event()),
                KnownEvent::Midi2(ev) => bed.as_mut().push_event(ev.event()),
                KnownEvent::ParamMod(ev) => bed.as_mut().push_event(ev.event()),
                KnownEvent::ParamValue(ev) => bed.as_mut().push_event(ev.event()),
                KnownEvent::Transport(ev) => bed.as_mut().push_event(ev.event()),
            }
        }

        let input_events = bed.input_events();
        bed.retrieve_events(input_events);
    }

    #[test]
    fn retrieve_events_01() {
        let events = [KnownEvent::Midi(Midi::build().port_index(1))];
        check_input_events(&events);
    }

    #[test]
    fn retrieve_events_02() {
        let events = [
            KnownEvent::Midi(Midi::build().port_index(1)),
            KnownEvent::Midi2(Midi2::build().port_index(2)),
            KnownEvent::ParamMod(ParamMod::build().amount(12.345)),
            KnownEvent::ParamValue(ParamValue::build().value(12.34)),
            KnownEvent::Transport(Transport::build().bar_number(11)),
        ];
        check_input_events(&events);
    }

    #[test]
    fn retrieve_events_03() {
        let events = [
            KnownEvent::Midi(Midi::build().port_index(1)),
            KnownEvent::Midi2(Midi2::build().port_index(2)),
            KnownEvent::ParamMod(ParamMod::build().amount(12.345)),
            KnownEvent::ParamValue(ParamValue::build().value(12.34)),
            KnownEvent::Transport(Transport::build().bar_number(11)),
        ];
        let events: Vec<_> = (0..1000)
            .map(|i| events[(7 * i + 13) % events.len()])
            .collect();

        check_input_events(&events);
    }

    #[test]
    fn process_input_events() {
        let event = Midi2::build().port_index(2);
        let mut bed = TestBed::new();
        bed.as_mut().push_event(event.event());
        let mut clap_process = clap_process {
            steady_time: 0,
            frames_count: 0,
            transport: null(),
            audio_inputs: null(),
            audio_outputs: null_mut(),
            audio_inputs_count: 0,
            audio_outputs_count: 0,
            in_events: &raw const bed.clap_input_events,
            out_events: SHIM_CLAP_OUTPUT_EVENTS.as_ref(),
        };
        let process =
            unsafe { Process::new_unchecked(NonNull::new_unchecked(&raw mut clap_process)) };

        let input_events = process.in_events();
        assert_eq!(input_events.size(), 1);
        let retrieved = input_events.get(0);

        let retrieved = retrieved.midi2().unwrap();
        assert_eq!(retrieved.port_index(), event.event().port_index());
    }
}

mod output_events {
    use std::{
        pin::Pin,
        ptr,
        ptr::{NonNull, null, null_mut},
    };

    use clap_clap::{
        events::{
            EventBuilder, Header, Midi, Midi2, OutputEvents, ParamMod, ParamValue, Transport,
        },
        ffi::{
            CLAP_EVENT_MIDI, CLAP_EVENT_MIDI2, CLAP_EVENT_PARAM_MOD, CLAP_EVENT_PARAM_VALUE,
            CLAP_EVENT_TRANSPORT, clap_event_header, clap_event_midi, clap_event_midi2,
            clap_event_param_mod, clap_event_param_value, clap_event_transport, clap_output_events,
            clap_process,
        },
        prelude::Process,
    };

    use crate::{
        events::input_events::KnownEvent, shims::events::input_events::SHIM_CLAP_INPUT_EVENTS,
    };

    struct TestBed {
        clap_output_events: clap_output_events,
        pushed: Vec<Box<[u8]>>,
        capacity: usize,
    }

    impl TestBed {
        fn new(capacity: usize) -> Pin<Box<Self>> {
            extern "C-unwind" fn try_push(
                list: *const clap_output_events,
                event: *const clap_event_header,
            ) -> bool {
                let test_bed: &mut TestBed = unsafe { &mut *((*list).ctx as *mut _) };

                if test_bed.pushed.len() < test_bed.capacity {
                    let event = unsafe { &*event };
                    let header = unsafe { Header::new(event) };
                    let bytes = header.to_bytes().to_owned().into_boxed_slice();

                    test_bed.pushed.push(bytes);
                    true
                } else {
                    false
                }
            }

            let mut test_bed = Box::new(Self {
                clap_output_events: clap_output_events {
                    ctx: null_mut(),
                    try_push: Some(try_push),
                },
                pushed: vec![],
                capacity,
            });

            test_bed.clap_output_events.ctx = &raw mut *test_bed as *mut _;

            Box::into_pin(test_bed)
        }

        fn clap_output_events(&mut self) -> &clap_output_events {
            &self.clap_output_events
        }

        fn get_event(&self, index: usize) -> Option<KnownEvent> {
            assert!(index < self.pushed.len());

            let header: clap_event_header =
                unsafe { ptr::read_unaligned(self.pushed[index].as_ptr() as *const _) };
            assert_eq!(header.size, self.pushed[index].len() as u32);

            let x = header.r#type as u32;

            if x == CLAP_EVENT_PARAM_VALUE {
                let event: clap_event_param_value =
                    unsafe { ptr::read_unaligned(self.pushed[index].as_ptr() as *const _) };
                return Some(KnownEvent::ParamValue(event.into()));
            }
            if x == CLAP_EVENT_PARAM_MOD {
                let event: clap_event_param_mod =
                    unsafe { ptr::read_unaligned(self.pushed[index].as_ptr() as *const _) };
                return Some(KnownEvent::ParamMod(event.into()));
            }
            if x == CLAP_EVENT_TRANSPORT {
                let event: clap_event_transport =
                    unsafe { ptr::read_unaligned(self.pushed[index].as_ptr() as *const _) };
                return Some(KnownEvent::Transport(event.into()));
            }
            if x == CLAP_EVENT_MIDI {
                let event: clap_event_midi =
                    unsafe { ptr::read_unaligned(self.pushed[index].as_ptr() as *const _) };
                return Some(KnownEvent::Midi(event.into()));
            }
            if x == CLAP_EVENT_MIDI2 {
                let event: clap_event_midi2 =
                    unsafe { ptr::read_unaligned(self.pushed[index].as_ptr() as *const _) };
                return Some(KnownEvent::Midi2(event.into()));
            }
            None
        }
    }

    #[test]
    fn self_test_01() {
        let event = Midi::build().port_index(11);

        let mut test_bed = TestBed::new(1);
        let mut output_events =
            unsafe { OutputEvents::new_unchecked(test_bed.clap_output_events()) };
        output_events.try_push(event.event()).unwrap();

        let retrieved = test_bed.get_event(0).unwrap();
        let KnownEvent::Midi(midi) = retrieved else {
            panic!()
        };
        assert_eq!(midi.event().port_index(), 11);
    }

    #[should_panic]
    #[test]
    fn self_test_02() {
        let event = Midi::build().port_index(11);

        let mut test_bed = TestBed::new(1);
        let mut output_events =
            unsafe { OutputEvents::new_unchecked(test_bed.clap_output_events()) };
        output_events.try_push(event.event()).unwrap();

        let _ = test_bed.get_event(1).unwrap();
    }

    fn check_push_events(events: &[KnownEvent]) {
        let mut bed = TestBed::new(events.len());
        let mut output_events = unsafe { OutputEvents::new_unchecked(bed.clap_output_events()) };

        for known in events {
            match known {
                KnownEvent::Midi(ev) => output_events.try_push(ev.event()).unwrap(),
                KnownEvent::Midi2(ev) => output_events.try_push(ev.event()).unwrap(),
                KnownEvent::ParamMod(ev) => output_events.try_push(ev.event()).unwrap(),
                KnownEvent::ParamValue(ev) => output_events.try_push(ev.event()).unwrap(),
                KnownEvent::Transport(ev) => output_events.try_push(ev.event()).unwrap(),
            }
        }

        let event = Midi::build();
        let _ = output_events.try_push(event.event()).unwrap_err();

        let retrieved: Vec<KnownEvent> = (0..events.len())
            .map(|i| bed.get_event(i).unwrap())
            .collect();

        assert_eq!(events, retrieved);
    }

    #[test]
    fn push_events_01() {
        let events = [
            KnownEvent::Midi(Midi::build().port_index(1)),
            KnownEvent::Midi2(Midi2::build().port_index(2)),
            KnownEvent::ParamMod(ParamMod::build().amount(12.345)),
            KnownEvent::ParamValue(ParamValue::build().value(12.34)),
            KnownEvent::Transport(Transport::build().bar_number(11)),
        ];

        check_push_events(&events);
    }

    #[test]
    fn push_events_02() {
        let events = [
            KnownEvent::Midi(Midi::build().port_index(1)),
            KnownEvent::Midi2(Midi2::build().port_index(2)),
            KnownEvent::ParamMod(ParamMod::build().amount(12.345)),
            KnownEvent::ParamValue(ParamValue::build().value(12.34)),
            KnownEvent::Transport(Transport::build().bar_number(11)),
        ];

        let events: Vec<_> = (0..1000)
            .map(|i| events[(13 * i + 27) % events.len()])
            .collect();
        check_push_events(&events);
    }

    #[test]
    fn process_out_events() {
        let event = Midi::build().port_index(34);
        let mut bed = TestBed::new(1);
        let mut clap_process = clap_process {
            steady_time: 0,
            frames_count: 0,
            transport: null(),
            audio_inputs: null(),
            audio_outputs: null_mut(),
            audio_inputs_count: 0,
            audio_outputs_count: 0,
            in_events: SHIM_CLAP_INPUT_EVENTS.as_ref(),
            out_events: bed.clap_output_events(),
        };
        let process =
            unsafe { Process::new_unchecked(NonNull::new(&raw mut clap_process).unwrap()) };

        let mut output_events = process.out_events();
        output_events.try_push(event.event()).unwrap();

        let KnownEvent::Midi(retrieved) = bed.get_event(0).unwrap() else {
            panic!()
        };

        assert_eq!(retrieved, event);
    }
}
