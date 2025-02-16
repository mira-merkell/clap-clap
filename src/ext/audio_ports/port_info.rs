use std::ptr::null;

use crate::{
    ffi::{
        CLAP_AUDIO_PORT_IS_MAIN, CLAP_AUDIO_PORT_PREFERS_64BITS,
        CLAP_AUDIO_PORT_REQUIRES_COMMON_SAMPLE_SIZE, CLAP_AUDIO_PORT_SUPPORTS_64BITS,
        CLAP_INVALID_ID, CLAP_PORT_AMBISONIC, CLAP_PORT_MONO, CLAP_PORT_STEREO, CLAP_PORT_SURROUND,
        clap_audio_port_info,
    },
    id::ClapId,
    impl_flags_u32,
    plugin::Plugin,
    prelude::AudioPorts,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u32)]
pub enum AudioPortFlags {
    /// This port is the main audio input or output. There can be only one main
    /// input and main output. Main port must be at index 0.
    IsMain = CLAP_AUDIO_PORT_IS_MAIN,
    /// This port can be used with 64 bits audio
    Supports64bits = CLAP_AUDIO_PORT_SUPPORTS_64BITS,
    /// 64 bits audio is preferred with this port
    Prefers64bits = CLAP_AUDIO_PORT_PREFERS_64BITS,
    /// This port must be used with the same sample size as all the other ports
    /// which have this flag. In other words if all ports have this flag then
    /// the plugin may either be used entirely with 64 bits audio or 32 bits
    /// audio, but it can't be mixed.
    RequiresCommonSampleSize = CLAP_AUDIO_PORT_REQUIRES_COMMON_SAMPLE_SIZE,
}

impl_flags_u32!(AudioPortFlags);

#[derive(Debug, Default, Clone, PartialEq)]
pub struct AudioPortInfo {
    pub id: ClapId,
    pub name: Option<String>,
    pub flags: u32,
    pub channel_count: u32,
    pub port_type: Option<AudioPortType>,
    pub in_place_pair: Option<ClapId>,
}

impl AudioPortInfo {
    pub fn builder() -> AudioPortInfoBuilder {
        AudioPortInfoBuilder::new(Self::default())
    }

    pub(super) fn fill_clap_audio_port_info(&self, info: &mut clap_audio_port_info) {
        info.id = self.id.into();

        if let Some(name) = &self.name {
            let n = name.len().min(info.name.len());
            unsafe {
                std::ptr::copy_nonoverlapping(name.as_ptr(), info.name.as_mut_ptr() as *mut _, n)
            }
            info.name[n] = b'\0' as _;
        } else {
            info.name[0] = b'\0' as _;
        };

        info.flags = self.flags;

        info.channel_count = self.channel_count;

        info.port_type = match self.port_type {
            Some(AudioPortType::Mono) => CLAP_PORT_MONO.as_ptr(),
            Some(AudioPortType::Stereo) => CLAP_PORT_STEREO.as_ptr(),
            Some(AudioPortType::Surround) => CLAP_PORT_SURROUND.as_ptr(),
            Some(AudioPortType::Ambisonic) => CLAP_PORT_AMBISONIC.as_ptr(),
            None => null(),
        };

        info.in_place_pair = self
            .in_place_pair
            .map(Into::into)
            .unwrap_or(CLAP_INVALID_ID);
    }
}

pub struct AudioPortInfoBuilder {
    info: AudioPortInfo,
}

impl AudioPortInfoBuilder {
    const fn new(info: AudioPortInfo) -> Self {
        Self { info }
    }

    pub const fn id(&mut self, id: ClapId) -> &mut Self {
        self.info.id = id;
        self
    }

    pub fn name(&mut self, name: &str) -> &mut Self {
        self.info.name = Some(name.to_string());
        self
    }

    pub const fn port_is_main(&mut self) -> &mut Self {
        self.info.flags |= CLAP_AUDIO_PORT_IS_MAIN;
        self
    }

    pub const fn prefers_64_bits(&mut self) -> &mut Self {
        self.info.flags |= CLAP_AUDIO_PORT_PREFERS_64BITS;
        self
    }

    pub const fn supports_64_bits(&mut self) -> &mut Self {
        self.info.flags |= CLAP_AUDIO_PORT_SUPPORTS_64BITS;
        self
    }

    pub const fn requires_common_sample_size(&mut self) -> &mut Self {
        self.info.flags |= CLAP_AUDIO_PORT_REQUIRES_COMMON_SAMPLE_SIZE;
        self
    }

    pub fn channel_count(&mut self, n: u32) -> &mut Self {
        self.info.channel_count = n;
        self
    }

    pub const fn port_type(&mut self, port_type: AudioPortType) -> &mut Self {
        self.info.port_type = Some(port_type);
        self
    }

    pub const fn in_place_pair(&mut self, id: ClapId) -> &mut Self {
        self.info.in_place_pair = Some(id);
        self
    }

    pub fn build(&self) -> AudioPortInfo {
        self.info.clone()
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum AudioPortType {
    Mono,
    Stereo,
    Surround,
    Ambisonic,
}

impl TryFrom<&str> for AudioPortType {
    type Error = crate::ext::audio_ports::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value == "mono" {
            Ok(AudioPortType::Mono)
        } else if value == "stereo" {
            Ok(AudioPortType::Stereo)
        } else if value == "ambisonic" {
            Ok(AudioPortType::Ambisonic)
        } else if value == "surround" {
            Ok(AudioPortType::Surround)
        } else {
            Err(Self::Error::PortType)
        }
    }
}

impl<P: Plugin> AudioPorts<P> for () {
    fn count(_: &P, _: bool) -> u32 {
        0
    }

    fn get(_: &P, _: u32, _: bool) -> Option<AudioPortInfo> {
        None
    }
}
