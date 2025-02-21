use std::{
    cell::UnsafeCell,
    sync::{Arc, Mutex},
};

use clap_clap::{
    Error,
    events::{InputEvents, OutputEvents},
    ext::{
        Extensions,
        params::{Error::ConvertToValue, ParamInfo, Params},
    },
    id::ClapId,
    plugin::{AudioThread, Plugin},
    prelude::{Process, Status, Status::Continue},
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
    call_flush: UnsafeCell<bool>,
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
            call_flush: UnsafeCell::new(false),
        }
    }
}

impl Plugin for TestPlugin {
    type AudioThread = TestAudioThread;
    const ID: &'static str = "";
    const NAME: &'static str = "";

    fn activate(&mut self, _: f64, _: u32, _: u32) -> Result<Self::AudioThread, Error> {
        Ok(TestAudioThread {
            call_flush: Arc::new(Mutex::new(false)),
        })
    }
}

struct TestAudioThread {
    call_flush: Arc<Mutex<bool>>,
}

impl AudioThread<TestPlugin> for TestAudioThread {
    fn process(&mut self, _: &mut Process) -> Result<Status, Error> {
        Ok(Continue)
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

    fn get_value(_: &TestPlugin, param_id: ClapId) -> Option<f64> {
        if param_id == ClapId::from(0) {
            Some(0.0)
        } else if param_id == ClapId::from(1) {
            Some(1.0)
        } else {
            None
        }
    }

    fn value_to_text(
        _: &TestPlugin,
        _: ClapId,
        value: f64,
        out_buf: &mut [u8],
    ) -> Result<(), clap_clap::ext::params::Error> {
        for (d, &s) in out_buf.iter_mut().zip(format!("{value:.3}").as_bytes()) {
            *d = s;
        }
        Ok(())
    }

    fn text_to_value(
        _: &TestPlugin,
        param_id: ClapId,
        param_value_text: &str,
    ) -> Result<f64, clap_clap::ext::params::Error> {
        if param_id != ClapId::from(0) {
            return Err(ConvertToValue);
        }

        param_value_text.parse().map_err(|_| ConvertToValue)
    }

    fn flush_inactive(plugin: &TestPlugin, _: &InputEvents, _: &OutputEvents) {
        unsafe {
            *plugin.call_flush.get() = true;
        }
    }

    fn flush(
        audio_thread: &<TestPlugin as Plugin>::AudioThread,
        _: &InputEvents,
        _: &OutputEvents,
    ) {
        let mut call = audio_thread.call_flush.lock().unwrap();
        *call = true;
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

#[test]
fn check_get_value() {
    let bed = TestBed::<TestPlugin>::new(TestConfig::default());

    let params = bed.ext_params.as_ref().unwrap();

    assert_eq!(params.get_value(ClapId::from(0)), Some(0.0));
    assert_eq!(params.get_value(ClapId::from(1)), Some(1.0));
    assert_eq!(params.get_value(ClapId::from(2)), None);
}

#[test]
fn check_value_to_text_01() {
    let bed = TestBed::<TestPlugin>::new(TestConfig::default());

    let params = bed.ext_params.as_ref().unwrap();

    let mut buf = vec![0; 3];
    params
        .value_to_text(ClapId::from(0), 1.0195, &mut buf)
        .unwrap();
    assert_eq!(String::from_utf8(buf).unwrap(), "1.0");
}

#[test]
fn check_value_to_text_02() {
    let bed = TestBed::<TestPlugin>::new(TestConfig::default());

    let params = bed.ext_params.as_ref().unwrap();

    let mut buf = vec![0; 0];
    params
        .value_to_text(ClapId::from(0), 1.0195, &mut buf)
        .unwrap();
    assert_eq!(String::from_utf8(buf).unwrap(), "");
}

#[test]
fn check_value_to_text_03() {
    let bed = TestBed::<TestPlugin>::new(TestConfig::default());

    let params = bed.ext_params.as_ref().unwrap();

    let mut buf = vec![0; 6];
    params
        .value_to_text(ClapId::from(0), 1.0195, &mut buf)
        .unwrap();
    assert_eq!(String::from_utf8(buf).unwrap(), "1.020\0");
}

#[test]
fn check_text_to_value() {
    let bed = TestBed::<TestPlugin>::new(TestConfig::default());

    let params = bed.ext_params.as_ref().unwrap();

    assert_eq!(params.text_to_value(ClapId::from(0), "0").unwrap(), 0.0);
    assert_eq!(params.text_to_value(ClapId::from(0), "0.0").unwrap(), 0.0);
    assert_eq!(params.text_to_value(ClapId::from(0), "0.1").unwrap(), 0.1);
    assert_eq!(params.text_to_value(ClapId::from(0), "-0.1").unwrap(), -0.1);
    assert_eq!(params.text_to_value(ClapId::from(0), ".1").unwrap(), 0.1);
    assert_eq!(
        params.text_to_value(ClapId::from(0), ".l/o.1").unwrap_err(),
        ConvertToValue
    );
    assert_eq!(
        params.text_to_value(ClapId::from(1), "").unwrap_err(),
        ConvertToValue
    );
    assert_eq!(
        params.text_to_value(ClapId::from(2), "").unwrap_err(),
        ConvertToValue
    );
}

#[test]
fn check_flush_inactive() {
    let bed = TestBed::<TestPlugin>::new(TestConfig::default());
    let params = bed.ext_params.as_ref().unwrap();

    assert!(!bed.plugin().is_active());
    {
        let mut plugin = bed.plugin();
        assert!(!*unsafe { plugin.plugin().call_flush.get_mut() });
    }
    params.flush();
    {
        let mut plugin = bed.plugin();
        assert!(*unsafe { plugin.plugin().call_flush.get_mut() });
    }
}

#[test]
fn check_flush_active() {
    let bed = TestBed::<TestPlugin>::new(TestConfig::default());
    let params = bed.ext_params.as_ref().unwrap();

    bed.activate();
    assert!(bed.plugin().is_active());
    {
        let mut plugin = bed.plugin();
        let call = unsafe { plugin.audio_thread().unwrap().call_flush.lock().unwrap() };
        assert!(!*call);
    }
    params.flush();
    {
        let mut plugin = bed.plugin();
        let call = unsafe { plugin.audio_thread().unwrap().call_flush.lock().unwrap() };
        assert!(*call);
    }
}
