use std::ptr::null_mut;

use clap_clap::{
    events::{Event, ParamMod},
    ffi::{CLAP_EVENT_PARAM_MOD, clap_event_header, clap_event_param_mod},
};

mod cast_clap_event_note {
    use clap_clap::{
        events::{Event, Note},
        ffi::{
            CLAP_EVENT_NOTE_CHOKE, CLAP_EVENT_NOTE_END, CLAP_EVENT_NOTE_OFF, CLAP_EVENT_NOTE_ON,
            clap_event_header, clap_event_note,
        },
    };

    fn build_clap_event_note(ev_type: u16) -> clap_event_note {
        assert!(size_of::<clap_event_note>() < u32::MAX as usize);
        clap_event_note {
            header: clap_event_header {
                size: size_of::<clap_event_note>() as u32,
                time: 1,
                space_id: 0,
                r#type: ev_type,
                flags: 3,
            },
            note_id: 1,
            port_index: 2,
            channel: 3,
            key: 4,
            velocity: 5.0,
        }
    }

    macro_rules! check_cast_clap_event_note {
        ($name:tt, $clap_event:ident, $ev:ident) => {
            #[test]
            fn $name() {
                assert_eq!($clap_event as u16 as u64, $clap_event as u64);
                let clap_event = build_clap_event_note($clap_event as u16);
                let expected = Note::from(clap_event);

                let ev = unsafe { Event::cast_and_copy_clap_event(&clap_event.header) }.unwrap();
                let Event::$ev(event) = ev else { panic!() };

                assert_eq!(event, expected);
            }
        };
    }

    check_cast_clap_event_note!(note_on, CLAP_EVENT_NOTE_ON, NoteOn);
    check_cast_clap_event_note!(note_off, CLAP_EVENT_NOTE_OFF, NoteOff);
    check_cast_clap_event_note!(note_choke, CLAP_EVENT_NOTE_CHOKE, NoteChoke);
    check_cast_clap_event_note!(note_end, CLAP_EVENT_NOTE_END, NoteEnd);
}

mod cast_clap_event_note_expression {
    use clap_clap::{
        events::{Event, Expression, NoteExpression},
        ffi::{
            CLAP_EVENT_NOTE_EXPRESSION, CLAP_NOTE_EXPRESSION_BRIGHTNESS,
            CLAP_NOTE_EXPRESSION_EXPRESSION, CLAP_NOTE_EXPRESSION_PAN,
            CLAP_NOTE_EXPRESSION_PRESSURE, CLAP_NOTE_EXPRESSION_TUNING,
            CLAP_NOTE_EXPRESSION_VIBRATO, CLAP_NOTE_EXPRESSION_VOLUME, clap_event_header,
            clap_event_note_expression,
        },
    };

    #[test]
    fn clap_event_note_expr_cast_u16() {
        assert_eq!(
            CLAP_EVENT_NOTE_EXPRESSION as u16 as u64,
            CLAP_EVENT_NOTE_EXPRESSION as u64
        );
    }

    fn build_clap_event_note_expression(expr_id: i32) -> clap_event_note_expression {
        assert!(size_of::<clap_event_note_expression>() < u32::MAX as usize);
        clap_event_note_expression {
            header: clap_event_header {
                size: size_of::<clap_event_note_expression>() as u32,
                time: 0,
                space_id: 0,
                r#type: CLAP_EVENT_NOTE_EXPRESSION as u16,
                flags: 0,
            },
            expression_id: expr_id,
            note_id: 1,
            port_index: 2,
            channel: 3,
            key: 4,
            value: 5.0,
        }
    }

    macro_rules! cast_clap_note_expression {
        ($name:tt, $clap_expr:tt, $expression:tt) => {
            #[test]
            fn $name() {
                let ne = build_clap_event_note_expression($clap_expr);
                let expr_exp = Expression::from(ne);

                let ev = unsafe { Event::cast_and_copy_clap_event(&ne.header) }.unwrap();
                let Event::NoteExpression(expr) = ev else {
                    panic!()
                };
                let NoteExpression::$expression(expr) = expr else {
                    panic!()
                };

                assert_eq!(expr, expr_exp);
            }
        };
    }

