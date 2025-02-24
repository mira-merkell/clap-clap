use std::{
    ffi::CStr,
    ops::{Deref, DerefMut},
    sync::{
        Arc, LazyLock,
        atomic::{AtomicU32, AtomicU64, Ordering},
    },
    time::SystemTime,
};

use clap_clap::{
    Error,
    ext::{
        Extensions,
        audio_ports::{AudioPortInfo, AudioPorts},
    },
    factory::{Factory, FactoryHost, FactoryPluginPrototype},
    ffi::{CLAP_EXT_AUDIO_PORTS, clap_plugin_audio_ports},
    host::Host,
    plugin::{AudioThread, ClapPlugin, Plugin},
    process::{Process, Status, Status::Continue},
};

use crate::{process::TestProcessConfig, shims::host::SHIM_CLAP_HOST};

#[cfg(test)]
mod desc;

#[derive(Default)]
pub struct TestPlugin {
    id: Option<SystemTime>,
    return_id: Option<SystemTime>,
    call_init: Option<Arc<Host>>,
    call_activate: Option<(f64, u32, u32)>,
    call_on_main_thread: AtomicU32,
    call_get_extension: AtomicU64,
}

static CALL_PLUGIN_DESTRUCTOR: AtomicU32 = AtomicU32::new(0);

impl Drop for TestPlugin {
    fn drop(&mut self) {
        CALL_PLUGIN_DESTRUCTOR.fetch_add(1, Ordering::Release);
    }
}

impl Plugin for TestPlugin {
    type AudioThread = TestAudioThread;
    const ID: &'static str = "clap.plugin.test";
    const NAME: &'static str = "Test Plugin";
    const VENDOR: &'static str = "⧉⧉⧉";
    const URL: &'static str = "none";
    const MANUAL_URL: &'static str = "manual none";
    const SUPPORT_URL: &'static str = "support none";
    const VERSION: &'static str = "0.0.099";
    const DESCRIPTION: &'static str = "test plugin";

    fn features() -> impl Iterator<Item = &'static str> {
        "test audio Allpass other features too: ⧉⧉⧉".split_whitespace()
    }

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

impl Extensions<Self> for TestPlugin {
    fn audio_ports() -> Option<impl AudioPorts<Self>> {
        Some(TestAudioPorts)
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
        #[allow(clippy::redundant_pattern_matching)]
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
        FactoryPluginPrototype::<TestPlugin>::build().unwrap(),
    )])
});

unsafe fn build_plugin<P: Plugin>() -> ClapPlugin<P> {
    let plugin = FACTORY
        .create_plugin(c"clap.plugin.test", unsafe {
            FactoryHost::new_unchecked(SHIM_CLAP_HOST.as_ref())
        })
        .unwrap();

    unsafe { ClapPlugin::new_unchecked(plugin) }
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

    let plugin_features: Vec<_> = TestPlugin::features().collect();

    assert_eq!(features, plugin_features);
}

