use std::{
    io::{Read, Write},
    mem,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

use clap_clap::prelude as clap;

// A plugin must implement `Default` trait.  The plugin instance will be created
// by the host with the call to `State::default()`.
struct Example {
    // Three independent parameters to save and load as the plugin's state.
    state: Arc<[AtomicU64; 3]>,
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
        3
    }

    fn get_info(_: &Example, param_index: u32) -> Option<clap::ParamInfo> {
        if param_index < 3 {
            Some(clap::ParamInfo {
                id: clap::ClapId::from(param_index as u16),
                flags: clap::params::InfoFlags::RequiresProcess as u32
                    // Some DAWs, e.g. Bitwig, display only automatable parameters.
                    | clap::params::InfoFlags::Automatable as u32,
                name: format!("Param {param_index}"),
                module: format!("{param_index}/param"),
                min_value: 0.0,
                max_value: 1.0,
                default_value: 0.0,
            })
        } else {
            None
        }
    }

    fn get_value(plugin: &Example, param_id: clap::ClapId) -> Option<f64> {
        let id: u32 = param_id.into();
        if id < 3 {
            Some(f64::from_bits(
                plugin.state[id as usize].load(Ordering::Relaxed),
            ))
        } else {
            None
        }
    }

    fn value_to_text(
        _: &Example,
        _: clap::ClapId,
        value: f64,
        out_buf: &mut [u8],
    ) -> Result<(), clap::Error> {
        for (out, &c) in out_buf.iter_mut().zip(format!("{value:.2}").as_bytes()) {
            *out = c;
        }
        Ok(())
    }

    fn text_to_value(
        _: &Example,
        _: clap::ClapId,
        param_value_text: &str,
    ) -> Result<f64, clap::Error> {
        param_value_text
            .parse()
            .map_err(|_| clap_clap::ext::params::Error::ConvertToValue.into())
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
        let buf: [u64; 3] = [0, 1, 2].map(|i| plugin.state[i].load(Ordering::Acquire));
        let buf: [u8; 24] = unsafe { mem::transmute(buf) };
        stream.write_all(&buf).map_err(Into::into)
    }

    fn load(plugin: &Example, stream: &mut clap::IStream) -> Result<(), clap::Error> {
        let mut buf: [u8; 24] = [0; 24];
        stream.read_exact(&mut buf)?;

        let buf: [u64; 3] = unsafe { mem::transmute(buf) };
        for i in 0..3 {
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
    state: Arc<[AtomicU64; 3]>,
}

impl clap::AudioThread<Example> for AudioThread {
    fn process(&mut self, process: &mut clap::Process) -> Result<clap::Status, clap::Error> {
        let in_events = process.in_events();

        for i in 0..in_events.size() {
            let header = in_events.get(i);

            if let Ok(param) = header.param_value() {
                let value = param.value();
                let id: u32 = param.param_id().into();

                if id < 3 {
                    self.state[id as usize].store(value.to_bits(), Ordering::Release);
                }
            }
        }
        Ok(clap::Continue)
    }
}

clap::entry!(Example);