    cast_clap_note_expression!(volume, CLAP_NOTE_EXPRESSION_VOLUME, Volume);
    cast_clap_note_expression!(pan, CLAP_NOTE_EXPRESSION_PAN, Pan);
    cast_clap_note_expression!(tuning, CLAP_NOTE_EXPRESSION_TUNING, Tuning);
    cast_clap_note_expression!(vibrato, CLAP_NOTE_EXPRESSION_VIBRATO, Vibrato);
    cast_clap_note_expression!(expression, CLAP_NOTE_EXPRESSION_EXPRESSION, Expression);
    cast_clap_note_expression!(brightness, CLAP_NOTE_EXPRESSION_BRIGHTNESS, Brightness);
    cast_clap_note_expression!(pressure, CLAP_NOTE_EXPRESSION_PRESSURE, Pressure);
}

mod cast_clap_event_param_value {
    use std::ptr::null_mut;

    use clap_clap::{
        events::{Event, ParamValue},
        ffi::{CLAP_EVENT_PARAM_VALUE, clap_event_header, clap_event_param_value},
    };

    #[test]
    fn cast_clap_event_param_value() {
        assert!(size_of::<clap_event_param_value>() < u32::MAX as usize);
        assert!((CLAP_EVENT_PARAM_VALUE as u64) < u16::MAX as u64);
        let clap_event = clap_event_param_value {
            header: clap_event_header {
                size: size_of::<clap_event_param_value>() as u32,
                time: 1,
                space_id: 2,
                r#type: CLAP_EVENT_PARAM_VALUE as u16,
                flags: 0,
            },
            param_id: 1,
            cookie: null_mut(),
            note_id: 2,
            port_index: 3,
            channel: 4,
            key: 5,
            value: 6.0,
        };
        let expected = ParamValue::from(clap_event);

        let ev = unsafe { Event::cast_and_copy_clap_event(&clap_event.header) }.unwrap();
        let Event::ParamValue(event) = ev else {
            panic!()
        };

        assert_eq!(event, expected);
    }
}

mod cast_clap_event_param_mod {
    use super::*;

    #[test]
    fn cast_clap_event_param_mod() {
        assert!(size_of::<clap_event_param_mod>() < u32::MAX as usize);
        assert!((CLAP_EVENT_PARAM_MOD as u64) < u16::MAX as u64);
        let clap_event = clap_event_param_mod {
            header: clap_event_header {
                size: size_of::<clap_event_param_mod>() as u32,
                time: 1,
                space_id: 2,
                r#type: CLAP_EVENT_PARAM_MOD as u16,
                flags: 0,
            },
            param_id: 1,
            cookie: null_mut(),
            note_id: 2,
            port_index: 3,
            channel: 4,
            key: 5,
            amount: 6.0,
        };
        let expected = ParamMod::from(clap_event);

        let ev = unsafe { Event::cast_and_copy_clap_event(&clap_event.header) }.unwrap();
        let Event::ParamMod(event) = ev else { panic!() };

        assert_eq!(event, expected);
    }
}

mod cast_clap_event_param_gesture {
    use clap_clap::{
        events::{Event, ParamGesture},
        ffi::{
            CLAP_EVENT_PARAM_GESTURE_BEGIN, CLAP_EVENT_PARAM_GESTURE_END, clap_event_header,
            clap_event_param_gesture,
        },
    };

    fn build_clap_event_param_gesture(ev_type: u16) -> clap_event_param_gesture {
        assert!(size_of::<clap_event_param_gesture>() < u32::MAX as usize);
        clap_event_param_gesture {
            header: clap_event_header {
                size: size_of::<clap_event_param_gesture>() as u32,
                time: 1,
                space_id: 0,
                r#type: ev_type,
                flags: 3,
            },
            param_id: 55,
        }
    }

    macro_rules! check_cast_clap_event_param_gesture {
        ($name:tt, $clap_event:ident, $ev:ident) => {
            #[test]
            fn $name() {
                assert_eq!($clap_event as u16 as u64, $clap_event as u64);
                let clap_event = build_clap_event_param_gesture($clap_event as u16);
                let expected = ParamGesture::from(clap_event);

                let ev = unsafe { Event::cast_and_copy_clap_event(&clap_event.header) }.unwrap();
                let Event::$ev(event) = ev else { panic!() };

                assert_eq!(event, expected);
            }
        };
    }

    check_cast_clap_event_param_gesture!(
        gesture_begin,
        CLAP_EVENT_PARAM_GESTURE_BEGIN,
        ParamGestureBegin
    );
    check_cast_clap_event_param_gesture!(
        gesture_end,
        CLAP_EVENT_PARAM_GESTURE_END,
        ParamGestureEnd
    );
}

