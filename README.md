# clap-clap

Another [CLAP] framework. Very much WIP. 🚧

## Goals

* Safe Rust interface to CLAP [C API].
* Follow CLAP module structure and terminology.
* Ergonomic and dynamical.
* Provide extensive testing and debugging platform for plugin authors.

[CLAP]: https://cleveraudio.org

[C API]: https://github.com/free-audio/clap/tree/main/include/clap

## Example

Implement a [ping-pong delay]:

```rust
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
    // Provide CLAP "audio ports" extension:
    // a pair of static stereo ports: one in and one out.
    fn audio_ports() -> Option<impl AudioPorts<PingPong>> {
        Some(StereoPorts)
    }
}

impl Plugin for PingPong {
    const ID: &'static str = "clap.example.ping_pong";
    const NAME: &'static str = "Ping-Pong";
    type Extensions = Self;

    /// Allocate resources: a stereo delay line, 1/8-second long.
    fn activate(&mut self, sample_rate: f64, _: usize, _: usize) -> Result<(), clap::Error> {
        let one_second = sample_rate as usize;
        self.del = vec![[0.0; 2]; one_second / 8];

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
```

[ping-pong delay]: ./examples/ping-pong/

## Compile the source code

Install *nightly* Rust (for the 2024 edition) and clone this repository together with its submodules:

```bash
git clone --recurse-submodules https://github.com/mira-merkell/clap-clap
```

Build the plugin with:

```bash
cargo build -r
```

and look for the compiled dynamical library in `target/release/`.

The name of the library is OS-specific. For example, on Linux it should be: `libping_pong.so`.
Copy the file to where your DAW can find it and rename it to: `ping_pong.clap`.
