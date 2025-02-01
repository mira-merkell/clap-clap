use std::{
    fmt::{Display, Formatter},
    ptr::{NonNull, slice_from_raw_parts, slice_from_raw_parts_mut},
};

use clap_sys::{
    CLAP_PROCESS_CONTINUE, CLAP_PROCESS_CONTINUE_IF_NOT_QUIET, CLAP_PROCESS_SLEEP,
    CLAP_PROCESS_TAIL, clap_audio_buffer, clap_process, clap_process_status,
};

use crate::process::Status::Continue;

pub struct Process(NonNull<clap_process>);

impl Process {
    /// # Safety
    ///
    /// The pointer to clap_process must be obtained from CLAP host
    /// calling clap_plugin.process().
    /// The Process struct's lifetime must not exceed the duration of
    /// the call to clap_plugin.process(), as the pointer represent valid
    /// data only within that function scope.
    ///
    /// There must be only one Process that wraps around the given pointer.
    pub(crate) const unsafe fn new(clap_process: NonNull<clap_process>) -> Self {
        Self(clap_process)
    }

    const fn as_clap_process(&self) -> &clap_process {
        // Safety:
        // By construction, we can obtain an exclusive reference to the underlying
        // pointer.
        unsafe { self.0.as_ref() }
    }

    const fn as_clap_process_mut(&mut self) -> &mut clap_process {
        // Safety:
        // By construction, we can obtain an exclusive reference to the underlying
        // pointer.
        unsafe { self.0.as_mut() }
    }

    pub const fn steady_time(&self) -> i64 {
        self.as_clap_process().steady_time
    }

    pub const fn frames_count(&self) -> u32 {
        self.as_clap_process().frames_count
    }

    pub fn frames(
        &mut self,
        op: impl FnMut(&mut Frame<'_>) -> Result<Status, Error>,
    ) -> Result<Status, crate::Error> {
        let mut res = Ok(Continue);
        let mut op = op;
        for i in 0..self.frames_count() {
            let mut frame = unsafe { Frame::new_unchecked(self, i) };
            res = op(&mut frame);
            if res.is_err() {
                break;
            }
        }
        res.map_err(Into::into)
    }

    pub fn transport(&self) {
        todo!()
    }

    pub const fn audio_inputs_count(&self) -> u32 {
        self.as_clap_process().audio_inputs_count
    }

    /// # Safety
    ///
    /// n must be less that self.audio_inputs_count()
    /// n must fit into usize (cast).
    pub const unsafe fn audio_inputs_unchecked(&self, n: u32) -> AudioBuffer<'_> {
        // Safety:
        // n is less that self.audio_inputs_count(), so clap_audio_buffer is
        // a valid pointer that belongs to Process.
        let clap_audio_buffer = unsafe { self.as_clap_process().audio_inputs.add(n as usize) };
        unsafe { AudioBuffer::new(self, clap_audio_buffer) }
    }

    pub fn audio_inputs(&self, n: u32) -> Option<AudioBuffer<'_>> {
        // Safety: We just checked if n is less then the limit.
        (n < self.audio_inputs_count()).then_some(unsafe { self.audio_inputs_unchecked(n) })
    }

    pub const fn audio_outputs_count(&self) -> u32 {
        // Safety: The requirements satisfied by the constructor guarantee that
        // dereferencing the pointer is safe.
        unsafe { *self.0.as_ptr() }.audio_outputs_count
    }

    /// # Safety
    ///
    /// n must be less that self.audio_outputs_count()
    /// n must fit into usize (cast)
    pub const unsafe fn audio_outputs_unchecked(&mut self, n: u32) -> AudioBufferMut<'_> {
        // Safety:
        // n is less that self.audio_inputs_count(), so clap_audio_buffer is
        // a valid pointer that belongs to Process.
        let clap_audio_buffer = unsafe { self.as_clap_process_mut().audio_outputs.add(n as usize) };
        let clap_audio_buffer = unsafe { NonNull::new_unchecked(clap_audio_buffer) };
        unsafe { AudioBufferMut::new(self, clap_audio_buffer) }
    }

    pub fn audio_outputs(&mut self, n: u32) -> Option<AudioBufferMut<'_>> {
        // Safety: We just checked if n is less then the limit.
        (n < self.audio_outputs_count()).then_some(unsafe { self.audio_outputs_unchecked(n) })
    }

    pub fn in_events(&self) {
        todo!()
    }

    pub fn out_events(&mut self) {
        todo!()
    }
}

