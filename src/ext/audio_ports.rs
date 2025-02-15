use crate::plugin::Plugin;

mod ffi;
pub(crate) use ffi::ClapPluginAudioPorts;

mod static_ports;
pub use static_ports::{MonoPorts, StereoPorts};

mod port_info;
pub use port_info::{AudioPortInfo, AudioPortInfoBuilder, AudioPortType};

pub trait AudioPorts<P>
where
    P: Plugin,
{
    fn count(plugin: &P, is_input: bool) -> u32;
    fn get(plugin: &P, index: u32, is_input: bool) -> Option<AudioPortInfo>;
}
