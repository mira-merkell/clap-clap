//! CLAP Process interface.
//!
//! The facilities here are mostly const functions to access audio buffers
//! and event lists in a safe way.
use std::{
    fmt::{Display, Formatter},
    ptr::{NonNull, slice_from_raw_parts, slice_from_raw_parts_mut},
};

use crate::{
    events::{Header, InputEvents, OutputEvents, Transport},
    ffi::{
        CLAP_PROCESS_CONTINUE, CLAP_PROCESS_CONTINUE_IF_NOT_QUIET, CLAP_PROCESS_SLEEP,
        CLAP_PROCESS_TAIL, clap_audio_buffer, clap_process, clap_process_status,
    },
    process::frame::FramesMut,
};

pub mod frame;

pub struct Process {
    clap_process: NonNull<clap_process>,
}

impl Process {
    /// # Safety
    ///
    /// 1. The pointer to clap_process must be obtained from CLAP host calling
    ///    `clap_plugin.process()`.
    /// 2. The Process lifetime must not exceed the duration of the call to
    ///    `clap_plugin.process()`, as the pointer represents valid data only
    ///    within that function scope.
    /// 3. There must be only one Process that wraps around the given pointer.
    /// 4. If 'clap_process.audio_input_count > 0', then
    ///    'clap_process.audio_inputs' must be non-null.
    /// 5. If 'clap_process.audio_outputs_count > 0', then
    ///    'clap_process.audio_outputs' must be non-null.
    /// 6. The pointers: `clap_process.in_events` and `clap_process.out_events`
    ///    must be non-null.  These structures must be valid, in the sense that
    ///    the function pointers that are their fields must be non-null (Some).
    #[doc(hidden)]
    pub const unsafe fn new_unchecked(clap_process: NonNull<clap_process>) -> Self {
        #[cfg(debug_assertions)]
        {
            let clap_process = unsafe { clap_process.as_ref() };
            assert!(clap_process.audio_inputs_count == 0 || !clap_process.audio_inputs.is_null());
            assert!(clap_process.audio_outputs_count == 0 || !clap_process.audio_outputs.is_null());

            assert!(!clap_process.in_events.is_null());
            let in_events = unsafe { &*clap_process.in_events };
            assert!(in_events.size.is_some() && in_events.get.is_some());

            assert!(!clap_process.out_events.is_null());
            let out_events = unsafe { &*clap_process.out_events };
            assert!(out_events.try_push.is_some());
        }

        Self { clap_process }
    }

    const fn clap_process(&self) -> &clap_process {
        // SAFETY: By the safety requirements of the constructor, we can obtain a shared
        // reference to the underlying pointer.
        unsafe { self.clap_process.as_ref() }
    }

    const fn clap_process_mut(&mut self) -> &mut clap_process {
        // SAFETY: By the safety requirements of the constructor, we can obtain an
        // exclusive reference to the underlying pointer.
        unsafe { self.clap_process.as_mut() }
    }

    pub const fn steady_time(&self) -> i64 {
        self.clap_process().steady_time
    }

    pub const fn frames_count(&self) -> u32 {
        self.clap_process().frames_count
    }

