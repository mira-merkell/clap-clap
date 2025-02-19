use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

use clap_clap::{
    events::{InputEvents, OutputEvents},
    ext::params::{Error, ParamInfo, ParamInfoFlags, Params},
    id::ClapId,
    prelude as clap,
};

// A plugin must implement `Default` trait.  The plugin instance will be created
// by the host with the call to `MyPlug::default()`.
struct Gain {
    gain: Arc<AtomicU64>,
}

impl Default for Gain {
    fn default() -> Self {
        Self {
            gain: Arc::new(AtomicU64::new(1.0f64.to_bits())),
        }
    }
}

struct GainParam;

impl clap::Extensions<Self> for Gain {
    // Provide CLAP "plugin_audio_ports" extension: for example,
    // a static layout of stereo ports, one in and one out.
    // If the plugin needs to dynamically control the port layout,
    // you might want to implement the AudioPorts trait yourself.
    fn audio_ports() -> Option<impl clap::AudioPorts<Self>> {
        Some(clap::StereoPorts::<1, 1>)
    }

    fn params() -> Option<impl Params<Self>> {
        Some(GainParam)
    }
}

impl Params<Gain> for GainParam {
    fn count(plugin: &Gain) -> u32 {
        1
    }

    fn get_info(plugin: &Gain, param_index: u32) -> Option<ParamInfo> {
        if param_index == 0 {
            Some(ParamInfo {
                id: ClapId::from(0),
                flags: ParamInfoFlags::RequiresProcess as u32
                    | ParamInfoFlags::IsModulatable as u32,
                cookie: None,
                name: "Gain".to_string(),
                module: "gain".to_string(),
                min_value: 0.0,
                max_value: 2.0,
                default_value: 1.0,
            })
        } else {
            None
        }
    }

    fn get_value(plugin: &Gain, param_id: ClapId) -> Option<f64> {
        if param_id == ClapId::from(0) {
            Some(f64::from_bits(plugin.gain.load(Ordering::Relaxed)))
        } else {
            None
        }
    }

    fn value_to_text(
        plugin: &Gain,
        param_id: ClapId,
        value: f64,
        out_buf: &mut [u8],
    ) -> Result<(), Error> {
        Ok(())
    }

    fn text_to_value(
        plugin: &Gain,
        param_id: ClapId,
        param_value_text: &str,
    ) -> Result<f64, Error> {
        Ok(0.0)
    }

    fn flush_inactive(plugin: &Gain, in_events: &InputEvents, out_events: &OutputEvents) {}

    fn flush(
        audio_thread: &<Gain as clap::Plugin>::AudioThread,
        in_events: &InputEvents,
        out_events: &OutputEvents,
    ) {
    }
}

impl clap::Plugin for Gain {
    type AudioThread = AudioThread;
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

    fn init(&mut self, host: Arc<clap::Host>) -> Result<(), clap::Error> {
        Ok(())
    }

    /// Start the audio thread.
    fn activate(&mut self, _: f64, _: u32, _: u32) -> Result<AudioThread, clap::Error> {
        Ok(AudioThread {
            gain: self.gain.clone(),
            smoothed: OnePole {
                b0: 1.0,
                a1: -0.999,
                y1: 0.0,
            },
        })
    }
}

struct AudioThread {
    gain: Arc<AtomicU64>,
    smoothed: OnePole,
}

impl clap::AudioThread<Gain> for AudioThread {
    fn process(&mut self, process: &mut clap::Process) -> Result<clap::Status, clap::Error> {
        let mut gain = f64::from_bits(self.gain.load(Ordering::Relaxed));

        let nframes = process.frames_count();
        let nev = process.in_events().size();
        let mut ev_index = 0;
        let mut next_ev_frame = if nev > 0 { 0 } else { nframes };

        let mut i = 0;
        while i < nframes {
            while ev_index < nev && next_ev_frame == i {
                {
                    let in_events = process.in_events();
                    let header = in_events.get(ev_index);
                    if header.time() != i {
                        next_ev_frame = header.time();
                        break;
                    }

                    if let Ok(param_value) = header.param_value() {
                        gain = param_value.value();
                        self.gain.store(gain.to_bits(), Ordering::Release);
                    }
                }

                ev_index += 1;

                if ev_index == nev {
                    next_ev_frame = nframes;
                    break;
                }
            }

            {
                let i = i as usize;
                let gain = gain as f32;

                // Get the input signal from the main input port.
                let in_l = process.audio_inputs(0).data32(0)[i];
                let in_r = process.audio_inputs(0).data32(1)[i];

                // Process samples. Here we simply swap left and right channels.
                let smoothed = self.smoothed.tick(gain);
                let out_l = in_r * smoothed;
                let out_r = in_l * smoothed;

                // Write the audio signal to the main output port.
                process.audio_outputs(0).data32(0)[i] = out_l;
                process.audio_outputs(0).data32(1)[i] = out_r;
            }

            i += 1;
        }
        Ok(clap::Status::Continue)
    }
}

// Export clap_entry symbols and build a plugin factory.
clap::entry!(Gain);

#[derive(Debug, Clone)]
pub struct OnePole {
    b0: f32,
    a1: f32,
    y1: f32,
}

impl OnePole {
    fn tick(&mut self, sample: f32) -> f32 {
        let y0 = sample * self.b0 - self.y1 * self.a1;
        self.y1 = y0;
        y0 * (1.0 + self.a1)
    }
}