pub struct AudioBuffer<'a> {
    process: &'a Process,
    clap_audio_buffer: *const clap_audio_buffer,
}

impl<'a> AudioBuffer<'a> {
    /// # Safety
    ///
    /// clap_audio_buffer must belong to Process
    const unsafe fn new(process: &'a Process, clap_audio_buffer: *const clap_audio_buffer) -> Self {
        Self {
            process,
            clap_audio_buffer,
        }
    }

    const fn as_clap_audio_buffer(&self) -> &clap_audio_buffer {
        // Safety:
        // By construction, audio_buffer can be only obtained from a shared
        // reference to Process.
        unsafe { self.clap_audio_buffer.as_ref().unwrap() }
    }

    /// # Safety
    ///
    /// n must be less than self.channel_count(),
    /// process.frames_count() must fit into u32 (cast without checking).
    pub const unsafe fn data32_unchecked(&self, n: u32) -> &[f32] {
        // Safety:
        // The caller guarantees this dereferencing is safe.
        let chan = unsafe { *self.as_clap_audio_buffer().data32.add(n as usize) };

        // Safety:
        // The CLAP host guarantees that the channel is at
        // least process.frames_count() long.
        unsafe { &*slice_from_raw_parts(chan, self.process.frames_count() as usize) }
    }

    pub fn data32(&self, n: u32) -> Option<&[f32]> {
        // Safety:
        // We just checked if n < channel_count()
        (n < self.channel_count()).then_some(unsafe { self.data32_unchecked(n) })
    }

    /// # Safety
    ///
    /// n must be less than self.channel_count()
    /// process.frames_count() must fit into u32 (cast without checking)
    pub const unsafe fn data64_unchecked(&self, n: u32) -> &[f64] {
        // Safety:
        // The caller guarantees this dereferencing is safe.
        let chan = unsafe { *self.as_clap_audio_buffer().data64.add(n as usize) };

        // Safety:
        // The CLAP host guarantees that the channel is at
        // least process.frames_count() long.
        unsafe { &*slice_from_raw_parts(chan, self.process.frames_count() as usize) }
    }

    pub fn data64(&self, n: u32) -> Option<&[f64]> {
        // Safety:
        // We just checked if n < channel_count()
        (n < self.channel_count()).then_some(unsafe { self.data64_unchecked(n) })
    }

    pub const fn channel_count(&self) -> u32 {
        // Safety:
        // The pointer is valid by construction.
        self.as_clap_audio_buffer().channel_count
    }

    pub const fn latency(&self) -> u32 {
        // Safety:
        // The pointer is valid by construction.
        self.as_clap_audio_buffer().latency
    }

    pub fn constant_mask(&self) -> u64 {
        // Safety:
        // The pointer is valid by construction.
        self.as_clap_audio_buffer().constant_mask
    }
}

pub struct AudioBufferMut<'a> {
    process: &'a mut Process,
    clap_audio_buffer: NonNull<clap_audio_buffer>,
}

impl<'a> AudioBufferMut<'a> {
    /// # Safety
    ///
    /// clap_audio_buffer must be a writable buffer that belongs to Process.
    const unsafe fn new(
        process: &'a mut Process,
        clap_audio_buffer: NonNull<clap_audio_buffer>,
    ) -> Self {
        Self {
            process,
            clap_audio_buffer,
        }
    }

    const fn as_clap_audio_buffer(&self) -> &clap_audio_buffer {
        // Safety:
        unsafe { self.clap_audio_buffer.as_ref() }
    }

    const fn as_clap_audio_buffer_mut(&mut self) -> &mut clap_audio_buffer {
        // Safety:
        unsafe { self.clap_audio_buffer.as_mut() }
    }

    /// # Safety
    ///
    /// n must be less than self.channel_count()
    /// process.frames_count() must fit into u32 (cast without checking)
    pub unsafe fn data32_unchecked(&mut self, n: u32) -> &mut [f32] {
        // Safety:
        // The caller guarantees this dereferencing is safe.
        let chan = unsafe { *self.as_clap_audio_buffer_mut().data32.add(n as usize) };

        // Safety:
        // The CLAP host guarantees that the channel is at
        // least process.frames_count() long.
        unsafe { &mut *slice_from_raw_parts_mut(chan, self.process.frames_count() as usize) }
    }

