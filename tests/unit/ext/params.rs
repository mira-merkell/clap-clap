use clap_clap::{
    Error,
    events::{InputEvents, OutputEvents},
    ext::{
        Extensions,
        params::{ParamInfo, Params},
    },
    id::ClapId,
    plugin::Plugin,
};

use crate::ext::{TestBed, TestConfig};

#[derive(Default)]
struct TestPlugin;

impl Plugin for TestPlugin {
    type AudioThread = ();
    const ID: &'static str = "";
    const NAME: &'static str = "";

    fn activate(&mut self, _: f64, _: u32, _: u32) -> Result<Self::AudioThread, Error> {
        Ok(())
    }
}

impl Extensions<Self> for TestPlugin {
    fn params() -> Option<impl Params<TestPlugin>> {
        Some(TestParams {})
    }
}

struct TestParams;

impl Params<TestPlugin> for TestParams {
    fn count(plugin: &TestPlugin) -> u32 {
        3
    }

    fn get_info(plugin: &TestPlugin, param_index: u32) -> Option<ParamInfo> {
        None
    }

    fn get_value(plugin: &TestPlugin, param_id: ClapId) -> Option<f64> {
        None
    }

    fn value_to_text(
        plugin: &TestPlugin,
        param_id: ClapId,
        value: f64,
        out_buf: &mut [u8],
    ) -> Result<(), clap_clap::ext::params::Error> {
        Ok(())
    }

    fn text_to_value(
        plugin: &TestPlugin,
        param_id: ClapId,
        param_value_text: &str,
    ) -> Result<f64, clap_clap::ext::params::Error> {
        Ok(0.0)
    }

    fn flush_inactive(plugin: &TestPlugin, in_events: &InputEvents, out_events: &OutputEvents) {}

    fn flush(
        audio_thread: &<TestPlugin as Plugin>::AudioThread,
        in_events: &InputEvents,
        out_events: &OutputEvents,
    ) {
    }
}

#[test]
fn check_params_count() {
    let bed = TestBed::<TestPlugin>::new(TestConfig::default());

    let params = bed.ext_params.as_ref().unwrap();

    assert_eq!(params.count(), 3);
}
