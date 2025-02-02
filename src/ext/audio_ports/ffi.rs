use clap_sys::{clap_audio_port_info, clap_plugin, clap_plugin_audio_ports};

use crate::{
    ext::AudioPorts,
    plugin::{ClapPlugin, Plugin},
};

extern "C" fn count<A, P>(plugin: *const clap_plugin, is_input: bool) -> u32
where
    P: Plugin,
    A: AudioPorts<P>,
{
    if plugin.is_null() {
        return 0;
    }
    // SAFETY: We just checked that the pointer is non-null and the plugin
    // has been obtained from host and is tied to type P.
    let mut clap_plugin = unsafe { ClapPlugin::<P>::new(plugin) };

    // SAFETY: This function is called on the main thread.
    // It is guaranteed that we are the only function accessing the plugin now.
    // So the mutable reference to plugin for the duration of this call is
    // safe.
    let plugin = unsafe { clap_plugin.plugin() };

    if is_input {
        A::inputs(plugin)
    } else {
        A::outputs(plugin)
    }
}

extern "C" fn get<A, P>(
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
    let mut clap_plugin = unsafe { ClapPlugin::<P>::new(plugin) };

    // SAFETY: This function is called on the main thread.
    // It is guaranteed that we are the only function accessing the plugin now.
    // So the mutable reference to plugin for the duration of this call is
    // safe.
    let plugin = unsafe { clap_plugin.plugin() };

    // SAFETY: The host guarantees we are the only function that can access info
    // for the duration of the function call.  So obtaining a mutable reference
    // is safe.
    let info = unsafe { &mut *info };

    is_input
        .then(|| A::input_info(plugin, index))
        .flatten()
        .or_else(|| A::output_info(plugin, index))
        .map(|q| q.fill_clap_audio_port_info(info))
        .is_some()
}

pub(crate) const fn clap_plugin_audio_ports<A, P>() -> clap_plugin_audio_ports
where
    P: Plugin,
    A: AudioPorts<P>,
{
    clap_plugin_audio_ports {
        count: Some(count::<A, P>),
        get: Some(get::<A, P>),
    }
}
