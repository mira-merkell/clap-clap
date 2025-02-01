use std::{
    fmt::Display,
    marker::PhantomData,
    ops::Deref,
    sync::{Arc, Mutex},
};

use clap_sys::clap_plugin;

use crate::{
    ext::{Extensions, audio_ports::ClapPluginAudioPorts},
    host::Host,
    process,
    process::{Process, Status::Continue},
};

mod desc;

pub(crate) use desc::PluginDescriptor;

mod ffi;

pub trait Plugin: Default {
    type AudioThread: AudioThread<Self>;
    type Extensions: Extensions<Self>;

    const ID: &'static str;
    const NAME: &'static str = "";
    const VENDOR: &'static str = "";
    const URL: &'static str = "";
    const MANUAL_URL: &'static str = "";
    const SUPPORT_URL: &'static str = "";
    const VERSION: &'static str = "";
    const DESCRIPTION: &'static str = "";

    /// Arbitrary keywords separated by whitespace.
    /// For example: `"fx stereo distortion"`.
    const FEATURES: &'static str = "";

    #[allow(unused_variables)]
    fn init(&mut self, host: Arc<Host>) -> Result<(), crate::Error> {
        Ok(())
    }

    fn activate(
        &mut self,
        sample_rate: f64,
        min_frames: usize,
        max_frames: usize,
    ) -> Result<Self::AudioThread, crate::Error>;

    fn on_main_thread(&mut self) {}
}

pub trait AudioThread<P: Plugin>: Send + Sync + Sized {
    fn start_processing(&mut self) -> Result<(), crate::Error> {
        Ok(())
    }

    fn stop_processing(&mut self) {}

    fn process(&mut self, process: &mut Process) -> Result<process::Status, crate::Error>;

    fn reset(&mut self) {}

    #[allow(unused_variables)]
    fn deactivate(self, plugin: &mut P) {}
}

impl<P: Plugin> AudioThread<P> for () {
    fn process(&mut self, _: &mut Process) -> Result<process::Status, crate::Error> {
        Ok(Continue)
    }
}

pub(crate) struct ClapPluginExtensions<P> {
    pub(crate) audio_ports: Option<ClapPluginAudioPorts<P>>,
}

impl<P: Plugin> ClapPluginExtensions<P> {
    fn new() -> Self {
        Self {
            audio_ports: P::Extensions::audio_ports().map(ClapPluginAudioPorts::new),
        }
    }
}

pub(crate) struct Runtime<P: Plugin> {
    pub(crate) audio_thread: Option<P::AudioThread>,
    pub(crate) descriptor: PluginDescriptor<P>,
    pub(crate) host: Arc<Host>,
    pub(crate) plugin: P,
    pub(crate) plugin_extensions: Mutex<ClapPluginExtensions<P>>,
}

impl<P: Plugin> Runtime<P> {
    pub(crate) fn initialize(host: Arc<Host>) -> Self {
        Self {
            descriptor: PluginDescriptor::allocate(),
            plugin: P::default(),
            audio_thread: None,
            host,
            plugin_extensions: Mutex::new(ClapPluginExtensions::new()),
        }
    }

    pub(crate) fn into_clap_plugin(self) -> ClapPlugin<P> {
        // Safety:
        // The leaked (via Box::into_raw) pointer satisfies requirements
        // for a safe call to ClapPlugin::new():
        // 1. it is non-null
        // 2. it represents a valid clap_plugin tied to type P.
        unsafe { ClapPlugin::new(Box::into_raw(ffi::box_clap_plugin(self))) }
    }

    /// Retake ownership of the runtime from the pointer to  clap_plugin.
    ///
    /// # Safety:
    ///
    /// The caller must assure it's only them who have access to the entire
    /// runtime: both main thread and the audio thread.
    /// This can requirement can be met during plugin initialization and
    /// destruction.
    unsafe fn from_clap_plugin(clap_plugin: ClapPlugin<P>) -> Self {
        let plugin_data = clap_plugin.plugin_data as *mut _;
        // Safety:
        // We can transmute the pointer to plugin_data like this, because:
        // 1. We have exclusive reference to it.
        // 2. We know the pointer's real type because of the constraints put on the
        //    constructor of ClapPlugin.
        // 3. We know that the pointer was initially leaked with Box::into_raw().
        *unsafe { Box::from_raw(plugin_data) }
    }
}

/// Safe wrapper around clap_plugin.
pub(crate) struct ClapPlugin<P: Plugin>(*const clap_plugin, PhantomData<P>);

impl<P: Plugin> ClapPlugin<P> {
    /// # Safety
    ///
    /// 1. The user must assure the pointer to plugin is non-null.
    /// 2. The pointer must point to a valid clap_plugin structure tied to the
    ///    plugin type P, and living in the host.
    ///
    /// Typically, a valid pointer comes from the host calling the plugin's
    /// methods, or from Runtime::into_clap_plugin()
    pub(crate) const unsafe fn new(clap_plugin: *const clap_plugin) -> Self {
        Self(clap_plugin, PhantomData)
    }

    pub(crate) const fn into_inner(self) -> *const clap_plugin {
        self.0
    }

    /// Obtain a mutable reference to the entire runtime.
    ///
    /// # Safety
    ///
    /// The caller must assure they're the only ones who access the runtime.
    pub(crate) const unsafe fn runtime(&mut self) -> &mut Runtime<P> {
        let runtime: *mut Runtime<P> = unsafe { *self.0 }.plugin_data as *mut _;
        unsafe { &mut *runtime }
    }

    /// Obtain a mutable reference to plugin.
    ///
    /// # Safety
    ///
    /// The caller must assure they're the only ones who access the plugin.
    pub(crate) const unsafe fn plugin(&mut self) -> &mut P {
        let runtime: *mut Runtime<P> = unsafe { *self.0 }.plugin_data as *mut _;
        unsafe { &mut (*runtime).plugin }
    }

    /// Obtain a mutable reference to audio thread.
    ///
    /// # Safety
    ///
    /// The caller must assure they're the only ones who access the
    /// audio_thread.
    pub(crate) const unsafe fn audio_thread(&mut self) -> Option<&mut P::AudioThread> {
        let runtime: *mut Runtime<P> = unsafe { *self.0 }.plugin_data as *mut _;
        unsafe { &mut (*runtime).audio_thread }.as_mut()
    }

    /// Obtain a mutex to plugin extensions.
    pub(crate) const fn plugin_extensions(&mut self) -> &Mutex<ClapPluginExtensions<P>> {
        let runtime: *mut Runtime<P> = unsafe { *self.0 }.plugin_data as *mut _;
        unsafe { &(*runtime).plugin_extensions }
    }
}

impl<P: Plugin> Deref for ClapPlugin<P> {
    type Target = clap_plugin;

    fn deref(&self) -> &Self::Target {
        // Safety:
        // ClapPlugin constructor guarantees that this is safe.
        unsafe { &*self.0 }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Error {}

impl Display for Error {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        Ok(())
    }
}

impl std::error::Error for Error {}

impl From<Error> for crate::Error {
    fn from(value: Error) -> Self {
        Self::Plugin(value)
    }
}
