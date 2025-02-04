use std::{pin::Pin, ptr::NonNull};

use clap_clap::process::Process;

use crate::TestProcess;

fn frames_input_data32(mut test_process: Pin<Box<TestProcess>>) {
    let num_in = test_process.audio_inputs_count;
    let num_ch = test_process.audio_inputs[0].channel_count;

    for pt in 0..num_in as usize {
        for ch in 0..num_ch as usize {
            for i in 0..test_process.frames_count as usize {
                test_process.audio_inputs[pt].data32[ch].0[i] = (pt * ch * i) as f32;
            }
        }
    }

    let mut clap_process = test_process.clap_process();
    let mut process =
        unsafe { Process::new_unchecked(NonNull::new_unchecked(&raw mut clap_process)) };

    let mut i = 0;
    let mut frames = process.frames();
    while let Some(frame) = frames.next() {
        for pt in 0..num_in {
            for ch in 0..num_ch {
                assert_eq!(frame.audio_input(pt).data32(ch), (pt * ch * i) as f32);
            }
        }
        i += 1;
    }
}

macro_rules! case_frames_input_data32 {
    ($name:ident, $num_ins:literal, $num_chan:literal) => {
        mod $name {
            use super::*;
            use crate::TestProcessConfig;

            #[test]
            fn outputs0() {
                let test_process = TestProcessConfig {
                    latency: 0,
                    steady_time: 0,
                    frames_count: 10,
                    channel_count: $num_chan,
                    audio_inputs_count: $num_ins,
                    audio_outputs_count: 0,
                }
                .build();

                frames_input_data32(test_process);
            }

            #[test]
            fn outputs0_noframe() {
                let test_process = TestProcessConfig {
                    latency: 0,
                    steady_time: 0,
                    frames_count: 0,
                    channel_count: $num_chan,
                    audio_inputs_count: $num_ins,
                    audio_outputs_count: 0,
                }
                .build();

                frames_input_data32(test_process);
            }

            #[test]
            fn outputs1() {
                let test_process = TestProcessConfig {
                    latency: 0,
                    steady_time: 0,
                    frames_count: 10,
                    channel_count: $num_chan,
                    audio_inputs_count: $num_ins,
                    audio_outputs_count: 1,
                }
                .build();

                frames_input_data32(test_process);
            }

            #[test]
            fn outputs1_long_frames() {
                let test_process = TestProcessConfig {
                    latency: 0,
                    steady_time: 0,
                    frames_count: 65536,
                    channel_count: $num_chan,
                    audio_inputs_count: $num_ins,
                    audio_outputs_count: 1,
                }
                .build();

                frames_input_data32(test_process);
            }
        }
    };
}

case_frames_input_data32!(frames_input_data32_01, 1, 1);
case_frames_input_data32!(frames_input_data32_02, 2, 1);
case_frames_input_data32!(frames_input_data32_03, 1, 2);
case_frames_input_data32!(frames_input_data32_04, 2, 2);
case_frames_input_data32!(frames_input_data32_05, 3, 1);
case_frames_input_data32!(frames_input_data32_06, 1, 3);

macro_rules! case_frames_output_data32 {
    ($name:ident, $num_outs:literal, $num_chan:literal) => {
        mod $name {
            use super::*;
            use crate::TestProcessConfig;

            #[test]
            fn inputs0() {
                let test_process = TestProcessConfig {
                    latency: 0,
                    steady_time: 0,
                    frames_count: 10,
                    channel_count: $num_chan,
                    audio_inputs_count: 0,
                    audio_outputs_count: $num_outs,
                }
                .build();

                frames_output_data32(test_process);
            }

            #[test]
            fn inputs0_noframe() {
                let test_process = TestProcessConfig {
                    latency: 0,
                    steady_time: 0,
                    frames_count: 0,
                    channel_count: $num_chan,
                    audio_inputs_count: 0,
                    audio_outputs_count: $num_outs,
                }
                .build();

                frames_output_data32(test_process);
            }

            #[test]
            fn inputs1() {
                let test_process = TestProcessConfig {
                    latency: 0,
                    steady_time: 0,
                    frames_count: 1024,
                    channel_count: $num_chan,
                    audio_inputs_count: 1,
                    audio_outputs_count: $num_outs,
                }
                .build();

                frames_output_data32(test_process);
            }

            #[test]
            fn inputs1_long_frames() {
                let test_process = TestProcessConfig {
                    latency: 0,
                    steady_time: 0,
                    frames_count: 65536,
                    channel_count: $num_chan,
                    audio_inputs_count: 1,
                    audio_outputs_count: $num_outs,
                }
                .build();

                frames_output_data32(test_process);
            }
        }
    };
}

fn frames_output_data32(mut test_process: Pin<Box<TestProcess>>) {
    let num_out = test_process.audio_outputs_count;
    let num_ch = test_process.audio_outputs[0].channel_count;

    let mut clap_process = test_process.clap_process();
    let mut process =
        unsafe { Process::new_unchecked(NonNull::new_unchecked(&raw mut clap_process)) };

    let mut i = 0;
    let mut frames = process.frames();
    while let Some(frame) = frames.next() {
        for pt in 0..num_out {
            for ch in 0..num_ch {
                *frame.audio_output(pt).data32(ch) = (pt * ch * i) as f32;
            }
        }
        i += 1;
    }

    for pt in 0..num_out as usize {
        for ch in 0..num_ch as usize {
            for i in 0..test_process.frames_count as usize {
                assert_eq!(
                    test_process.audio_outputs[pt].data32[ch].0[i],
                    (pt * ch * i) as f32
                );
            }
        }
    }
}

case_frames_output_data32!(frames_output_data32_01, 1, 1);
case_frames_output_data32!(frames_output_data32_02, 2, 1);
case_frames_output_data32!(frames_output_data32_03, 1, 2);
case_frames_output_data32!(frames_output_data32_04, 2, 2);
case_frames_output_data32!(frames_output_data32_05, 3, 1);
case_frames_output_data32!(frames_output_data32_06, 1, 3);
