use std::marker::PhantomData;

use crate::{
    ext::audio_ports::AudioPorts,
    ffi::{clap_audio_port_info, clap_plugin, clap_plugin_audio_ports},
    plugin::{ClapPlugin, Plugin},
};

extern "C-unwind" fn count<A, P>(plugin: *const clap_plugin, is_input: bool) -> u32
where
    P: Plugin,
    A: AudioPorts<P>,
{
    if plugin.is_null() {
        return 0;
    }
    // SAFETY: We just checked that the pointer is non-null and the plugin
    // has been obtained from host and is tied to type P.
    let mut clap_plugin = unsafe { ClapPlugin::<P>::new_unchecked(plugin) };

    // SAFETY: This function is called on the main thread.
    // It is guaranteed that we are the only function accessing the plugin now.
    // So the mutable reference to plugin for the duration of this call is
    // safe.
    let plugin = unsafe { clap_plugin.plugin() };

    A::count(plugin, is_input)
}

extern "C-unwind" fn get<A, P>(
    plugin: *const clap_plugin,
    index: u32,
    is_input: bool,
    info: *mut clap_audio_port_info,
) -> bool
where
    P: Plugin,
    A: AudioPorts<P>,
{
    if plugin.is_null() {
        return false;
    }
    // SAFETY: We just checked that the pointer is non-null and the plugin
    // has been obtained from host and is tied to type P.
    let mut clap_plugin = unsafe { ClapPlugin::<P>::new_unchecked(plugin) };

    // SAFETY: This function is called on the main thread.
    // It is guaranteed that we are the only function accessing the plugin now.
    // So the mutable reference to plugin for the duration of this call is
    // safe.
    let plugin = unsafe { clap_plugin.plugin() };

    // SAFETY: The host guarantees we are the only function that can access info
    // for the duration of the function call.  So obtaining a mutable reference
    // is safe.
    let info = unsafe { &mut *info };

    A::get(plugin, index, is_input)
        .map(|x| x.fill_clap_audio_port_info(info))
        .is_some()
}

pub struct ClapPluginAudioPorts<P> {
    #[allow(unused)]
    clap_plugin_audio_ports: clap_plugin_audio_ports,
    _marker: PhantomData<P>,
}

impl<P: Plugin> ClapPluginAudioPorts<P> {
    pub fn new<A: AudioPorts<P>>(_: A) -> Self {
        Self {
            clap_plugin_audio_ports: clap_plugin_audio_ports {
                count: Some(count::<A, P>),
                get: Some(get::<A, P>),
            },
            _marker: PhantomData,
        }
    }
}