    pub fn data32(&mut self, n: u32) -> Option<&mut [f32]> {
        // Safety:
        // We just checked if n < channel_count()
        (n < self.channel_count()).then_some(unsafe { self.data32_unchecked(n) })
    }

    /// # Safety
    /// n must be less than self.channel_count()
    /// process.frames_count() must fit into u32 (cast without checking)
    pub unsafe fn data64_unchecked(&mut self, n: u32) -> &mut [f64] {
        // Safety:
        // The caller guarantees this dereferencing is safe.
        let chan = unsafe { *self.as_clap_audio_buffer_mut().data64.add(n as usize) };

        // Safety:
        // The CLAP host guarantees that the channel is at
        // least process.frames_count() long.
        unsafe { &mut *slice_from_raw_parts_mut(chan, self.process.frames_count() as usize) }
    }

    pub fn data64(&mut self, n: u32) -> Option<&mut [f64]> {
        // Safety:
        // We just checked if n < channel_count()
        (n < self.channel_count()).then_some(unsafe { self.data64_unchecked(n) })
    }

    pub const fn channel_count(&self) -> u32 {
        self.as_clap_audio_buffer().channel_count
    }

    pub const fn latency(&self) -> u32 {
        self.as_clap_audio_buffer().latency
    }

    pub fn constant_mask(&self) -> u64 {
        self.as_clap_audio_buffer().constant_mask
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

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Error {}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "process")
    }
}

impl std::error::Error for Error {}

impl From<Error> for crate::Error {
    fn from(value: Error) -> Self {
        Self::Process(value)
    }
}

pub struct Frame<'a> {
    process: &'a mut Process,
    index: u32,
    //in_events: (usize, usize), // First and last in-event for this frame in the process's event
    // list.
}

impl<'a> Frame<'a> {
    /// # Safety
    ///
    /// index must be less that process.frames_count().
    const unsafe fn new_unchecked(process: &'a mut Process, index: u32) -> Self {
        Self { process, index }
    }

    /// # Safety
    ///
    /// n must be less that process.audio_inputs_count().
    pub unsafe fn audio_input_unchecked(&self, n: u32) -> AudioFrame<'a, '_> {
        // Safety: the caller upholds the safety requirements.
        unsafe { AudioFrame::new_unchecked(self, n) }
    }

    pub fn audio_input(&self, n: u32) -> Option<AudioFrame<'a, '_>> {
        (n < self.process.audio_inputs_count())
            // Safety: We just checked if n is less that audio_input_count().
            .then_some(unsafe { AudioFrame::new_unchecked(self, n) })
    }
}

pub struct AudioFrame<'a: 'b, 'b> {
    frame: &'b Frame<'a>,
    n: u32,
}

impl<'a: 'b, 'b> AudioFrame<'a, 'b> {
    /// # Safety
    ///
    /// n must be less than process.audio_inputs_count().
    const unsafe fn new_unchecked(frame: &'b Frame<'a>, n: u32) -> Self {
        Self { frame, n }
    }

    const fn audio_input(&self) -> AudioBuffer {
        // Safety:
        // By construction, n is less than process.audio_inputs_count().
        unsafe { self.frame.process.audio_inputs_unchecked(self.n) }
    }

    pub const fn channel_count(&self) -> u32 {
        self.audio_input().channel_count()
    }

    /// # Safety
    ///
    /// n must be less than self.channel_count()
    pub const unsafe fn data32_unchecked(&mut self, channel: u32) -> f32 {
        // Safety:
        // The caller guarantees this dereferencing is safe.
        unsafe { self.audio_input().data32_unchecked(channel)[self.frame.index as usize] }
    }

    pub fn data32(&mut self, channel: u32) -> Option<f32> {
        // Safety:
        // We just checked if n < channel_count()
        (channel < self.channel_count()).then_some(unsafe { self.data32_unchecked(channel) })
    }

    /// # Safety
    ///
    /// n must be less than self.channel_count()
    pub const unsafe fn data64_unchecked(&mut self, channel: u32) -> f64 {
        // Safety:
        // The caller guarantees this dereferencing is safe.
        unsafe { self.audio_input().data64_unchecked(channel)[self.frame.index as usize] }
    }

    pub fn data64(&mut self, channel: u32) -> Option<f64> {
        // Safety:
        // We just checked if n < channel_count()
        (channel < self.channel_count()).then_some(unsafe { self.data64_unchecked(channel) })
    }
}
