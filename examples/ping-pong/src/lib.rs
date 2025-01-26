use clap::{
    Extensions, Plugin, Process,
    extensions::{AudioPorts, audio_ports::StereoPorts},
    process, process::Status::Continue,
};

#[derive(Default)]
struct PingPong {
    del: Vec<[f32; 2]>, // delay line
    cur: usize,         // cursor (and sample counter)
}

impl Extensions<Self> for PingPong {
    // Provide CLAP "audio ports" extension: for example,
    // a pair of static stereo ports: one in and one out.
    fn audio_ports() -> Option<impl AudioPorts<PingPong>> {
        Some(StereoPorts)
    }
}

impl Plugin for PingPong {
    const ID: &'static str = "clap.example.ping_pong";
    const NAME: &'static str = "Ping-Pong";
    type Extensions = Self;

    /// Allocate resources: a stereo delay line, 200ms long.
    fn activate(&mut self, sample_rate: f64, _: usize, _: usize) -> Result<(), clap::Error> {
        let one_second = sample_rate as usize;
        self.del = vec![[0.0; 2]; one_second / 5];

        Ok(())
    }

    /// Process audio: Feedback delay with ping-pong effect.
    fn process(&mut self, pc: &mut Process) -> Result<process::Status, process::Error> {
        // Link audio ports in[0] and out[0] with a closure that processes
        // one frame (two channels) of samples at a time.
        pc.link_ports(0, 0)?.with_op(|frame: &mut [f32]| {
            let n = self.del.len();
            let (front, back) = (self.cur % n, (n + self.cur - 1) % n);
            let (front_l, front_r) = (self.del[front][0], self.del[front][1]);

            self.del[back][0] = frame[1] + 0.66 * front_r;
            self.del[back][1] = frame[0] + 0.66 * front_l;

            frame[0] += front_l;
            frame[1] += front_r;
            self.cur += 1;
        });

        Ok(Continue)
    }
}

// Export clap_entry symbols and build a plugin factory.
clap::entry!(PingPong);
