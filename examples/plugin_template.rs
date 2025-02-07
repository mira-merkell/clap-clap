//! This file is here to demonstrate how to wire a CLAP plugin using
//! [`clap_clap`] library. You can use it as a starting point.
//!
//! This is an example of a simple audio processor that declares two stereo
//! ports, one in and one out, and simply swaps the left/right audio channels.
//! It is a modified version of the [plugin template] from the original CLAP
//! repository, and is meant to resemble that code as much as possible.
//!
//! The plugin should be compiled as a dynamical library with C ABI.  You can
//! specify the library type in `Cargo.toml` of your crate:
//!
//! ```toml
//! [lib]
//! crate-type = ["cdylib"]
//! ```
//!
//! [`clap_clap`]: https://github.com/mira-merkell/clap-clap
//! [plugin template]: https://github.com/free-audio/clap/blob/main/src/plugin-template.c

use std::sync::Arc;

use clap_clap::{
    Error,
    ext::{AudioPorts, Extensions, plugin::audio_ports::StereoPorts},
    host::Host,
    plugin::{AudioThread, Plugin},
    process::{Process, Status, Status::Continue},
};

// A plugin must implement `Default` trait.  The plugin instance will be created
// by the host with the call to `MyPlug::default()`.
#[derive(Default)]
struct MyPlug {
    host: Option<Arc<Host>>,
}

impl Extensions<Self> for MyPlug {
    // Provide CLAP "audio_ports" extension: for example,
    // a static layout of stereo ports, one in and one out.
    // If the plugin needs to dynamically control the port layout,
    // you might want to implement the AudioPorts trait yourself.
    fn audio_ports() -> Option<impl AudioPorts<Self>> {
        Some(StereoPorts::<1, 1>)
    }
}

impl Plugin for MyPlug {
    type AudioThread = ChannelSwap;
    type Extensions = Self;

    const ID: &'static str = "com.your-company.YourPlugin";
    const NAME: &'static str = "Plugin Name";
    const VENDOR: &'static str = "Vendor";
    const URL: &'static str = "https://your-domain.com/your-plugin";
    const MANUAL_URL: &'static str = "https://your-domain.com/your-plugin/manual";
    const SUPPORT_URL: &'static str = "https://your-domain.com/support";
    const VERSION: &'static str = "1.4.2";
    const DESCRIPTION: &'static str = "The plugin description.";
    const FEATURES: &'static str = "instrument stereo";

    fn init(&mut self, host: Arc<Host>) -> Result<(), Error> {
        // Store a reference to the host.
        self.host = Some(host.clone());

        // We can retrieve host extensions here. E.g., the logging facility:
        host.get_extension().log()?.info("hello, sonic world")?;
        Ok(())
    }

    /// Start the audio thread.
    fn activate(&mut self, _sample_rate: f64, _: u32, _: u32) -> Result<ChannelSwap, Error> {
        Ok(ChannelSwap {})
    }
}

/// Declare an audio processor.
/// Instances of this type will live on the audio thread.
struct ChannelSwap {}

impl AudioThread<MyPlug> for ChannelSwap {
    fn process(&mut self, process: &mut Process) -> Result<Status, Error> {
        // Generate a lending iterator over frames of audio samples and events.
        // The entire Process API, together with its derived types, is `const`.
        // The methods are cheap to call in a loop on the audio thread.
        let mut frames = process.frames();
        while let Some(frame) = frames.next() {
            // Get the input signal from the main input port.
            let in_l = frame.audio_input(0).data32(0);
            let in_r = frame.audio_input(0).data32(1);

            // Process samples. Here we simply swap left and right channels.
            let out_l = in_r;
            let out_r = in_l;

            // Write the audio signal to the main output port.
            *frame.audio_output(0).data32(0) = out_l;
            *frame.audio_output(0).data32(1) = out_r;
        }
        Ok(Continue)
    }
}

// Export clap_entry symbols and build a plugin factory.
clap_clap::entry!(MyPlug);
