use crate::process::{AudioBuffer, AudioBufferMut, Process};

pub struct Frame<'a> {
    process: &'a mut Process,
    index: u32,
}

impl<'a> Frame<'a> {
    /// # Safety
    ///
    /// 1. `index` must be less that `process.frames_count()`.
    pub(crate) const unsafe fn new_unchecked(process: &'a mut Process, index: u32) -> Self {
        Self { process, index }
    }

    /// # Safety
    ///
    /// 1. n must be less that `process.audio_inputs_count()`.
    const unsafe fn audio_input_unchecked(&self, n: u32) -> FrameInput<'a, '_> {
        // Safety: the caller upholds the safety requirements.
        unsafe { FrameInput::new_unchecked(self, n) }
    }

    /// # Panic
    ///
    /// This function will panic if n is greater or equal to
    /// `process.audio_inputs_count()`
    pub const fn audio_input(&self, n: u32) -> FrameInput<'a, '_> {
        if n < self.process.audio_inputs_count() {
            // Safety: We just checked if n is less that audio_input_count().
            unsafe { Frame::audio_input_unchecked(self, n) }
        } else {
            panic!("the audio input number must be less than the number of available input ports")
        }
    }

    /// # Safety
    ///
    /// 1. n must be less that `process.audio_outputs_count()`.
    const unsafe fn audio_output_unchecked(&mut self, n: u32) -> FrameOutput<'a, '_> {
        // Safety: the caller upholds the safety requirements.
        unsafe { FrameOutput::new_unchecked(self, n) }
    }

    /// # Panic
    ///
    /// This function will panic if n is greater or equal to
    /// `process.audio_outputs_count()`
    pub const fn audio_output(&mut self, n: u32) -> FrameOutput<'a, '_> {
        if n < self.process.audio_outputs_count() {
            // Safety: We just checked if n is less that audio_output_count().
            unsafe { Frame::audio_output_unchecked(self, n) }
        } else {
            panic!("the audio output number must be less than the number of available output ports")
        }
    }
}

pub struct FrameInput<'a: 'b, 'b> {
    frame: &'b Frame<'a>,
    n: u32,
}

impl<'a: 'b, 'b> FrameInput<'a, 'b> {
    /// # Safety
    ///
    /// 1. n must be less than process.audio_inputs_count().
    const unsafe fn new_unchecked(frame: &'b Frame<'a>, n: u32) -> Self {
        Self { frame, n }
    }

    const fn audio_input(&self) -> AudioBuffer {
        // SAFETY: By construction, n is less than process.audio_inputs_count().
        unsafe { self.frame.process.audio_inputs_unchecked(self.n) }
    }

    pub const fn channel_count(&self) -> u32 {
        self.audio_input().channel_count()
    }

    /// # Safety
    ///
    /// 1. n must be less than self.channel_count()
    pub const unsafe fn data32_unchecked(&mut self, channel: u32) -> f32 {
        // SAFETY: The caller guarantees this dereferencing is safe.
        unsafe { self.audio_input().data32_unchecked(channel)[self.frame.index as usize] }
    }

    pub const fn data32(&mut self, channel: u32) -> f32 {
        if channel < self.channel_count() {
            // Safety:
            // We just checked if n < channel_count()
            unsafe { self.data32_unchecked(channel) }
        } else {
            panic!("channel number must be less that the number of available channels")
        }
    }
}

pub struct FrameOutput<'a: 'b, 'b> {
    frame: &'b mut Frame<'a>,
    n: u32,
}

impl<'a: 'b, 'b> FrameOutput<'a, 'b> {
    /// # Safety
    ///
    /// 1. n must be less than process.audio_outputs_count().
    const unsafe fn new_unchecked(frame: &'b mut Frame<'a>, n: u32) -> Self {
        Self { frame, n }
    }

    const fn audio_output(&mut self) -> AudioBufferMut {
        // SAFETY: By construction, n is less than process.audio_outputs_count().
        unsafe { self.frame.process.audio_outputs_unchecked(self.n) }
    }

    pub const fn channel_count(&mut self) -> u32 {
        self.audio_output().channel_count()
    }

    /// # Safety
    ///
    /// 1. `channel` must be less than self.channel_count().
    /// 2. `channel` must fit into usize (cast).
    const unsafe fn data32_unchecked(&mut self, channel: u32) -> &mut f32 {
        let index = self.frame.index;
        // SAFETY: We hold a mutable reference to Frame, and hence a mutable
        // reference to process as well.  Hence, in is guaranteed that we are
        // the only ones accessing the audio buffer.  Hence, we can create
        // safely a mutable reference to one of the samples in the buffer for
        // the lifetime of self.
        let channel = unsafe {
            *self
                .audio_output()
                .clap_audio_buffer
                .as_mut()
                .data32
                .add(channel as usize)
        };
        unsafe { &mut *channel.add(index as usize) }
    }

    pub const fn data32(&mut self, channel: u32) -> &mut f32 {
        if channel < self.channel_count() {
            // SAFETY: We just checked if `channel < self.channel_count()`
            unsafe { self.data32_unchecked(channel) }
        } else {
            panic!("channel number must be less that the number of available channels")
        }
    }
}
