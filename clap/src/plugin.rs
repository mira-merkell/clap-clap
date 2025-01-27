use crate::process::Process;
use crate::{
    ext::audio_ports::ClapPluginAudioPorts, host::ClapHost, plugin,
    process,
};
use clap_sys::{clap_plugin, clap_plugin_descriptor, CLAP_VERSION};
use std::fmt::Display;
use std::{ffi::c_char, ffi::CString, marker::PhantomData, ptr::null, ptr::NonNull, str::FromStr};
use crate::ext::Extensions;

#[derive(Debug, Copy, Clone)]
pub enum Error {}

impl Display for Error {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        Ok(())
    }
}

impl std::error::Error for Error {}

pub trait Plugin: Default + Sync + Send {
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

    type Extensions: Extensions<Self>;

    #[allow(unused_variables)]
    fn activate(
        &mut self,
        sample_rate: f64,
        min_frames: usize,
        max_frames: usize,
    ) -> Result<(), plugin::Error> {
        Ok(())
    }

    fn deactivate(&mut self) {}

    fn start_processing(&mut self) -> Result<(), process::Error> {
        Ok(())
    }

    fn stop_processing(&mut self) {}

    fn process(&mut self, process: &mut Process) -> Result<process::Status, process::Error>;

    fn reset(&mut self) {}

    fn on_main_thread(&self) {}
}

#[allow(warnings, unused)]
pub struct PluginDescriptor<P> {
    pub(crate) id: CString,
    name: CString,
    vendor: CString,
    url: CString,
    manual_url: CString,
    support_url: CString,
    version: CString,
    description: CString,
    features: Box<[CString]>,

    raw_features: Box<[*const c_char]>,
    pub(crate) raw_descriptor: clap_plugin_descriptor,
    _marker: PhantomData<P>,
}

impl<P: Plugin> PluginDescriptor<P> {
    pub fn allocate() -> Self {
        let id = CString::from_str(P::ID).unwrap();
        let name = CString::from_str(P::NAME).unwrap();
        let vendor = CString::from_str(P::VENDOR).unwrap();
        let url = CString::from_str(P::URL).unwrap();
        let manual_url = CString::from_str(P::MANUAL_URL).unwrap();
        let support_url = CString::from_str(P::SUPPORT_URL).unwrap();
        let version = CString::from_str(P::VERSION).unwrap();
        let description = CString::from_str(P::DESCRIPTION).unwrap();

        let features: Vec<_> = String::from_str(P::FEATURES)
            .unwrap()
            .split_whitespace()
            .map(|s| CString::from_str(s).unwrap())
            .collect();
        let mut features_raw: Vec<_> = features.iter().map(|f| f.as_c_str().as_ptr()).collect();
        features_raw.push(null());
        let features_raw = features_raw.into_boxed_slice();

        let raw = clap_plugin_descriptor {
            clap_version: CLAP_VERSION,
            id: id.as_c_str().as_ptr(),
            name: name.as_c_str().as_ptr(),
            vendor: vendor.as_c_str().as_ptr(),
            url: url.as_c_str().as_ptr(),
            manual_url: manual_url.as_c_str().as_ptr(),
            support_url: support_url.as_c_str().as_ptr(),
            version: version.as_c_str().as_ptr(),
            description: description.as_c_str().as_ptr(),
            features: features_raw.as_ptr(),
        };

        Self {
            id,
            name,
            vendor,
            url,
            manual_url,
            support_url,
            version,
            description,
            features: features.into(),
            raw_features: features_raw,
            raw_descriptor: raw,
            _marker: PhantomData,
        }
    }
}

pub(crate) struct ClapPluginExtensions<P> {
    pub(crate) audio_ports: Option<ClapPluginAudioPorts<P>>,
}

pub(crate) struct ClapPlugin<P> {
    pub(crate) descriptor: PluginDescriptor<P>,
    pub(crate) _host: ClapHost,
    pub(crate) plugin: P,
    pub(crate) plugin_extensions: ClapPluginExtensions<P>,
}

impl<P: Plugin> ClapPlugin<P> {
    pub(crate) fn generate(plugin: P, host: ClapHost) -> Self {
        let audio_ports = P::Extensions::audio_ports().map(|ap| ClapPluginAudioPorts::new(ap));

        Self {
            descriptor: PluginDescriptor::allocate(),
            plugin,
            _host: host,
            plugin_extensions: ClapPluginExtensions { audio_ports },
        }
    }

    pub(crate) fn boxed_clap_plugin(self) -> Box<clap_plugin> {
        ffi::box_clap_plugin(self)
    }
}

pub(crate) struct ClapPluginWrapper<P> {
    clap_plugin: NonNull<clap_plugin>,
    _marker: PhantomData<P>,
}

impl<P: Plugin> ClapPluginWrapper<P> {
    pub(crate) const unsafe fn new(clap_plugin: NonNull<clap_plugin>) -> Self {
        Self {
            clap_plugin,
            _marker: PhantomData,
        }
    }

    pub(crate) fn clap_plugin(&self) -> &ClapPlugin<P> {
        let data = unsafe { self.clap_plugin.as_ref() }.plugin_data;
        unsafe { &*(data as *const _) }
    }

    pub(crate) fn clap_plugin_mut(&mut self) -> &mut ClapPlugin<P> {
        let data = unsafe { self.clap_plugin.as_ref() }.plugin_data;
        unsafe { &mut *(data as *mut _) }
    }

    unsafe fn take(self) -> ClapPlugin<P> {
        let clap_plugin = unsafe { Box::from_raw(self.clap_plugin.as_ptr()) };
        let data: *mut ClapPlugin<P> = clap_plugin.plugin_data as *mut _;

        *unsafe { Box::from_raw(data) }
    }
}

mod ffi;

pub(crate) const unsafe fn wrap_clap_plugin_from_host<P: Plugin>(
    plugin: *const clap_sys::clap_plugin,
) -> ClapPluginWrapper<P> {
    let plugin = plugin as *mut _;
    unsafe { ClapPluginWrapper::<P>::new(std::ptr::NonNull::new_unchecked(plugin)) }
}