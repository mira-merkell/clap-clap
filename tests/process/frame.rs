use std::{hash::Hasher, pin::Pin, ptr::NonNull};

use clap_clap::{
    plugin::{AudioThread, Plugin},
    process::Process,
};

use crate::{TestProcess, TestProcessConfig};

mod data32;

fn frames_init(mut test_process: Pin<Box<TestProcess>>) {
    let num_frames = test_process.frames_count;
    let audio_in = test_process.audio_inputs_count;
    let audio_out = test_process.audio_outputs_count;
    let num_chan = test_process.audio_inputs[0].channel_count;

    let mut clap_process = test_process.clap_process();
    let mut process =
        unsafe { Process::new_unchecked(NonNull::new_unchecked(&raw mut clap_process)) };

    let mut frames = process.frames();
    for _ in 0..num_frames {
        let frame = frames.next().expect("the number of frames doesn't match");

        for k in 0..audio_in {
            assert_eq!(frame.audio_input(k).channel_count(), num_chan);
        }
        for k in 0..audio_out {
            assert_eq!(frame.audio_output(k).channel_count(), num_chan);
        }
    }

    let frame = frames.next();
    assert!(frame.is_none());
}

macro_rules! case_frames_init {
    ($name:ident, $num_frames:literal, $num_chan:literal, $audio_in:literal, $audio_out:literal) => {
        #[test]
        fn $name() {
            let test_process = TestProcessConfig {
                latency: 0,
                steady_time: 0,
                frames_count: $num_frames,
                channel_count: $num_chan,
                audio_inputs_count: $audio_in,
                audio_outputs_count: $audio_out,
            }
            .build();

            frames_init(test_process);
        }
    };
}

case_frames_init!(frames_init_01, 1, 1, 1, 1);
case_frames_init!(frames_init_02, 1, 2, 3, 4);
case_frames_init!(frames_init_03, 1024, 2, 2, 2);
case_frames_init!(frames_init_04, 8, 7, 10, 20);
