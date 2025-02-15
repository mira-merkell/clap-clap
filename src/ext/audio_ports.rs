use crate::{ffi::clap_host_audio_ports, impl_flags_u32, plugin::Plugin, prelude::Host};

mod ffi;
pub(crate) use ffi::ClapPluginAudioPorts;

mod static_ports;
pub use static_ports::{MonoPorts, StereoPorts};

mod port_info;
pub use port_info::{AudioPortFlags, AudioPortInfo, AudioPortInfoBuilder, AudioPortType};

use crate::ffi::{
    CLAP_AUDIO_PORTS_RESCAN_CHANNEL_COUNT, CLAP_AUDIO_PORTS_RESCAN_FLAGS,
    CLAP_AUDIO_PORTS_RESCAN_IN_PLACE_PAIR, CLAP_AUDIO_PORTS_RESCAN_LIST,
    CLAP_AUDIO_PORTS_RESCAN_NAMES, CLAP_AUDIO_PORTS_RESCAN_PORT_TYPE,
};

pub trait AudioPorts<P>
where
    P: Plugin,
{
    fn count(plugin: &P, is_input: bool) -> u32;
    fn get(plugin: &P, index: u32, is_input: bool) -> Option<AudioPortInfo>;
}

/// Port rescan flags.
///
/// # Example
///
/// ```rust
/// # use clap_clap::ext::audio_ports::RescanFlags;
/// assert_eq!(RescanFlags::Names as u32, 0b1);
/// assert!(RescanFlags::Names.is_set(0b101));
/// assert_eq!(RescanFlags::Names.set(0b100), 0b101);
/// assert_eq!(RescanFlags::Names.clear(0b101), 0b100);
/// ```
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u32)]
pub enum RescanFlags {
    /// The ports name did change, the host can scan them right away.
    Names = CLAP_AUDIO_PORTS_RESCAN_NAMES,
    ///  The flags did change
    Flags = CLAP_AUDIO_PORTS_RESCAN_FLAGS,
    ///  The channel_count did change
    ChannelCount = CLAP_AUDIO_PORTS_RESCAN_CHANNEL_COUNT,
    ///  The port type did change
    PortType = CLAP_AUDIO_PORTS_RESCAN_PORT_TYPE,
    ///  The in-place pair did change
    InPlacePair = CLAP_AUDIO_PORTS_RESCAN_IN_PLACE_PAIR,
    ///  The list of ports have changed: entries have been removed/added.
    List = CLAP_AUDIO_PORTS_RESCAN_LIST,
}

impl_flags_u32!(RescanFlags);

pub struct HostAudioPorts<'a> {
    host: &'a Host,
    clap_host_audio_ports: &'a clap_host_audio_ports,
}

impl<'a> HostAudioPorts<'a> {
    /// # Safety
    ///
    /// All extension interface function pointers must be non-null (Some), and
    /// the functions must be thread-safe.
    pub(crate) const unsafe fn new_unchecked(
        host: &'a Host,
        clap_host_audio_ports: &'a clap_host_audio_ports,
    ) -> Self {
        Self {
            host,
            clap_host_audio_ports,
        }
    }

    pub fn is_rescan_flag_supported(&self, flag: RescanFlags) -> bool {
        // SAFETY: By construction, the callback must be a valid function pointer,
        // and the call is thread-safe.
        let callback = self.clap_host_audio_ports.is_rescan_flag_supported.unwrap();
        unsafe { callback(self.host.clap_host(), flag as u32) }
    }

    pub fn rescan(&self, flags: u32) {
        // SAFETY: By construction, the callback must be a valid function pointer,
        // and the call is thread-safe.
        let callback = self.clap_host_audio_ports.rescan.unwrap();
        unsafe { callback(self.host.clap_host(), flags) };
    }
}
