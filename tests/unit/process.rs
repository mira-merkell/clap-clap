use std::{
    pin::Pin,
    ptr::{NonNull, null},
};

use clap_clap::{
    ffi::{clap_audio_buffer, clap_event_transport, clap_process},
    process::Process,
};

mod frame;

trait Float: Copy + Clone + Default + PartialEq {}
impl Float for f32 {}
impl Float for f64 {}

#[derive(Debug, PartialEq, Clone)]
struct TestChannel<T: Float>(Vec<T>);

impl<T: Float> TestChannel<T> {
    fn new(n: u32) -> Self {
        let n = n.try_into().unwrap();
        Self(vec![T::default(); n])
    }

    fn as_mut_ptr(&mut self) -> *mut T {
        self.0.as_mut_ptr()
    }

    fn data(&mut self) -> &mut [T] {
        &mut self.0
    }
}

#[derive(Debug)]
struct TestAudioBuffer {
    data32: Vec<TestChannel<f32>>,
    data64: Vec<TestChannel<f64>>,
    channel_count: u32,
    latency: u32,
    constant_mask: u64,

    raw_data32: Vec<*mut f32>,
    raw_data64: Vec<*mut f64>,
}

impl TestAudioBuffer {
    fn new(n: u32, channel_count: u32, latency: u32) -> Pin<Box<Self>> {
        let mut data32 = vec![TestChannel::new(n); channel_count as usize];
        let mut data64 = vec![TestChannel::new(n); channel_count as usize];
        let raw_data32 = data32.iter_mut().map(|ch| ch.as_mut_ptr()).collect();
        let raw_data64 = data64.iter_mut().map(|ch| ch.as_mut_ptr()).collect();

        Box::pin(Self {
            data32,
            data64,
            channel_count,
            latency,
            constant_mask: 0,
            raw_data32,
            raw_data64,
        })
    }

    fn clap_audio_buffer(self: &mut Pin<Box<Self>>) -> clap_audio_buffer {
        clap_audio_buffer {
            data32: self.raw_data32.as_mut_ptr(),
            data64: self.raw_data64.as_mut_ptr(),
            channel_count: self.channel_count,
            latency: self.latency,
            constant_mask: self.constant_mask,
        }
    }
}

#[derive(Debug)]
pub struct TestProcess {
    steady_time: i64,
    frames_count: u32,
    transport: *const clap_event_transport, // not implemented
    audio_inputs: Vec<Pin<Box<TestAudioBuffer>>>,
    audio_outputs: Vec<Pin<Box<TestAudioBuffer>>>,
    audio_inputs_count: u32,
    audio_outputs_count: u32,

    raw_audio_inputs: Vec<clap_audio_buffer>,
    raw_audio_outputs: Vec<clap_audio_buffer>,
}

impl TestProcess {
    fn new(config: TestProcessConfig) -> Pin<Box<Self>> {
        let mut audio_inputs: Vec<_> = (0..config.audio_inputs_count)
            .map(|_| {
                TestAudioBuffer::new(config.frames_count, config.channel_count, config.latency)
            })
            .collect();
        let raw_audio_inputs = audio_inputs
            .iter_mut()
            .map(|ai| ai.clap_audio_buffer())
            .collect();

        let mut audio_outputs: Vec<_> = (0..config.audio_outputs_count)
            .map(|_| {
                TestAudioBuffer::new(config.frames_count, config.channel_count, config.latency)
            })
            .collect();
        let raw_audio_outputs = audio_outputs
            .iter_mut()
            .map(|ao| ao.clap_audio_buffer())
            .collect();

        Box::pin(Self {
            steady_time: config.steady_time,
            frames_count: config.frames_count,
            transport: null(),
            audio_inputs,
            audio_outputs,
            audio_inputs_count: config.audio_inputs_count,
            audio_outputs_count: config.audio_outputs_count,
            raw_audio_inputs,
            raw_audio_outputs,
        })
    }

