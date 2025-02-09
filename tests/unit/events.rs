use std::ptr::null_mut;

use clap_clap::{
    events::{Event, ParamMod, ParamValue},
    ffi::{
        CLAP_EVENT_NOTE_CHOKE, CLAP_EVENT_NOTE_END, CLAP_EVENT_NOTE_OFF, CLAP_EVENT_NOTE_ON,
        CLAP_EVENT_PARAM_MOD, CLAP_EVENT_PARAM_VALUE, clap_event_header, clap_event_note,
        clap_event_param_mod, clap_event_param_value,
    },
};

mod cast_clap_event_note {
    use clap_clap::events::{Event, Note};

    use super::*;

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
                let note = build_clap_event_note($clap_event as u16);
                let note_exp = Note::from(note);

                let ev = unsafe { Event::cast_and_copy_clap_event(&note.header) }.unwrap();
                let Event::$ev(note) = ev else { panic!() };

                assert_eq!(note, note_exp);
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
            CLAP_NOTE_EXPRESSION_VIBRATO, CLAP_NOTE_EXPRESSION_VOLUME, clap_event_note_expression,
        },
    };

    use super::*;

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

#[test]
fn cast_clap_event_param_value() {
    assert!(size_of::<clap_event_param_value>() < u32::MAX as usize);
    assert!((CLAP_EVENT_PARAM_VALUE as u64) < u16::MAX as u64);
    let pv = clap_event_param_value {
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
    let pv_exp = ParamValue::from(pv);

    let ev = unsafe { Event::cast_and_copy_clap_event(&pv.header) }.unwrap();
    let Event::ParamValue(pv) = ev else { panic!() };

    assert_eq!(pv, pv_exp);
}

#[test]
fn cast_clap_event_param_mod() {
    assert!(size_of::<clap_event_param_mod>() < u32::MAX as usize);
    assert!((CLAP_EVENT_PARAM_MOD as u64) < u16::MAX as u64);
    let pm = clap_event_param_mod {
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
    let pm_exp = ParamMod::from(pm);

    let ev = unsafe { Event::cast_and_copy_clap_event(&pm.header) }.unwrap();
    let Event::ParamMod(pm) = ev else { panic!() };

    assert_eq!(pm, pm_exp);
}

mod cast_clap_event_param_gesture {
    use clap_clap::{
        events::{Event, ParamGesture},
        ffi::{
            CLAP_EVENT_PARAM_GESTURE_BEGIN, CLAP_EVENT_PARAM_GESTURE_END, clap_event_param_gesture,
        },
    };

    use super::*;

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
                let gesture = build_clap_event_param_gesture($clap_event as u16);
                let gesture_exp = ParamGesture::from(gesture);

                let ev = unsafe { Event::cast_and_copy_clap_event(&gesture.header) }.unwrap();
                let Event::$ev(gesture) = ev else { panic!() };

                assert_eq!(gesture, gesture_exp);
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
