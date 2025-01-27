use crate::ext::AudioPorts;
use crate::plugin::{Plugin, wrap_clap_plugin_from_host};
use clap_sys::{clap_audio_port_info, clap_plugin, clap_plugin_audio_ports};

extern "C" fn count<A, P>(plugin: *const clap_plugin, is_input: bool) -> u32
where
    P: Plugin,
    A: AudioPorts<P>,
{
    let wrapper = unsafe { wrap_clap_plugin_from_host::<P>(plugin) };
    let plugin = &wrapper.clap_plugin().plugin;
    if is_input {
        A::inputs(plugin) as u32
    } else {
        A::outputs(plugin) as u32
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
    let wrapper = unsafe { wrap_clap_plugin_from_host::<P>(plugin) };
    let plugin = &wrapper.clap_plugin().plugin;
    let index = index.try_into().expect("index must fit into usize");
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
