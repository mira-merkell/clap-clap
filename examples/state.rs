use std::{
    io::{Read, Write},
    mem,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

use clap_clap::prelude as clap;

const NUM_PARAMS: usize = 3;
const NUM_BYTES: usize = NUM_PARAMS * 8; // Parameters have type: f64.

// A plugin must implement `Default` trait.  The plugin instance will be created
// by the host with the call to `State::default()`.
struct Example {
    // Three independent parameters to save and load as the plugin's state.
    state: Arc<[AtomicU64; NUM_PARAMS]>,
}

impl Default for Example {
    fn default() -> Self {
        Self {
            state: Arc::new([
                AtomicU64::new(0.0f64.to_bits()),
                AtomicU64::new(0.0f64.to_bits()),
                AtomicU64::new(0.0f64.to_bits()),
            ]),
        }
    }
}

impl clap::Extensions<Self> for Example {
    fn params() -> Option<impl clap::Params<Self>> {
        Some(ExampleParams)
    }

    fn state() -> Option<impl clap::State<Self>> {
        Some(ExampleState)
    }
}

struct ExampleParams;

impl clap::Params<Example> for ExampleParams {
    fn count(_: &Example) -> u32 {
        NUM_PARAMS as u32
    }

    fn get_info(_: &Example, param_index: u32) -> Option<clap::ParamInfo> {
        (param_index < NUM_PARAMS as u32).then(|| {
            clap::ParamInfo {
                id: clap::ClapId::from(param_index as u16),
                flags: clap::params::InfoFlags::RequiresProcess as u32
                    // Some DAWs, e.g. Bitwig, display only automatable parameters.
                    | clap::params::InfoFlags::Automatable as u32,
                name: format!("Param {param_index}"),
                module: format!("{param_index}/param"),
                min_value: 0.0,
                max_value: 1.0,
                default_value: 0.0,
            }
        })
    }

    fn get_value(plugin: &Example, param_id: clap::ClapId) -> Option<f64> {
        let id: usize = param_id.into();
        (id < NUM_PARAMS).then(|| f64::from_bits(plugin.state[id].load(Ordering::Relaxed)))
    }

    fn value_to_text(
        _: &Example,
        _: clap::ClapId,
        value: f64,
        mut out_buf: &mut [u8],
    ) -> Result<(), clap::Error> {
        Ok(write!(out_buf, "{value:.2}")?)
    }

    fn text_to_value(
        _: &Example,
        _: clap::ClapId,
        param_value_text: &str,
    ) -> Result<f64, clap::Error> {
        Ok(param_value_text.parse()?)
    }

    fn flush_inactive(_: &Example, _: &clap::InputEvents, _: &clap::OutputEvents) {}

    fn flush(
        _: &<Example as clap::Plugin>::AudioThread,
        _: &clap::InputEvents,
        _: &clap::OutputEvents,
    ) {
    }
}

struct ExampleState;

impl clap::State<Example> for ExampleState {
    fn save(plugin: &Example, stream: &mut clap::OStream) -> Result<(), clap::Error> {
        let buf: [u64; NUM_PARAMS] =
            std::array::from_fn(|i| plugin.state[i].load(Ordering::Acquire));
        let buf: [u8; NUM_BYTES] = unsafe { mem::transmute(buf) };
        stream.write_all(&buf).map_err(Into::into)
    }

    fn load(plugin: &Example, stream: &mut clap::IStream) -> Result<(), clap::Error> {
        let mut buf: [u8; NUM_BYTES] = [0; NUM_BYTES];
        stream.read_exact(&mut buf)?;

        let buf: [u64; NUM_PARAMS] = unsafe { mem::transmute(buf) };
        for i in 0..NUM_PARAMS {
            plugin.state[i].store(buf[i], Ordering::Release);
        }

        Ok(())
    }
}

impl clap::Plugin for Example {
    type AudioThread = AudioThread;

    const ID: &'static str = "com.your-company.YourPlugin";
    const NAME: &'static str = "Plugin Name";
    const VENDOR: &'static str = "Vendor";
    const URL: &'static str = "https://your-domain.com/your-plugin";
    const MANUAL_URL: &'static str = "https://your-domain.com/your-plugin/manual";
    const SUPPORT_URL: &'static str = "https://your-domain.com/support";
    const VERSION: &'static str = "1.4.2";
    const DESCRIPTION: &'static str = "The plugin description.";

    fn features() -> impl Iterator<Item = &'static str> {
        "example parameter state".split_whitespace()
    }

    fn init(&mut self, _: Arc<clap::Host>) -> Result<(), clap::Error> {
        Ok(())
    }

    /// Start the audio thread.
    fn activate(&mut self, _: f64, _: u32, _: u32) -> Result<AudioThread, clap::Error> {
        Ok(AudioThread {
            state: self.state.clone(),
        })
    }
}

struct AudioThread {
    state: Arc<[AtomicU64; NUM_PARAMS]>,
}

impl clap::AudioThread<Example> for AudioThread {
    fn process(&mut self, process: &mut clap::Process) -> Result<clap::Status, clap::Error> {
        let in_events = process.in_events();

        for i in 0..in_events.size() {
            let header = in_events.get(i);

            if let Ok(param) = header.param_value() {
                let value = param.value();
                let id: usize = param.param_id().into();

                if id < NUM_PARAMS {
                    self.state[id].store(value.to_bits(), Ordering::Release);
                }
            }
        }
        Ok(clap::Continue)
    }
}

clap::entry!(Example);