    pub fn clap_process(self: &mut Pin<Box<Self>>) -> clap_process {
        clap_process {
            steady_time: self.steady_time,
            frames_count: self.frames_count,
            transport: self.transport,
            audio_inputs: self.raw_audio_inputs.as_ptr(),
            audio_outputs: self.raw_audio_outputs.as_mut_ptr(),
            audio_inputs_count: self.audio_inputs_count,
            audio_outputs_count: self.audio_outputs_count,
            in_events: null(),
            out_events: null(),
        }
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct TestProcessConfig {
    pub latency: u32,
    pub steady_time: i64,
    pub frames_count: u32,
    pub channel_count: u32,
    pub audio_inputs_count: u32,
    pub audio_outputs_count: u32,
}

impl TestProcessConfig {
    pub fn build(self) -> Pin<Box<TestProcess>> {
        TestProcess::new(self)
    }
}

#[test]
fn self_test_01() {
    let process = TestProcessConfig {
        latency: 0,
        steady_time: 1,
        frames_count: 2,
        channel_count: 3,
        audio_inputs_count: 4,
        audio_outputs_count: 5,
    }
    .build();

    assert_eq!(process.audio_inputs[0].latency, 0);
    assert_eq!(process.audio_outputs[0].latency, 0);
    assert_eq!(process.steady_time, 1);
    assert_eq!(process.frames_count, 2);
    assert_eq!(process.audio_inputs[0].channel_count, 3);
    assert_eq!(process.audio_outputs[0].channel_count, 3);
    assert_eq!(process.audio_inputs_count, 4);
    assert_eq!(process.audio_inputs.len(), 4);
    assert_eq!(process.audio_outputs_count, 5);
    assert_eq!(process.audio_outputs.len(), 5);
}

#[test]
fn self_test_02() {
    let mut process = TestProcessConfig {
        latency: 0,
        steady_time: 1,
        frames_count: 2,
        channel_count: 3,
        audio_inputs_count: 4,
        audio_outputs_count: 5,
    }
    .build();

    let clap_process = process.clap_process();
    assert_eq!(clap_process.steady_time, 1);
    assert_eq!(clap_process.frames_count, 2);
    assert_eq!(clap_process.audio_inputs_count, 4);
    assert_eq!(clap_process.audio_outputs_count, 5);
}

#[test]
fn self_test_03_32() {
    let mut process = TestProcessConfig {
        latency: 0,
        steady_time: 1,
        frames_count: 2,
        channel_count: 3,
        audio_inputs_count: 4,
        audio_outputs_count: 5,
    }
    .build();

    process.audio_inputs[0].data32[0].data()[0] = 11.13;
    process.audio_outputs[2].data32[2].data()[1] = 0.777;

    let clap_process = process.clap_process();

    let in0 = unsafe { *clap_process.audio_inputs.add(0) };
    let in0_ch0 = unsafe { *in0.data32.add(0) };
    let sample = unsafe { *in0_ch0.add(0) };
    assert_eq!(sample, 11.13);

    let out2 = unsafe { *clap_process.audio_outputs.add(2) };
    let out2_ch2 = unsafe { *out2.data32.add(2) };
    let sample = unsafe { *out2_ch2.add(1) };
    assert_eq!(sample, 0.777);
}

#[test]
fn self_test_04_64() {
    let mut process = TestProcessConfig {
        latency: 0,
        steady_time: 1,
        frames_count: 2,
        channel_count: 3,
        audio_inputs_count: 4,
        audio_outputs_count: 5,
    }
    .build();

    process.audio_inputs[0].data64[0].data()[0] = 11.13;
    process.audio_outputs[2].data64[2].data()[1] = 0.777;

    let clap_process = process.clap_process();

    let in0 = unsafe { *clap_process.audio_inputs.add(0) };
    let in0_ch0 = unsafe { *in0.data64.add(0) };
    let sample = unsafe { *in0_ch0.add(0) };
    assert_eq!(sample, 11.13);

    let out2 = unsafe { *clap_process.audio_outputs.add(2) };
    let out2_ch2 = unsafe { *out2.data64.add(2) };
    let sample = unsafe { *out2_ch2.add(1) };
    assert_eq!(sample, 0.777);
}

#[test]
fn process_new() {
    let mut test_process = TestProcessConfig {
        latency: 0,
        steady_time: 1,
        frames_count: 2,
        channel_count: 3,
        audio_inputs_count: 4,
        audio_outputs_count: 5,
    }
    .build();

    let mut clap_process = test_process.clap_process();
    let mut process =
        unsafe { Process::new_unchecked(NonNull::new_unchecked(&raw mut clap_process)) };

    assert_eq!(process.steady_time(), test_process.steady_time);
    assert_eq!(process.frames_count(), test_process.frames_count);
    assert_eq!(
        process.audio_inputs_count(),
        test_process.audio_inputs_count
    );
    assert_eq!(
        process.audio_outputs_count(),
        test_process.audio_outputs_count
    );

    assert_eq!(
        process.audio_inputs(0).latency(),
        test_process.audio_inputs[0].latency
    );
    assert_eq!(
        process.audio_inputs(0).channel_count(),
        test_process.audio_inputs[0].channel_count
    );

    assert_eq!(
        process.audio_outputs(0).latency(),
        test_process.audio_outputs[0].latency
    );
    assert_eq!(
        process.audio_outputs(0).channel_count(),
        test_process.audio_outputs[0].channel_count
    );
}

#[test]
#[should_panic(
    expected = "audio input number must be less than the number of available input ports"
)]
fn audio_input_wrong_no() {
    let mut test_process = TestProcessConfig {
        latency: 0,
        steady_time: 0,
        frames_count: 2,
        channel_count: 1,
        audio_inputs_count: 1,
        audio_outputs_count: 0,
    }
    .build();
    let mut clap_process = test_process.clap_process();
    let process = unsafe { Process::new_unchecked(NonNull::new_unchecked(&raw mut clap_process)) };

    let _ = process.audio_inputs(2);
}