#[test]
fn call_init() {
    let mut wrap = TestWrapper::build();

    let clap_plugin = unsafe { wrap.as_ref() };
    assert!(unsafe { clap_plugin.init.unwrap()(clap_plugin) });

    let plugin = unsafe { wrap.plugin() };
    let host = plugin.call_init.as_ref().unwrap();
    assert_eq!(host.name(), "");
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

#[test]
fn call_start_processing() {
    let mut wrap = TestWrapper::build();

    let clap_plugin = unsafe { wrap.as_ref() };
    unsafe { clap_plugin.init.unwrap()(clap_plugin) };
    unsafe { clap_plugin.activate.unwrap()(clap_plugin, 1.0, 2, 3) };

    unsafe { clap_plugin.start_processing.unwrap()(clap_plugin) };

    let audio_thread = unsafe { wrap.audio_thread() }.unwrap();
    assert!(audio_thread.call_start_processing.is_some());

    unsafe { clap_plugin.deactivate.unwrap()(clap_plugin) };
}

#[test]
fn call_stop_processing() {
    let mut wrap = TestWrapper::build();

    let clap_plugin = unsafe { wrap.as_ref() };
    unsafe { clap_plugin.init.unwrap()(clap_plugin) };
    unsafe { clap_plugin.activate.unwrap()(clap_plugin, 1.0, 2, 3) };

    unsafe { clap_plugin.stop_processing.unwrap()(clap_plugin) };

    let audio_thread = unsafe { wrap.audio_thread() }.unwrap();
    assert!(audio_thread.call_stop_processing.is_some());

    unsafe { clap_plugin.deactivate.unwrap()(clap_plugin) };
}

#[test]
fn call_reset() {
    let mut wrap = TestWrapper::build();

    let clap_plugin = unsafe { wrap.as_ref() };
    unsafe { clap_plugin.init.unwrap()(clap_plugin) };
    unsafe { clap_plugin.activate.unwrap()(clap_plugin, 1.0, 2, 3) };

    unsafe { clap_plugin.reset.unwrap()(clap_plugin) };

    let audio_thread = unsafe { wrap.audio_thread() }.unwrap();
    assert!(audio_thread.call_reset.is_some());

    unsafe { clap_plugin.deactivate.unwrap()(clap_plugin) };
}

#[test]
fn call_process() {
    let mut wrap = TestWrapper::build();

    let clap_plugin = unsafe { wrap.as_ref() };
    unsafe { clap_plugin.init.unwrap()(clap_plugin) };
    unsafe { clap_plugin.activate.unwrap()(clap_plugin, 1.0, 2, 3) };

    let mut process = TestProcessConfig {
        latency: 0,
        steady_time: 0,
        frames_count: 3,
        channel_count: 0,
        audio_inputs_count: 0,
        audio_outputs_count: 0,
    }
    .build();
    let clap_process = process.clap_process();
    unsafe { clap_plugin.process.unwrap()(clap_plugin, &clap_process) };

    let audio_thread = unsafe { wrap.audio_thread() }.unwrap();
    assert_eq!(audio_thread.frames_processed, 3);

    unsafe { clap_plugin.deactivate.unwrap()(clap_plugin) };
}

pub struct TestAudioPorts;

impl AudioPorts<TestPlugin> for TestAudioPorts {
    fn count(plugin: &TestPlugin, _: bool) -> u32 {
        // As a signature, set the first bit of plugin's call_get_extension field.
        plugin.call_get_extension.fetch_or(1, Ordering::Release);
        11
    }

    fn get(_: &TestPlugin, _: u32, _: bool) -> Option<AudioPortInfo> {
        None
    }
}

#[test]
fn call_get_extension_audio_ports() {
    let mut wrap = TestWrapper::build();
    let audio_ins = TestAudioPorts::count(unsafe { wrap.plugin() }, true);

    let clap_plugin = unsafe { wrap.as_ref() };

    // Fetch clap_plugin_audio_ports extension.
    let ext =
        unsafe { clap_plugin.get_extension.unwrap()(clap_plugin, CLAP_EXT_AUDIO_PORTS.as_ptr()) };
    assert!(!ext.is_null());
    let ext = ext as *const clap_plugin_audio_ports;

    assert_eq!(
        unsafe { (*ext).count.unwrap()(clap_plugin, true) },
        audio_ins
    );

    // Check if it was actually this plugin that was called.
    // TestAudioPorts sets the first bit of plugin.call_get_extensions.
    let plugin = unsafe { wrap.plugin() };
    assert_eq!(plugin.call_get_extension.load(Ordering::Acquire) & 1, 1);
}

#[test]
fn is_active() {
    let wrap = TestWrapper::build();

    let clap_plugin = unsafe { wrap.as_ref() };
    unsafe { clap_plugin.init.unwrap()(clap_plugin) };

    assert!(!wrap.is_active());
    unsafe { clap_plugin.activate.unwrap()(clap_plugin, 1.1, 1, 7) };
    assert!(wrap.is_active());
}

#[test]
fn is_inactive() {
    let wrap = TestWrapper::build();

    let clap_plugin = unsafe { wrap.as_ref() };
    unsafe { clap_plugin.init.unwrap()(clap_plugin) };
    assert!(!wrap.is_active());
    unsafe { clap_plugin.activate.unwrap()(clap_plugin, 1.1, 1, 7) };
    assert!(wrap.is_active());
    unsafe { clap_plugin.deactivate.unwrap()(clap_plugin) }
    assert!(!wrap.is_active());
}