    pub const fn frames(&mut self) -> FramesMut<'_> {
        FramesMut::new(self)
    }

    /// Transport info at sample 0.
    ///
    /// If None, then this is a free running host and no transport events will
    /// be provided.
    pub const fn transport(&self) -> Option<Transport<'_>> {
        if self.clap_process().transport.is_null() {
            return None;
        }
        // SAFETY: We just checked if transport is non-null. We know that
        // clap_event_transfer is constant and valid for the duration of self,
        // so it's safe to create a shared reference to it for the lifetime of self.
        let header = unsafe { &(*self.clap_process().transport).header };
        // SAFETY: We just crated a reference to clap_event_header from a valid
        // clap_event_transport.
        let header = unsafe { Header::new(header) };
        // SAFETY: We know that header is a header of a clap_event_transport.
        Some(unsafe { Transport::new_unchecked(header) })
    }

    pub const fn audio_inputs_count(&self) -> u32 {
        self.clap_process().audio_inputs_count
    }

    /// # Safety
    ///
    /// 1. The audio input number `n` must be less that
    ///    `self.audio_inputs_count()`
    /// 2. The audio input number `n` must fit into `usize` (cast).
    const unsafe fn audio_inputs_unchecked(&self, n: u32) -> AudioBuffer<'_> {
        debug_assert!(n < self.audio_inputs_count());
        // SAFETY: `n` is less than `self.audio_inputs_count()`, so `clap_audio_buffer`
        // is a valid pointer that belongs to `Process`.
        let clap_audio_buffer = unsafe { self.clap_process().audio_inputs.add(n as usize) };
        unsafe { AudioBuffer::new(self, clap_audio_buffer) }
    }

    /// # Panic
    ///
    /// This function will panic if `n` is greater or equal
    /// to `self.audio_input_counts()`.
    pub const fn audio_inputs(&self, n: u32) -> AudioBuffer<'_> {
        assert!(
            n < self.audio_inputs_count(),
            "audio input number must be less than the number of available input ports"
        );

        // SAFETY: we just checked if n is less then the limit.
        unsafe { self.audio_inputs_unchecked(n) }
    }

    pub const fn audio_outputs_count(&self) -> u32 {
        self.clap_process().audio_outputs_count
    }

    /// # Safety
    ///
    /// 1. The audio output number `n` must be less that
    ///    self.audio_outputs_count()
    /// 2. The audio output number `n` must fit into usize (cast).
    const unsafe fn audio_outputs_unchecked(&mut self, n: u32) -> AudioBufferMut<'_> {
        debug_assert!(n < self.audio_outputs_count());
        // SAFETY: `n` is less that `self.audio_output_count()`, so `clap_audio_buffer`
        // is a valid pointer that belongs to `Process`.
        let clap_audio_buffer = unsafe { self.clap_process_mut().audio_outputs.add(n as usize) };
        let clap_audio_buffer = unsafe { NonNull::new_unchecked(clap_audio_buffer) };
        unsafe { AudioBufferMut::new(self, clap_audio_buffer) }
    }

    /// # Panic
    ///
    /// This function will panic if `n` is larger or equal
    /// `self.audio_output_counts()`.
    pub const fn audio_outputs(&mut self, n: u32) -> AudioBufferMut<'_> {
        assert!(
            n < self.audio_outputs_count(),
            "audio output number must be less than the number of available output ports"
        );

        // SAFETY: we just checked if n is less then the limit.
        unsafe { self.audio_outputs_unchecked(n) }
    }

    pub const fn in_events(&self) -> InputEvents {
        // SAFETY: By construction, the pointer is non-null.
        let in_events = unsafe { &*self.clap_process().in_events };
        // SAFETY: By construction, the pointers are Some.
        unsafe { InputEvents::new_unchecked(in_events) }
    }

    pub fn out_events(&self) -> OutputEvents {
        // SAFETY: By construction, the pointer is non-null.
        let out_events = unsafe { &*self.clap_process().out_events };
        // SAFETY: By construction, the pointer is Some.
        unsafe { OutputEvents::new_unchecked(out_events) }
    }
}

/// Const audio buffer.
pub struct AudioBuffer<'a> {
    process: &'a Process,
    clap_audio_buffer: *const clap_audio_buffer,
}

impl<'a> AudioBuffer<'a> {
    /// # Safety
    ///
    /// `clap_audio_buffer` must be non-null and must belong to Process.
    const unsafe fn new(process: &'a Process, clap_audio_buffer: *const clap_audio_buffer) -> Self {
        debug_assert!(!clap_audio_buffer.is_null());
        Self {
            process,
            clap_audio_buffer,
        }
    }

    const fn as_clap_audio_buffer(&self) -> &clap_audio_buffer {
        // SAFETY: By construction, audio_buffer can be only obtained from a shared
        // reference to Process.
        unsafe { self.clap_audio_buffer.as_ref().unwrap() }
    }

    /// # Safety
    ///
    /// 1. `channel` must be less than `self.channel_count()`,
    /// 2. `process.frames_count()` must fit into `usize` (cast).
    const unsafe fn data32_unchecked(&self, channel: u32) -> &[f32] {
        debug_assert!(channel < self.channel_count());
        // SAFETY: The caller guarantees this dereferencing is safe.
        let chan = unsafe { *self.as_clap_audio_buffer().data32.add(channel as usize) };

        debug_assert!((self.process.frames_count() as u64) < usize::MAX as u64);
        // SAFETY: The CLAP host guarantees that the channel is at
        // least process.frames_count() long.
        unsafe { &*slice_from_raw_parts(chan, self.process.frames_count() as usize) }
    }

    /// # Panic
    ///
    /// This function will panic if `channel` is larger or equal to
    /// `self.channel.count()`.
    pub const fn data32(&self, channel: u32) -> &[f32] {
        assert!(
            channel < self.channel_count(),
            "channel number must be less that the number of available channels"
        );

        // SAFETY: we just checked if `channel < self.channel_count()`
        unsafe { self.data32_unchecked(channel) }
    }

