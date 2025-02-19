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

use crate::{
    ext::{TestBed, TestConfig},
    shims::plugin::ShimPlugin,
};

#[test]
fn no_impl_params() {
    let bed = TestBed::<ShimPlugin>::new(TestConfig::default());
    assert!(bed.ext_params.is_none())
}

struct TestPlugin {
    info: Vec<ParamInfo>,
}

impl Default for TestPlugin {
    fn default() -> Self {
        Self {
            info: vec![
                ParamInfo {
                    id: ClapId::from(1),
                    flags: 1,
                    name: "u93".to_string(),
                    module: "eu/o33".to_string(),
                    min_value: 0.0,
                    max_value: 1.0,
                    default_value: 1.0,
                },
                ParamInfo {
                    id: ClapId::from(2),
                    flags: 2,
                    name: "ee3".to_string(),
                    module: "euo0".to_string(),
                    min_value: -10.0,
                    max_value: 10.0,
                    default_value: 0.10,
                },
            ],
        }
    }
}

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
        plugin.info.len() as u32
    }

    fn get_info(plugin: &TestPlugin, param_index: u32) -> Option<ParamInfo> {
        (param_index < plugin.info.len() as u32).then(|| plugin.info[param_index as usize].clone())
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

    assert_eq!(params.count(), TestPlugin::default().info.len() as u32);
}

#[test]
fn check_params_get_info() {
    let bed = TestBed::<TestPlugin>::new(TestConfig::default());

    let params = bed.ext_params.as_ref().unwrap();

    assert_eq!(params.count(), 2);
    assert_eq!(params.get_info(0).unwrap(), TestPlugin::default().info[0]);
    assert_eq!(params.get_info(1).unwrap(), TestPlugin::default().info[1]);
}
