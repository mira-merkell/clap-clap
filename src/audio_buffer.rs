use std::ptr::{NonNull, slice_from_raw_parts, slice_from_raw_parts_mut};

use crate::{ffi::clap_audio_buffer, prelude::Process};

/// Shared audio buffer.
pub struct AudioBuffer<'a> {
    process: &'a Process,
    clap_audio_buffer: *const clap_audio_buffer,
}

impl<'a> AudioBuffer<'a> {
    /// # Safety
    ///
    /// `clap_audio_buffer` must be non-null and must belong to Process.
    pub const unsafe fn new_unchecked(
        process: &'a Process,
        clap_audio_buffer: *const clap_audio_buffer,
    ) -> Self {
        debug_assert!(!clap_audio_buffer.is_null());
        Self {
            process,
            clap_audio_buffer,
        }
    }

    const fn clap_audio_buffer(&self) -> &clap_audio_buffer {
        // SAFETY: By construction, audio_buffer can be only obtained from a shared
        // reference to Process.
        unsafe { self.clap_audio_buffer.as_ref().unwrap() }
    }

    /// # Safety
    ///
    /// 1. `channel` must be less than `self.channel_count()`,
    /// 2. `process.frames_count()` must fit into `usize` (cast).
    pub const unsafe fn data32_unchecked(&self, channel: u32) -> &[f32] {
        debug_assert!(channel < self.channel_count());
        // SAFETY: The caller guarantees this dereferencing is safe.
        let chan = unsafe { *self.clap_audio_buffer().data32.add(channel as usize) };

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
    pub const unsafe fn data64_unchecked(&self, channel: u32) -> &[f64] {
        debug_assert!(channel < self.channel_count());
        // SAFETY: The caller guarantees this dereferencing is safe.
        let chan = unsafe { *self.clap_audio_buffer().data64.add(channel as usize) };

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
        self.clap_audio_buffer().channel_count
    }

    pub const fn latency(&self) -> u32 {
        self.clap_audio_buffer().latency
    }

    pub const fn constant_mask(&self) -> u64 {
        self.clap_audio_buffer().constant_mask
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
    pub const unsafe fn new_unchecked(
        process: &'a mut Process,
        clap_audio_buffer: NonNull<clap_audio_buffer>,
    ) -> Self {
        Self {
            process,
            clap_audio_buffer,
        }
    }

    const fn clap_audio_buffer(&self) -> &clap_audio_buffer {
        // SAFETY: The constructor guarantees that we have exclusive access to the audio
        // buffer. We know that the buffer remains constant for the lifetime of
        // self, so the aliasing is safe.
        unsafe { self.clap_audio_buffer.as_ref() }
    }

    const fn clap_audio_buffer_mut(&mut self) -> &mut clap_audio_buffer {
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
        let chan = unsafe { *self.clap_audio_buffer_mut().data32.add(channel as usize) };

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
        let chan = unsafe { *self.clap_audio_buffer_mut().data64.add(channel as usize) };

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
        self.clap_audio_buffer().channel_count
    }

    pub const fn latency(&self) -> u32 {
        self.clap_audio_buffer().latency
    }

    pub const fn constant_mask(&self) -> u64 {
        self.clap_audio_buffer().constant_mask
    }
}
