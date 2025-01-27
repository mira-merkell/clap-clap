use clap::{
    ext::{AudioPorts, Extensions, audio_ports::StereoPorts},
    plugin::{self, Plugin},
    process::{self, Process, Status::Continue},
};

#[derive(Default)]
struct PingPong {
    // Defaults to None.  We will allocate the space for a 200ms delay line
    // once the host tells us what the sample rate is.
    delay: Option<Vec<[f32; 2]>>,

    cursor: usize, // index into the delay queue
}

impl Extensions<Self> for PingPong {
    // Provide CLAP "audio_ports" extension: for example,
    // a static layout of stereo ports: one in and two out.
    // If the plugin needs to dynamically control the port layout,
    // you might want to implement the AudioPorts trait yourself.
    fn audio_ports() -> Option<impl AudioPorts<Self>> {
        Some(StereoPorts::<1, 2>)
    }
}

impl Plugin for PingPong {
    const ID: &'static str = "clap.example.ping_pong";
    const NAME: &'static str = "Ping-Pong";
    type Extensions = Self;

    /// Allocate resources: a stereo delay line, 200ms.
    fn activate(&mut self, sample_rate: f64, _: usize, _: usize) -> Result<(), plugin::Error> {
        let one_second = sample_rate as usize;
        self.delay = Some(vec![[0.0; 2]; one_second / 5]);
        Ok(())
    }

    /// Process audio: Feedback delay with ping-pong effect.
    fn process(&mut self, pc: &mut Process) -> Result<process::Status, process::Error> {
        let delay = self.delay.as_mut().ok_or(process::Error::Plugin)?;

        // Link audio ports: in[0] and out[0] with a closure that processes
        // one frame (two channels) of samples at a time.
        pc.link_audio_ports(0, 0)?.with_op(|frame: &mut [f32]| {
            let n = delay.len();
            let (front, back) = (self.cursor % n, (n + self.cursor - 1) % n);
            let (front_l, front_r) = (delay[front][0], delay[front][1]);

            delay[back][0] = frame[1] + 0.66 * front_r;
            delay[back][1] = frame[0] + 0.66 * front_l;

            frame[0] = front_l;
            frame[1] = front_r;
            self.cursor += 1; // Prepare for overflow in about 12 million years.
        });

        // Pass the dry signal to the second output.
        pc.link_audio_ports(0, 1)?.with_op(|_| ());

        Ok(Continue)
    }
}

// Export clap_entry symbols and build a plugin factory.
clap::entry!(PingPong);
