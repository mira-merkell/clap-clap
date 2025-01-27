# clap-clap

[![CI](https://github.com/mira-merkell/clap-clap/actions/workflows/CI.yml/badge.svg)](https://github.com/mira-merkell/clap-clap/actions/workflows/CI.yml)

Another [CLAP] framework. Very much WIP. 🚧

## Goals

* Provide a safe-Rust interface to the [CLAP API].
* Follow CLAP framework and terminology of extension modules.
* Let plugins interact dynamically CLAP hosts.
* Build extensive testing and debugging platform for plugin authors.

[CLAP]: https://cleveraudio.org

[CLAP API]: https://github.com/free-audio/clap/tree/main/include/clap

## Example

Implement a [ping-pong delay]:

```rust
use clap::{
    ext::{audio_ports::StereoPorts, AudioPorts, Extensions},
    plugin::{self, Plugin},
    process::{self, Process, Status::Continue},
};

#[derive(Default)]
struct PingPong {
    // Delay line,  defaults to None.  We will allocate the space
    // for a 125ms buffer, once the host tells us what the sample rate is.
    delay: Option<Vec<[f32; 2]>>,
    cursor: usize, // index into the delay queue
}

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
    type Extensions = Self;

    /// Allocate resources: a stereo delay line, 125ms.
    fn activate(&mut self, sample_rate: f64, _: usize, _: usize) -> Result<(), plugin::Error> {
        let one_second = sample_rate as usize;
        self.delay = Some(vec![[0.0; 2]; one_second / 8]);
        Ok(())
    }

    /// Process audio: Feedback delay with ping-pong effect.
    fn process(&mut self, pc: &mut Process) -> Result<process::Status, process::Error> {
        let delay = self.delay.as_mut().ok_or(process::Error::Plugin)?;
        let n = delay.len();

        // Link audio ports: in:0 and out:0 with a closure that processes
        // one frame (two channels) of samples at a time.
        pc.link_audio_ports(0, 0)?.with_op(|frame: &mut [f32]| {
            let (front, back) = (self.cursor % n, (n - 1 + self.cursor) % n);
            let (front_l, front_r) = (delay[front][0], delay[front][1]);

            // Write from the in port into the back of the delay line.
            // Swap stereo channels, feed the signal back with 0.66 damping.
            delay[back][0] = frame[1] + 0.66 * front_r;
            delay[back][1] = frame[0] + 0.66 * front_l;

            // Write into the out port from the front of the delay line.
            frame[0] = front_l;
            frame[1] = front_r;

            self.cursor += 1; // Prepare for overflow in about 12 million years.
        });

        // Pass the dry signal to the second output port.
        pc.link_audio_ports(0, 1)?.with_op(|_| ());

        Ok(Continue)
    }
}

// Export clap_entry symbols and build a plugin factory.
clap::entry!(PingPong);
```

[ping-pong delay]: ./examples/ping-pong/

## Compile the source code

Install *nightly* Rust (for the 2024 edition) and clone this repository together
with its submodules:

```bash
git clone --recurse-submodules https://github.com/mira-merkell/clap-clap
```

Build the plugin with:

```bash
cargo build -r
```

and look for the compiled dynamical library in `target/release/`.

The name of the library is OS-specific. For example, on Linux it should be:
`libping_pong.so`, whereas on Windows it's `ping_pong.dll`.Copy the file to
where your DAW can find it and rename it to: `ping_pong.clap`.
