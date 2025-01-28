use crate::ext::Extensions;
use crate::host::Host;
use crate::process::Process;
use crate::{ext::audio_ports::ClapPluginAudioPorts, process};
use clap_sys::clap_plugin;
use std::fmt::Display;
use std::sync::Arc;
use std::{marker::PhantomData, ptr::NonNull};

#[derive(Debug, Clone)]
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

pub trait Plugin: Default {
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

    type AudioThread: AudioThread<Self>;
    type Extensions: Extensions<Self>;

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

pub(crate) struct ClapPluginExtensions<P> {
    pub(crate) audio_ports: Option<ClapPluginAudioPorts<P>>,
}

pub(crate) struct Runtime<P: Plugin> {
    pub(crate) audio_thread: Option<P::AudioThread>,
    pub(crate) descriptor: PluginDescriptor<P>,
    pub(crate) host: Arc<Host>,
    pub(crate) plugin: P,
    pub(crate) plugin_extensions: ClapPluginExtensions<P>,
}

impl<P: Plugin> Runtime<P> {
    pub(crate) fn generate(host: Arc<Host>) -> Self {
        Self {
            descriptor: PluginDescriptor::allocate(),
            plugin: P::default(),
            audio_thread: None,
            host: host.clone(),
            plugin_extensions: ClapPluginExtensions {
                audio_ports: P::Extensions::audio_ports().map(|ext| ClapPluginAudioPorts::new(ext)),
            },
        }
    }

    pub(crate) fn boxed_clap_plugin(self) -> Box<clap_plugin> {
        ffi::box_clap_plugin(self)
    }
}

pub(crate) struct ClapPlugin<P> {
    clap_plugin: NonNull<clap_plugin>,
    _marker: PhantomData<P>,
}

impl<P: Plugin> ClapPlugin<P> {
    pub(crate) const unsafe fn new(clap_plugin: NonNull<clap_plugin>) -> Self {
        Self {
            clap_plugin,
            _marker: PhantomData,
        }
    }

    pub(crate) fn plugin_data(&self) -> &Runtime<P> {
        let data = unsafe { self.clap_plugin.as_ref() }.plugin_data;
        unsafe { &*(data as *const _) }
    }

    pub(crate) fn plugin_data_mut(&mut self) -> &mut Runtime<P> {
        let data = unsafe { self.clap_plugin.as_ref() }.plugin_data;
        unsafe { &mut *(data as *mut _) }
    }

    unsafe fn take(self) -> Runtime<P> {
        let clap_plugin = unsafe { Box::from_raw(self.clap_plugin.as_ptr()) };
        let data: *mut Runtime<P> = clap_plugin.plugin_data as *mut _;

        *unsafe { Box::from_raw(data) }
    }
}

/// Safety:
///
/// 1. The user must assure the pointer to plugin is non-null.
/// 2. The pointer must point to a valid clap_plugin structure tied to the plugin
///    type P, and living in the host.
///
/// Typically, a valid pointer comes from the host calling the plugin's methods.
pub(crate) const unsafe fn wrap_clap_plugin_from_host<P: Plugin>(
    plugin: *const clap_sys::clap_plugin,
) -> ClapPlugin<P> {
    let plugin = plugin as *mut _;
    unsafe { ClapPlugin::<P>::new(NonNull::new_unchecked(plugin)) }
}

pub(crate) use desc::PluginDescriptor;

mod desc;
mod ffi;
