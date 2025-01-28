# clap-clap

[![CI](https://github.com/mira-merkell/clap-clap/actions/workflows/CI.yml/badge.svg)](https://github.com/mira-merkell/clap-clap/actions/workflows/CI.yml)

Another [CLAP] framework. Very much WIP. 🚧

## Goals

* Provide a safe-Rust interface to the [CLAP API].
* Follow CLAP framework and terminology of extension modules.
* Let plugins interact dynamically with CLAP hosts.
* Build extensive testing and debugging platform for plugin authors.

[CLAP]: https://cleveraudio.org

[CLAP API]: https://github.com/free-audio/clap/tree/main/include/clap

## Example

Implement a [ping-pong delay]:

```rust
use clap::{
    ext::{audio_ports::StereoPorts, AudioPorts, Extensions},
    host::Host,
    plugin::{AudioThread, Plugin},
    process::{Process, Status, Status::Continue},
    Error,
};
use std::sync::Arc;

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
    type AudioThread = Delay;
    type Extensions = Self;

    /// Let's say Hi to the host!
    fn init(&mut self, host: Arc<Host>) -> Result<(), Error> {
        host.get_extension().log()?.info("hello, sonic world!")?;
        Ok(())
    }

    /// Start the audio thread.
    fn activate(&mut self, sample_rate: f64, _: usize, _: usize) -> Result<Delay, Error> {
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
        let n = self.buf.len();
        // Link audio ports: in:0 and out:0 with a closure that processes
        // one frame (two channels) of samples at a time.
        process
            .link_audio_ports(0, 0)?
            .with_op(|frame: &mut [f32]| {
                let (front, back) = (self.cur % n, (n - 1 + self.cur) % n);
                let (front_l, front_r) = (self.buf[front][0], self.buf[front][1]);

                // Write from the in port into the back of the delay line.
                // Feed the signal back with 0.66 damping, swap left/right channels.
                self.buf[back][0] = frame[1] + 0.66 * front_r;
                self.buf[back][1] = frame[0] + 0.66 * front_l;

                // Write into the out port from the front of the delay line.
                frame[0] = front_l;
                frame[1] = front_r;

                self.cur += 1; // Prepare for overflow in about 12 million years.
            });

        // Pass the dry signal to the second output port.
        process.link_audio_ports(0, 1)?.with_op(|_| ());

        Ok(Continue)
    }
}

// Export clap_entry symbols and build a plugin factory.
clap::entry!(PingPong);
```

[ping-pong delay]: https://en.wikipedia.org/wiki/Delay_(audio_effect)#Ping-pong_delay

## Compile the source code

Install Rust >=1.85.0 (for the 2024 edition, available on *nightly* and *beta*
channels) and clone this repository together with its submodules:

```bash
git clone --recurse-submodules https://github.com/mira-merkell/clap-clap
```

Build the example plugin with:

```bash
cargo build -p ping-pong --release
```

and look for the compiled dynamical library in `target/release/`.

The name of the library is OS-specific. For example, on Linux it should be:
`libping_pong.so`, whereas on Windows it's `ping_pong.dll`. Copy the file to
where your DAW can find it and rename it to: `ping_pong.clap`.
