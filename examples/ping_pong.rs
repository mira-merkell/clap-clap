use std::sync::Arc;

use clap_clap::{
    Error,
    ext::{AudioPorts, Extensions, audio_ports::StereoPorts},
    host::Host,
    plugin::{AudioThread, Plugin},
    process::{Process, Status, Status::Continue},
};

#[derive(Default)]
struct PingPong;

impl Extensions<Self> for PingPong {
    // Provide CLAP "audio_ports" extension: for example,
    // a static layout of stereo ports, one in and two out.
    // If the plugin needs to dynamically control the port layout,
    // you might want to implement the AudioPorts trait yourself.
    fn audio_ports() -> Option<impl AudioPorts<Self>> {
        Some(StereoPorts::<1, 2>)
    }
}

impl Plugin for PingPong {
    type AudioThread = Delay;
    type Extensions = Self;

    const ID: &'static str = "clap.example.ping_pong";
    const NAME: &'static str = "Ping-Pong";
    const VENDOR: &'static str = "⧉⧉⧉";

    /// Let's say Hi to the host!
    fn init(&mut self, host: Arc<Host>) -> Result<(), Error> {
        host.get_extension().log()?.info("hello, sonic world!")?;
        Ok(())
    }

    /// Start the audio thread.
    fn activate(&mut self, sample_rate: f64, _: u32, _: u32) -> Result<Delay, Error> {
        // Allocate resources: a stereo delay line, 125ms.
        Ok(Delay::new(sample_rate, 0.125))
    }
}

/// The signal processor: Feedback delay with ping-pong effect.
///
/// Instances of this type will live on the audio thread.
struct Delay {
    buf: Vec<[f32; 2]>,
    cur: usize, // index into the delay line
}

impl Delay {
    fn new(sample_rate: f64, time: f64) -> Self {
        // Calculate the number of samples needed to hold
        // the buffer of length `time` seconds.
        let samples = (sample_rate * time) as usize;
        Self {
            buf: vec![[0.0; 2]; samples],
            cur: 0,
        }
    }
}

impl AudioThread<PingPong> for Delay {
    fn process(&mut self, process: &mut Process) -> Result<Status, Error> {
        // Generate a lending iterator over mutable references
        // to consecutiveframes of audio samples and events.
        let mut frames = process.frames();

        let n = self.buf.len();
        // Process the audio block frame by frame.
        while let Some(frame) = frames.next() {
            // Get the position of the current front and back of the delay line.
            let (front, back) = (self.cur % n, (n - 1 + self.cur) % n);
            // Get the audio signal from the front of the delay line.
            let (front_l, front_r) = (self.buf[front][0], self.buf[front][1]);

            // Get the input signal from the main input port.
            let in_l = frame.audio_input(0).data32(0);
            let in_r = frame.audio_input(0).data32(1);

            // Write from the input port into the back of the delay line.
            // Feed the signal back with 0.66 damping, swap left/right channels.
            self.buf[back][0] = in_r + 0.66 * front_r;
            self.buf[back][1] = in_l + 0.66 * front_l;

            // Write into the main output port from the front of the delay line.
            *frame.audio_output(0).data32(0) = front_l;
            *frame.audio_output(0).data32(1) = front_r;

            // Pass the dry signal to the second output port.
            *frame.audio_output(1).data32(0) = in_l;
            *frame.audio_output(1).data32(1) = in_r;

            // On a 64-bit machine, prepare for overflow in about 12 million years.
            self.cur += 1;
        }

        Ok(Continue)
    }
}

// Export clap_entry symbols and build a plugin factory.
clap_clap::entry!(PingPong);
