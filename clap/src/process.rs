use clap_sys::{
    CLAP_PROCESS_CONTINUE, CLAP_PROCESS_CONTINUE_IF_NOT_QUIET, CLAP_PROCESS_SLEEP,
    CLAP_PROCESS_TAIL, clap_audio_buffer, clap_process, clap_process_status,
};
use std::ptr::{slice_from_raw_parts, slice_from_raw_parts_mut};

pub struct Process(pub(crate) clap_process);

impl Process {
    pub fn steady_time(&self) -> i64 {
        self.0.steady_time
    }

    pub fn frames_count(&self) -> usize {
        self.0
            .frames_count
            .try_into()
            .expect("frame_count must fit into usize")
    }

    pub fn audio_inputs_count(&self) -> usize {
        self.0
            .audio_inputs_count
            .try_into()
            .expect("audio_inputs_count must fit into usize")
    }

    pub unsafe fn audio_input_unchecked(&self, n: usize) -> Input<'_> {
        Input {
            buf: unsafe { &*self.0.audio_inputs.add(n) },
            frames_count: self.frames_count(),
        }
    }

    pub fn audio_input(&self, n: usize) -> Option<Input<'_>> {
        (n < self.0.audio_inputs_count as usize).then_some(unsafe { self.audio_input_unchecked(n) })
    }

    pub fn audio_inputs_iter(&self) -> impl Iterator<Item = Input<'_>> {
        (0..self.audio_inputs_count()).map(|n| unsafe { self.audio_input_unchecked(n) })
    }

    pub fn audio_outputs_count(&self) -> usize {
        self.0
            .audio_outputs_count
            .try_into()
            .expect("audio_outputs_count must fit into usize")
    }

    pub unsafe fn audio_output_unchecked(&mut self, n: usize) -> Output<'_> {
        Output {
            buf: unsafe { &mut *self.0.audio_outputs.add(n) },
            frames_count: self.frames_count(),
        }
    }

    pub fn audio_output(&mut self, n: usize) -> Option<Output<'_>> {
        (n < self.0.audio_outputs_count as usize)
            .then_some(unsafe { self.audio_output_unchecked(n) })
    }

    pub unsafe fn link_ports_unchecked(&mut self, port_in: usize, port_out: usize) -> Link<'_> {
        let channel_count = unsafe { self.audio_input_unchecked(port_in).channel_count() };
        let port_in = unsafe { &*self.0.audio_inputs.add(port_in) };
        let port_out = unsafe { &mut *self.0.audio_outputs.add(port_out) };

        unsafe { Link::new_unchecked(port_in, port_out, channel_count, self.frames_count()) }
    }

    pub fn link_audio_ports(&mut self, port_in: usize, port_out: usize) -> Result<Link<'_>, Error> {
        let port_in = ((port_in as u32) < self.0.audio_inputs_count)
            .then_some(unsafe { &*self.0.audio_inputs.add(port_in) })
            .ok_or(Error::Link)?;
        let port_out = ((port_out as u32) < self.0.audio_outputs_count)
            .then_some(unsafe { &mut *self.0.audio_outputs.add(port_out) })
            .ok_or(Error::Link)?;

        Link::new(port_in, port_out, self.frames_count()).ok_or(Error::Link)
    }
}

pub struct Input<'a> {
    buf: &'a clap_audio_buffer,
    frames_count: usize,
}

impl Input<'_> {
    pub fn channel_count(&self) -> usize {
        self.buf
            .channel_count
            .try_into()
            .expect("channel count must fit into usize")
    }

    pub fn latency(&self) -> usize {
        self.buf
            .latency
            .try_into()
            .expect("latency must fit into usize")
    }

    pub fn constant_mask(&self) -> u64 {
        self.buf.constant_mask
    }

    pub unsafe fn channel_unchecked(&self, n: usize) -> &[f32] {
        let samples = unsafe { *self.buf.data32.add(n) };
        unsafe { &*slice_from_raw_parts(samples, self.frames_count) }
    }

    pub fn channel(&self, n: usize) -> Option<&[f32]> {
        (n < self.buf.channel_count as usize).then_some(unsafe { self.channel_unchecked(n) })
    }

    pub fn channel_iter(&self) -> impl Iterator<Item = &[f32]> {
        (0..self.channel_count()).map(|n| unsafe { self.channel_unchecked(n) })
    }
}

pub struct Output<'a> {
    buf: &'a mut clap_audio_buffer,
    frames_count: usize,
}

impl Output<'_> {
    pub fn channel_count(&self) -> usize {
        self.buf.channel_count as _
    }

    pub fn latency(&self) -> usize {
        self.buf.latency as _
    }

    pub fn constant_mask(&self) -> u64 {
        self.buf.constant_mask
    }

    pub unsafe fn channel_unchecked(&mut self, n: usize) -> &mut [f32] {
        let samples = unsafe { *self.buf.data32.add(n) };
        unsafe { &mut *slice_from_raw_parts_mut(samples, self.frames_count) }
    }

    pub fn channel_mut(&mut self, n: usize) -> Option<&mut [f32]> {
        (n < self.buf.channel_count as usize).then_some(unsafe { self.channel_unchecked(n) })
    }
}

/// Up to 8 channel ports.
pub struct Link<'a> {
    port_in: &'a clap_audio_buffer,
    port_out: &'a mut clap_audio_buffer,
    channel_count: usize,
    frames_count: usize,
    frame: [f32; 8],
}

impl<'a> Link<'a> {
    unsafe fn new_unchecked(
        port_in: &'a clap_audio_buffer,
        port_out: &'a mut clap_audio_buffer,
        channel_count: usize,
        frames_count: usize,
    ) -> Self {
        Self {
            port_in,
            port_out,
            channel_count,
            frames_count,
            frame: [0.0; 8],
        }
    }

    fn new(
        port_in: &'a clap_audio_buffer,
        port_out: &'a mut clap_audio_buffer,
        frames_count: usize,
    ) -> Option<Self> {
        let channel_count = usize::try_from(port_in.channel_count).ok()?;
        (channel_count <= 8 && port_in.channel_count == port_out.channel_count).then_some(unsafe {
            Self::new_unchecked(port_in, port_out, channel_count, frames_count)
        })
    }

    pub fn with_op(&mut self, op: impl FnMut(&mut [f32])) {
        let mut op = op;

        for i in 0..self.frames_count {
            for k in 0..self.channel_count {
                let sample = unsafe { *(*self.port_in.data32.add(k)).add(i) };
                self.frame[k] = sample;
            }

            op(&mut self.frame[0..self.channel_count]);

            for k in 0..self.channel_count {
                let sample = unsafe { &mut *(*self.port_out.data32.add(k)).add(i) };
                *sample = self.frame[k];
            }
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Status {
    Continue,
    ContinueIfNotQuiet,
    Tail,
    Sleep,
}

impl From<Status> for clap_process_status {
    fn from(value: Status) -> Self {
        use Status::*;
        match value {
            Continue => CLAP_PROCESS_CONTINUE,
            ContinueIfNotQuiet => CLAP_PROCESS_CONTINUE_IF_NOT_QUIET,
            Tail => CLAP_PROCESS_TAIL,
            Sleep => CLAP_PROCESS_SLEEP,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Error {
    Init,
    Link,
    Plugin,
}
