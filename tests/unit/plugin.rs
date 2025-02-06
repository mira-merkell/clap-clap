use std::{
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
    plugin::{AudioThread, Plugin},
    process::{Process, Status, Status::Continue},
};
use clap_sys::clap_plugin;

use crate::host::{TestHost, TestHostConfig};

#[derive(Default)]
pub struct TestPlugin {
    id: Option<SystemTime>,
    returned_id: Option<SystemTime>,
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
    const VERSION: &'static str = "0.0.0";
    const DESCRIPTION: &'static str = "test plugin";
    const FEATURES: &'static str = "test audio Allpass";

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
        plugin.returned_id = Some(self.plugin_id)
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

struct ClapPlugin(*const clap_plugin);

impl ClapPlugin {
    fn build() -> Self {
        let plugin = FACTORY
            .create_plugin(c"clap.plugin.test", unsafe {
                FactoryHost::new(HOST.as_clap_host())
            })
            .unwrap();

        assert!(unsafe { (*plugin).init.unwrap()(plugin) });
        ClapPlugin(plugin)
    }
}

impl Drop for ClapPlugin {
    fn drop(&mut self) {
        unsafe { (*self.0).destroy.unwrap()(self.0) };
    }
}

#[test]
fn call_plugin_destructor() {
    drop(ClapPlugin::build());

    assert!(CALL_PLUGIN_DESTRUCTOR.load(Ordering::Acquire) > 0);
}

#[test]
fn call_audio_thread_destructor() {
    let plugin = ClapPlugin::build();

    assert!(unsafe { (*plugin.0).activate.unwrap()(plugin.0, 0.0, 0, 0) });
    unsafe { (*plugin.0).deactivate.unwrap()(plugin.0) };

    assert!(CALL_AUDIO_THREAD_DESTRUCTOR.load(Ordering::Acquire) > 0);
}