mod cast_clap_event_transport {
    use clap_clap::{
        events::{Event, Transport},
        ffi::{CLAP_EVENT_TRANSPORT, clap_event_header, clap_event_transport},
    };

    #[test]
    fn cast_clap_event_transport() {
        assert!(size_of::<clap_event_transport>() < u32::MAX as usize);
        assert!((CLAP_EVENT_TRANSPORT as u64) < u16::MAX as u64);
        let clap_event = clap_event_transport {
            header: clap_event_header {
                size: size_of::<clap_event_transport>() as u32,
                time: 1,
                space_id: 2,
                r#type: CLAP_EVENT_TRANSPORT as u16,
                flags: 0,
            },
            flags: 1,
            song_pos_beats: 2,
            song_pos_seconds: 3,
            tempo: 4.0,
            tempo_inc: 5.0,
            loop_start_beats: 6,
            loop_end_beats: 7,
            loop_start_seconds: 8,
            loop_end_seconds: 9,
            bar_start: 10,
            bar_number: 11,
            tsig_num: 12,
            tsig_denom: 13,
        };
        let expected = Transport::from(clap_event);

        let ev = unsafe { Event::cast_and_copy_clap_event(&clap_event.header) }.unwrap();
        let Event::Transport(event) = ev else {
            panic!("wrong cast")
        };

        assert_eq!(event, expected);
    }
}

mod cast_clap_event_midi {
    use clap_clap::{
        events::{Event, Midi},
        ffi::{CLAP_EVENT_MIDI, clap_event_header, clap_event_midi},
    };

    #[test]
    fn cast_clap_event_midi() {
        assert!(size_of::<clap_event_midi>() < u32::MAX as usize);
        assert!((CLAP_EVENT_MIDI as u64) < u16::MAX as u64);
        let clap_event = clap_event_midi {
            header: clap_event_header {
                size: size_of::<clap_event_midi>() as u32,
                time: 1,
                space_id: 2,
                r#type: CLAP_EVENT_MIDI as u16,
                flags: 0,
            },
            port_index: 2,
            data: [1, 2, 3],
        };
        let expected = Midi::from(clap_event);

        let ev = unsafe { Event::cast_and_copy_clap_event(&clap_event.header) }.unwrap();
        let Event::Midi(event) = ev else {
            panic!("wrong cast")
        };

        assert_eq!(event, expected);
    }
}

mod cast_clap_event_midi2 {
    use clap_clap::{
        events::{Event, Midi2},
        ffi::{CLAP_EVENT_MIDI2, clap_event_header, clap_event_midi2},
    };

    #[test]
    fn cast_clap_event_midi2() {
        assert!(size_of::<clap_event_midi2>() < u32::MAX as usize);
        assert!((CLAP_EVENT_MIDI2 as u64) < u16::MAX as u64);
        let clap_event = clap_event_midi2 {
            header: clap_event_header {
                size: size_of::<clap_event_midi2>() as u32,
                time: 1,
                space_id: 2,
                r#type: CLAP_EVENT_MIDI2 as u16,
                flags: 0,
            },
            port_index: 2,
            data: [1, 2, 3, 4],
        };
        let expected = Midi2::from(clap_event);

        let ev = unsafe { Event::cast_and_copy_clap_event(&clap_event.header) }.unwrap();
        let Event::Midi2(event) = ev else {
            panic!("wrong cast")
        };

        assert_eq!(event, expected);
    }
}

mod cast_clap_event_midi_sysex {
    use std::ptr::null;

    use clap_clap::{
        events::{Event, MidiSysex},
        ffi::{CLAP_EVENT_MIDI_SYSEX, clap_event_header, clap_event_midi_sysex},
    };

    #[test]
    fn cast_clap_event_midi2() {
        assert!(size_of::<clap_event_midi_sysex>() < u32::MAX as usize);
        assert!((CLAP_EVENT_MIDI_SYSEX as u64) < u16::MAX as u64);
        let clap_event = clap_event_midi_sysex {
            header: clap_event_header {
                size: size_of::<clap_event_midi_sysex>() as u32,
                time: 1,
                space_id: 2,
                r#type: CLAP_EVENT_MIDI_SYSEX as u16,
                flags: 0,
            },
            port_index: 2,
            buffer: null(),
            size: 0,
        };
        let expected = MidiSysex::from(clap_event);

        let ev = unsafe { Event::cast_and_copy_clap_event(&clap_event.header) }.unwrap();
        let Event::MidiSysex(event) = ev else {
            panic!("wrong cast")
        };

        assert_eq!(event, expected);
    }
}
