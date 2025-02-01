use std::sync::Arc;

use clap_clap::clap::{
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
    const ID: &'static str = "clap.example.ping_pong";
    const NAME: &'static str = "Ping-Pong";
    const VENDOR: &'static str = "⧉⧉⧉";
    type AudioThread = Delay;
    type Extensions = Self;

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
struct Delay {
    buf: Vec<[f32; 2]>,
    cur: usize, // index into the delay queue
}

impl Delay {
    fn new(sample_rate: f64, time: f64) -> Self {
        let samples = (sample_rate * time) as usize;
        Self {
            buf: vec![[0.0; 2]; samples],
            cur: 0,
        }
    }
}

impl AudioThread<PingPong> for Delay {
    fn process(&mut self, process: &mut Process) -> Result<Status, Error> {
        process.frames(|_| Ok(Continue))
    }
}

// Export clap_entry symbols and build a plugin factory.
clap::entry!(PingPong);
