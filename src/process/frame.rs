use crate::{
    events::{Header, InputEvents},
    process::{AudioBuffer, AudioBufferMut, Process},
};

pub struct Frame<'a> {
    process: &'a mut Process,
    index: u32,
    in_events: Option<(u32, u32)>,
}

impl<'a> Frame<'a> {
    /// # Safety
    ///
    /// 1. `index` must be less that `process.frames_count()`.
    pub(crate) const unsafe fn new_unchecked(
        process: &'a mut Process,
        index: u32,
        in_events: Option<(u32, u32)>,
    ) -> Self {
        Self {
            process,
            index,
            in_events,
        }
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

    pub const fn in_events(&self) -> Option<FrameInputEvents> {
        if let Some((start, end)) = self.in_events {
            Some(FrameInputEvents::new(self, start, end))
        } else {
            None
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
    /// `channel` must be less than `self.channel_count()`
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

    /// # Safety
    ///
    /// `channel` must be less than `self.channel_count()`
    pub const unsafe fn data64_unchecked(&mut self, channel: u32) -> f64 {
        // SAFETY: The caller guarantees this dereferencing is safe.
        unsafe { self.audio_input().data64_unchecked(channel)[self.frame.index as usize] }
    }

    pub const fn data64(&mut self, channel: u32) -> f64 {
        if channel < self.channel_count() {
            // Safety:
            // We just checked if n < channel_count()
            unsafe { self.data64_unchecked(channel) }
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

    /// # Safety
    ///
    /// 1. `channel` must be less than self.channel_count().
    /// 2. `channel` must fit into usize (cast).
    const unsafe fn data64_unchecked(&mut self, channel: u32) -> &mut f64 {
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
                .data64
                .add(channel as usize)
        };
        unsafe { &mut *channel.add(index as usize) }
    }

    pub const fn data64(&mut self, channel: u32) -> &mut f64 {
        if channel < self.channel_count() {
            // SAFETY: We just checked if `channel < self.channel_count()`
            unsafe { self.data64_unchecked(channel) }
        } else {
            panic!("channel number must be less that the number of available channels")
        }
    }
}

pub struct FrameInputEvents<'a: 'b, 'b> {
    frame: &'b Frame<'a>,
    in_events: InputEvents<'b>,
    index: u32,
    end: u32,
}

impl<'a: 'b, 'b> FrameInputEvents<'a, 'b> {
    const fn new(frame: &'b Frame<'a>, start: u32, end: u32) -> Self {
        Self {
            frame,
            in_events: frame.process.in_events(),
            index: start,
            end,
        }
    }

    fn next(&mut self) -> Option<&Header> {
        if self.index >= self.end {
            return None;
        }
        let header = self.in_events.get(self.index);
        if header.time() > self.frame.index {
            return None;
        }
        self.index += 1;
        Some(header)
    }
}

/// Lending iterator over frames from Process.
pub struct FramesMut<'a> {
    frame: Option<Frame<'a>>,
    index: u32,
    in_events: Option<(u32, u32)>,
}

impl<'a> FramesMut<'a> {
    pub const fn new(process: &'a mut Process) -> Self {
        let in_events = if process.ev_size > 0 {
            Some((0, process.ev_size))
        } else {
            None
        };

        let frame = if process.frames_count() > 0 {
            // SAFETY: we just checked if number of frames in the process
            // is greater than zero.
            Some(unsafe { Frame::new_unchecked(process, 0, None) })
        } else {
            None
        };

        Self {
            frame,
            index: 0,
            in_events,
        }
    }

    /// Generate mutable references to consecutive frames in the `process`.
    ///
    /// Unlike `next()` from the `Iterator` trait, this function is generic
    /// over the lifetime of `&mut self`.  In other words, without lifetime
    /// elision, the signature of this function reads:
    ///
    /// ```text
    ///  pub const fn next<'b>(&'b mut self) -> Option<&'b mut Frame<'a>>;
    /// ```
    ///
    /// and each returned reference is valid only until the subsequent call to
    /// `next()`.
    ///
    /// Note also that unlike `Iterator::next()`, this function is `const`.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use clap_clap::process::frame::FramesMut;
    /// # use clap_clap::process::Process;
    /// fn process_frames(process: &mut Process) {
    ///     let mut frames = FramesMut::new(process);
    ///     while let Some(frame) = frames.next() {
    ///         // Process frame here.
    ///     }
    /// }
    /// ```
    ///
    /// Be aware that if you don't bind the iterator to `frames` in the
    /// example above, and instead you write:
    ///
    /// ```rust,no_run
    /// # use clap_clap::process::frame::FramesMut;
    /// # use clap_clap::process::Process;
    /// fn process_frames_endlessly(process: &mut Process) {
    ///     while let Some(frame) = FramesMut::new(process).next() { // <-- Danger: infinite loop.
    ///         // over and over again...
    ///     }
    /// }
    /// ```
    ///
    /// you will most probably end up with an infinite loop, as the iterator is
    /// created anew each time we enter the `while` block.
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Option<&mut Frame<'a>> {
        let ev_list = find_frame_events(self);

        if let Some(frame) = self.frame.take() {
            let process = frame.process;
            if self.index < process.frames_count() {
                self.frame = Some(unsafe { Frame::new_unchecked(process, self.index, ev_list) });
                self.index += 1;
            }
        }

        self.frame.as_mut()
    }
}

fn find_frame_events(iter: &mut FramesMut) -> Option<(u32, u32)> {
    let frame = iter.frame.as_mut()?;
    let in_events = iter.in_events.as_mut()?;
    if in_events.0 >= in_events.1 {
        return None;
    }
    let list = frame.process.in_events();
    let start = in_events.0;
    let start_time = list.get(start).time();
    if start_time > frame.index {
        return None;
    }

    let mut end = start;
    let mut end_time = start_time;
    while end_time <= frame.index {
        end += 1;
        if end >= in_events.1 {
            break;
        }
        end_time = list.get(end).time();
    }
    in_events.0 = end;

    Some((start, end))
}
