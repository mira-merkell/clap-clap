use std::{
    sync::{
        Arc,
        atomic::{AtomicU32, Ordering},
    },
    time::SystemTime,
};

use clap_clap::{
    Error,
    host::Host,
    plugin::{AudioThread, Plugin},
    process::{Process, Status, Status::Continue},
};

#[derive(Default)]
pub struct TestPlugin {
    id: Option<SystemTime>,
    returned_id: Option<SystemTime>,
    call_init: Option<Arc<Host>>,
    call_activate: Option<(f64, u32, u32)>,
    call_on_main_thread: AtomicU32,
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
