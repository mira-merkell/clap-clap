use std::{
    ffi::CStr,
    ops::{Deref, DerefMut},
    pin::Pin,
    sync::{
        Arc, LazyLock,
        atomic::{AtomicU32, Ordering},
    },
    time::SystemTime,
};

use clap_clap::{
    Error,
    factory::{Factory, FactoryHost, FactoryPluginDescriptor},
    host::Host,
    plugin::{AudioThread, ClapPlugin, Plugin},
    process::{Process, Status, Status::Continue},
};

use crate::host::{TestHost, TestHostConfig};

#[derive(Default)]
pub struct TestPlugin {
    id: Option<SystemTime>,
    return_id: Option<SystemTime>,
    call_init: Option<Arc<Host>>,
    call_activate: Option<(f64, u32, u32)>,
    call_on_main_thread: AtomicU32,
}

static CALL_PLUGIN_DESTRUCTOR: AtomicU32 = AtomicU32::new(0);

impl Drop for TestPlugin {
    fn drop(&mut self) {
        CALL_PLUGIN_DESTRUCTOR.fetch_add(1, Ordering::Release);
    }
}

impl Plugin for TestPlugin {
    type AudioThread = TestAudioThread;
    type Extensions = ();
    const ID: &'static str = "clap.plugin.test";
    const NAME: &'static str = "Test Plugin";
    const VENDOR: &'static str = "⧉⧉⧉";
    const URL: &'static str = "none";
    const MANUAL_URL: &'static str = "manual none";
    const SUPPORT_URL: &'static str = "support none";
    const VERSION: &'static str = "0.0.099";
    const DESCRIPTION: &'static str = "test plugin";
    const FEATURES: &'static str = "test audio Allpass other features too: ⧉⧉⧉";

    fn init(&mut self, host: Arc<Host>) -> Result<(), Error> {
        self.id = Some(SystemTime::now());
        self.call_init = Some(host);
        Ok(())
    }

    fn activate(
        &mut self,
        sample_rate: f64,
        min_frames: u32,
        max_frames: u32,
    ) -> Result<Self::AudioThread, Error> {
        self.call_activate = Some((sample_rate, min_frames, max_frames));
        Ok(TestAudioThread::new(self.id.unwrap()))
    }

    fn on_main_thread(&mut self) {
        self.call_on_main_thread.fetch_add(1, Ordering::Release);
    }
}

pub struct TestAudioThread {
    plugin_id: SystemTime,
    call_start_processing: Option<()>,
    call_stop_processing: Option<()>,
    call_reset: Option<()>,
    frames_processed: u64,
}

static CALL_AUDIO_THREAD_DESTRUCTOR: AtomicU32 = AtomicU32::new(0);

impl Drop for TestAudioThread {
    fn drop(&mut self) {
        CALL_AUDIO_THREAD_DESTRUCTOR.fetch_add(1, Ordering::Release);
    }
}

impl TestAudioThread {
    fn new(plugin_id: SystemTime) -> Self {
        Self {
            plugin_id,
            call_start_processing: None,
            call_stop_processing: None,
            call_reset: None,
            frames_processed: 0,
        }
    }
}

impl AudioThread<TestPlugin> for TestAudioThread {
    fn start_processing(&mut self) -> Result<(), Error> {
        self.call_start_processing = Some(());
        Ok(())
    }

    fn stop_processing(&mut self) {
        self.call_stop_processing = Some(());
    }

    fn process(&mut self, process: &mut Process) -> Result<Status, Error> {
        let mut frames = process.frames();
        while let Some(_) = frames.next() {
            self.frames_processed += 1;
        }
        Ok(Continue)
    }

    fn reset(&mut self) {
        self.call_reset = Some(());
    }

    fn deactivate(self, plugin: &mut TestPlugin) {
        plugin.return_id = Some(self.plugin_id)
    }
}

static FACTORY: LazyLock<Factory> = LazyLock::new(|| {
    Factory::new(vec![Box::new(
        FactoryPluginDescriptor::<TestPlugin>::build().unwrap(),
    )])
});

static HOST: LazyLock<Pin<Box<TestHost>>> = LazyLock::new(|| {
    TestHostConfig {
        name: "test_host",
        vendor: "mira-merkell",
        url: "none",
        version: "0.0.0",
    }
    .build()
});

unsafe fn build_plugin<P: Plugin>() -> ClapPlugin<P> {
    let plugin = FACTORY
        .create_plugin(c"clap.plugin.test", unsafe {
            FactoryHost::new(HOST.as_clap_host())
        })
        .unwrap();

    unsafe { ClapPlugin::new(plugin) }
}