#[test]
fn audio_input_data32() {
    let mut test_process = TestProcessConfig {
        latency: 0,
        steady_time: 0,
        frames_count: 2,
        channel_count: 1,
        audio_inputs_count: 1,
        audio_outputs_count: 0,
    }
    .build();

    test_process.audio_inputs[0].data32[0].data()[0] = 0.1;
    test_process.audio_inputs[0].data32[0].data()[1] = 0.2;

    let mut clap_process = test_process.clap_process();
    let process = unsafe { Process::new_unchecked(NonNull::new_unchecked(&raw mut clap_process)) };

    let in0 = process.audio_inputs(0);
    assert_eq!(in0.data32(0)[0], 0.1);
    assert_eq!(in0.data32(0)[1], 0.2);
}

#[test]
fn audio_input_data64() {
    let mut test_process = TestProcessConfig {
        latency: 0,
        steady_time: 0,
        frames_count: 2,
        channel_count: 1,
        audio_inputs_count: 1,
        audio_outputs_count: 0,
    }
    .build();

    test_process.audio_inputs[0].data64[0].data()[0] = 0.1;
    test_process.audio_inputs[0].data64[0].data()[1] = 0.2;

    let mut clap_process = test_process.clap_process();
    let process = unsafe { Process::new_unchecked(NonNull::new_unchecked(&raw mut clap_process)) };

    let in0 = process.audio_inputs(0);
    assert_eq!(in0.data64(0)[0], 0.1);
    assert_eq!(in0.data64(0)[1], 0.2);
}

#[test]
#[should_panic(
    expected = "audio output number must be less than the number of available output ports"
)]
fn audio_output_wrong_no() {
    let mut test_process = TestProcessConfig {
        latency: 0,
        steady_time: 0,
        frames_count: 2,
        channel_count: 1,
        audio_inputs_count: 1,
        audio_outputs_count: 0,
    }
    .build();

    let mut clap_process = test_process.clap_process();
    let mut process =
        unsafe { Process::new_unchecked(NonNull::new_unchecked(&raw mut clap_process)) };

    let _ = process.audio_outputs(0);
}