    /// # Safety
    ///
    /// 1. `channel` must be less than `self.channel_count()`,
    /// 2. `process.frames_count()` must fit into `usize` (cast).
    const unsafe fn data64_unchecked(&self, channel: u32) -> &[f64] {
        debug_assert!(channel < self.channel_count());
        // SAFETY: The caller guarantees this dereferencing is safe.
        let chan = unsafe { *self.as_clap_audio_buffer().data64.add(channel as usize) };

        debug_assert!((self.process.frames_count() as u64) < usize::MAX as u64);
        // SAFETY: The CLAP host guarantees that the channel is at
        // least process.frames_count() long.
        unsafe { &*slice_from_raw_parts(chan, self.process.frames_count() as usize) }
    }

    /// # Panic
    ///
    /// This function will panic if `channel` is larger or equal to
    /// `self.channel.count()`.
    pub const fn data64(&self, channel: u32) -> &[f64] {
        assert!(
            channel < self.channel_count(),
            "channel number must be less that the number of available channels"
        );

        // SAFETY: we just checked if `channel < self.channel_count()`
        unsafe { self.data64_unchecked(channel) }
    }

    pub const fn channel_count(&self) -> u32 {
        self.as_clap_audio_buffer().channel_count
    }

    pub const fn latency(&self) -> u32 {
        self.as_clap_audio_buffer().latency
    }

    pub const fn constant_mask(&self) -> u64 {
        self.as_clap_audio_buffer().constant_mask
    }
}

/// Writable audio buffer.
pub struct AudioBufferMut<'a> {
    process: &'a mut Process,
    clap_audio_buffer: NonNull<clap_audio_buffer>,
}

impl<'a> AudioBufferMut<'a> {
    /// # Safety
    ///
    /// `clap_audio_buffer` must be a writable buffer that belongs to Process.
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
        // SAFETY: The constructor guarantees that we have exclusive access to the audio
        // buffer. We know that the buffer remains constant for the lifetime of
        // self, so the aliasing is safe.
        unsafe { self.clap_audio_buffer.as_ref() }
    }

    const fn as_clap_audio_buffer_mut(&mut self) -> &mut clap_audio_buffer {
        // SAFETY: The constructor guarantees that we have exclusive access to the audio
        // buffer.
        unsafe { self.clap_audio_buffer.as_mut() }
    }

    /// # Safety
    ///
    /// 1. The number of channels  must be less than `self.channel_count()`
    /// 2. `process.frames_count()` must fit into `usize` (cast)
    const unsafe fn data32_unchecked(&mut self, channel: u32) -> &mut [f32] {
        debug_assert!(channel < self.channel_count());
        // SAFETY: The caller guarantees this dereferencing is safe.
        let chan = unsafe { *self.as_clap_audio_buffer_mut().data32.add(channel as usize) };

        debug_assert!((self.process.frames_count() as u64) < usize::MAX as u64);
        // SAFETY: The CLAP host guarantees that the channel is at
        // least process.frames_count() long.
        unsafe { &mut *slice_from_raw_parts_mut(chan, self.process.frames_count() as usize) }
    }

    /// # Panic
    ///
    /// This function will panic if `channel` is greater or equal to
    /// `self.channel.count()`.
    pub const fn data32(&mut self, channel: u32) -> &mut [f32] {
        assert!(
            channel < self.channel_count(),
            "channel number must be less that the number of available channels"
        );

        // SAFETY: We just checked if `n < channel_count()`
        unsafe { self.data32_unchecked(channel) }
    }

    /// # Safety
    ///
    /// 1. The number of channels  must be less than `self.channel_count()`
    /// 2. `process.frames_count()` must fit into `usize` (cast)
    const unsafe fn data64_unchecked(&mut self, channel: u32) -> &mut [f64] {
        debug_assert!(channel < self.channel_count());
        // SAFETY: The caller guarantees this dereferencing is safe.
        let chan = unsafe { *self.as_clap_audio_buffer_mut().data64.add(channel as usize) };

        debug_assert!((self.process.frames_count() as u64) < usize::MAX as u64);
        // SAFETY: The CLAP host guarantees that the channel is at
        // least process.frames_count() long.
        unsafe { &mut *slice_from_raw_parts_mut(chan, self.process.frames_count() as usize) }
    }

    /// # Panic
    ///
    /// This function will panic if `channel` is greater or equal to
    /// `self.channel.count()`.
    pub const fn data64(&mut self, channel: u32) -> &mut [f64] {
        assert!(
            channel < self.channel_count(),
            "channel number must be less that the number of available channels"
        );

        // SAFETY: We just checked if `n < channel_count()`
        unsafe { self.data64_unchecked(channel) }
    }

    pub const fn channel_count(&self) -> u32 {
        self.as_clap_audio_buffer().channel_count
    }

    pub const fn latency(&self) -> u32 {
        self.as_clap_audio_buffer().latency
    }

    pub const fn constant_mask(&self) -> u64 {
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