unsafe fn destroy_plugin<P: Plugin>(plugin: ClapPlugin<P>) {
    unsafe { plugin.as_ref().destroy.unwrap()(plugin.as_ref()) };
}

#[test]
fn call_plugin_destructor() {
    let plugin = unsafe { build_plugin::<TestPlugin>() };

    unsafe { plugin.as_ref().destroy.unwrap()(plugin.as_ref()) };

    assert!(CALL_PLUGIN_DESTRUCTOR.load(Ordering::Acquire) > 0);
}

struct TestWrapper(Option<ClapPlugin<TestPlugin>>);

impl TestWrapper {
    fn build() -> Self {
        Self(Some(unsafe { build_plugin() }))
    }
}

impl Drop for TestWrapper {
    fn drop(&mut self) {
        unsafe { destroy_plugin(self.0.take().unwrap()) }
    }
}

impl Deref for TestWrapper {
    type Target = ClapPlugin<TestPlugin>;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref().unwrap()
    }
}

impl DerefMut for TestWrapper {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.as_mut().unwrap()
    }
}

#[test]
fn call_audio_thread_destructor() {
    let plugin = TestWrapper::build();
    let plugin = unsafe { plugin.as_ref() };

    assert!(unsafe { plugin.init.unwrap()(plugin) });
    assert!(unsafe { plugin.activate.unwrap()(plugin, 0.0, 0, 0) });
    unsafe { plugin.deactivate.unwrap()(plugin) };

    assert!(CALL_AUDIO_THREAD_DESTRUCTOR.load(Ordering::Acquire) > 0);
}

macro_rules! test_plugin_descriptor {
    ($($desc:tt, $plugin_desc:ident);* ) => {

        #[cfg(test)]
        mod plugin_descriptor {
            use super::*;
            use std::ffi::CStr;

            $(
                #[test]
                fn $desc() {
                    let plugin = TestWrapper::build();

                    let name = unsafe { CStr::from_ptr((*(plugin.as_ref().desc)).$desc) };
                    assert_eq!(TestPlugin::$plugin_desc, name.to_str().unwrap());
                }
            )*
        }
    };
}

test_plugin_descriptor!(
    id, ID;
    name, NAME;
    vendor, VENDOR;
    url, URL;
    manual_url, MANUAL_URL;
    support_url, SUPPORT_URL;
    description, DESCRIPTION;
    version, VERSION
);

#[test]
fn plugin_descriptor_features() {
    let wrap = TestWrapper::build();
    let plugin = unsafe { wrap.as_ref() };
    let mut feat = unsafe { (*plugin.desc).features };

    let mut features = Vec::new();
    while !unsafe { *feat }.is_null() {
        features.push(unsafe { CStr::from_ptr(*feat).to_str().unwrap() });
        feat = unsafe { feat.add(1) };
    }

    let plugin_features: Vec<_> = TestPlugin::FEATURES.split_whitespace().collect();

    assert_eq!(features, plugin_features);
}

#[test]
fn call_init() {
    let mut wrap = TestWrapper::build();

    let clap_plugin = unsafe { wrap.as_ref() };
    assert!(unsafe { clap_plugin.init.unwrap()(clap_plugin) });

    let plugin = unsafe { wrap.plugin() };
    let host = plugin.call_init.as_ref().unwrap();
    assert_eq!(host.name(), "test_host");
}

#[test]
fn call_on_main_thread() {
    let mut wrap = TestWrapper::build();

    let clap_plugin = unsafe { wrap.as_ref() };
    unsafe { clap_plugin.on_main_thread.unwrap()(clap_plugin) };

    let plugin = unsafe { wrap.plugin() };
    assert_eq!(plugin.call_on_main_thread.load(Ordering::Acquire), 1);
}

#[test]
fn call_activate() {
    let mut wrap = TestWrapper::build();

    let clap_plugin = unsafe { wrap.as_ref() };
    unsafe { clap_plugin.init.unwrap()(clap_plugin) };
    unsafe { clap_plugin.activate.unwrap()(clap_plugin, 1.1, 1, 7) };

    let plugin = unsafe { wrap.plugin() };
    assert_eq!(plugin.call_activate, Some((1.1, 1, 7)));
}

#[test]
fn call_deactivate() {
    let mut wrap = TestWrapper::build();

    let clap_plugin = unsafe { wrap.as_ref() };
    unsafe { clap_plugin.init.unwrap()(clap_plugin) };
    unsafe { clap_plugin.activate.unwrap()(clap_plugin, 1.1, 1, 7) };
    unsafe { clap_plugin.deactivate.unwrap()(clap_plugin) };

    let plugin = unsafe { wrap.plugin() };
    assert!(plugin.return_id.is_some());
    assert_eq!(plugin.id, plugin.return_id);
}