#[test]
fn audio_output_data32() {
    let mut test_process = TestProcessConfig {
        latency: 0,
        steady_time: 0,
        frames_count: 2,
        channel_count: 1,
        audio_inputs_count: 0,
        audio_outputs_count: 1,
    }
    .build();

    {
        let mut clap_process = test_process.clap_process();
        let mut process =
            unsafe { Process::new_unchecked(NonNull::new_unchecked(&raw mut clap_process)) };

        let mut out0 = process.audio_outputs(0);
        out0.data32(0)[0] = 0.1;
        out0.data32(0)[1] = 0.2;
    }

    assert_eq!(test_process.audio_outputs[0].data32[0].data()[0], 0.1);
    assert_eq!(test_process.audio_outputs[0].data32[0].data()[1], 0.2);
}

#[test]
fn audio_output_data64() {
    let mut test_process = TestProcessConfig {
        latency: 0,
        steady_time: 0,
        frames_count: 2,
        channel_count: 1,
        audio_inputs_count: 0,
        audio_outputs_count: 1,
    }
    .build();

    {
        let mut clap_process = test_process.clap_process();
        let mut process =
            unsafe { Process::new_unchecked(NonNull::new_unchecked(&raw mut clap_process)) };

        let mut out0 = process.audio_outputs(0);
        out0.data64(0)[0] = 0.1;
        out0.data64(0)[1] = 0.2;
    }

    assert_eq!(test_process.audio_outputs[0].data64[0].data()[0], 0.1);
    assert_eq!(test_process.audio_outputs[0].data64[0].data()[1], 0.2);
}

#[test]
fn audio_input_output_map_data32() {
    const NUM_FRAMES: u32 = 1024;
    let mut test_process = TestProcessConfig {
        latency: 0,
        steady_time: 0,
        frames_count: NUM_FRAMES,
        channel_count: 7,
        audio_inputs_count: 3,
        audio_outputs_count: 2,
    }
    .build();

    let mut buf = vec![];
    for i in 0..NUM_FRAMES {
        test_process.audio_inputs[2].data32[3].0[i as usize] = i as f32;
    }

    {
        let mut clap_process = test_process.clap_process();
        let mut process =
            unsafe { Process::new_unchecked(NonNull::new_unchecked(&raw mut clap_process)) };

        for sample in process.audio_inputs(2).data32(3) {
            buf.push(*sample)
        }
        for (out, sample) in process.audio_outputs(1).data32(6).iter_mut().zip(buf) {
            *out = sample;
        }
    }

    for i in 0..NUM_FRAMES as usize {
        assert_eq!(test_process.audio_inputs[2].data32[3].0[i], i as f32);
        assert_eq!(test_process.audio_outputs[1].data32[6].0[i], i as f32);
    }
}

#[test]
fn audio_input_output_map_data64() {
    const NUM_FRAMES: u32 = 1024;
    let mut test_process = TestProcessConfig {
        latency: 0,
        steady_time: 0,
        frames_count: NUM_FRAMES,
        channel_count: 7,
        audio_inputs_count: 3,
        audio_outputs_count: 2,
    }
    .build();

    let mut buf = vec![];
    for i in 0..NUM_FRAMES {
        test_process.audio_inputs[2].data64[3].0[i as usize] = i as f64;
    }

    {
        let mut clap_process = test_process.clap_process();
        let mut process =
            unsafe { Process::new_unchecked(NonNull::new_unchecked(&raw mut clap_process)) };

        for sample in process.audio_inputs(2).data64(3) {
            buf.push(*sample)
        }
        for (out, sample) in process.audio_outputs(1).data64(6).iter_mut().zip(buf) {
            *out = sample;
        }
    }

    for i in 0..NUM_FRAMES as usize {
        assert_eq!(test_process.audio_inputs[2].data64[3].0[i], i as f64);
        assert_eq!(test_process.audio_outputs[1].data64[6].0[i], i as f64);
    }
}
