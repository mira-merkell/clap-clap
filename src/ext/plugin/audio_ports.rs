use std::{marker::PhantomData, ptr::null};

use crate::{
    ext::plugin::audio_ports::ffi::clap_plugin_audio_ports,
    ffi::{
        CLAP_AUDIO_PORT_IS_MAIN, CLAP_AUDIO_PORT_PREFERS_64BITS,
        CLAP_AUDIO_PORT_REQUIRES_COMMON_SAMPLE_SIZE, CLAP_AUDIO_PORT_SUPPORTS_64BITS,
        CLAP_INVALID_ID, CLAP_PORT_AMBISONIC, CLAP_PORT_MONO, CLAP_PORT_STEREO, CLAP_PORT_SURROUND,
        clap_audio_port_info, clap_plugin_audio_ports,
    },
    id::ClapId,
    plugin::Plugin,
};

pub trait AudioPorts<P>
where
    P: Plugin,
{
    fn count(plugin: &P, is_input: bool) -> u32;
    fn get(plugin: &P, index: u32, is_input: bool) -> Option<AudioPortInfo>;
}

mod ffi;

#[derive(Debug, Default, Clone)]
pub struct AudioPortInfo {
    id: ClapId,
    name: Option<String>,
    flags: u32,
    channel_count: Option<u32>,
    port_type: Option<AudioPortType>,
    in_place_pair: Option<ClapId>,
}

impl AudioPortInfo {
    pub fn builder() -> AudioPortInfoBuilder {
        AudioPortInfoBuilder::new(Self::default())
    }

    fn fill_clap_audio_port_info(&self, info: &mut clap_audio_port_info) {
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

        info.channel_count = self.channel_count.unwrap_or(0);

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
        self.info.channel_count = Some(n);
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

#[derive(Debug, Copy, Clone)]
pub enum AudioPortType {
    Mono,
    Stereo,
    Surround,
    Ambisonic,
}

impl<P: Plugin> AudioPorts<P> for () {
    fn count(_: &P, _: bool) -> u32 {
        0
    }

    fn get(_: &P, _: u32, _: bool) -> Option<AudioPortInfo> {
        None
    }
}

/// Static mono ports, in and out.
#[derive(Debug, Copy, Clone)]
pub struct MonoPorts<const IN: u32, const OUT: u32>;

impl<P, const IN: u32, const OUT: u32> AudioPorts<P> for MonoPorts<IN, OUT>
where
    P: Plugin,
{
    fn count(_: &P, is_input: bool) -> u32 {
        if is_input { IN } else { OUT }
    }

    fn get(_: &P, index: u32, is_input: bool) -> Option<AudioPortInfo> {
        if is_input {
            (index < IN).then_some(if index == 0 {
                AudioPortInfo::builder()
                    .id(index.try_into().unwrap())
                    .name("Main In")
                    .port_is_main()
                    .channel_count(1)
                    .port_type(AudioPortType::Mono)
                    .build()
            } else {
                AudioPortInfo::builder()
                    .id(index.try_into().unwrap())
                    .name(format!("In {index}").as_str())
                    .channel_count(1)
                    .port_type(AudioPortType::Mono)
                    .build()
            })
        } else {
            (index < OUT).then_some(if index == 0 {
                AudioPortInfo::builder()
                    .id((IN + index).try_into().unwrap())
                    .name("Main Out")
                    .port_is_main()
                    .channel_count(1)
                    .port_type(AudioPortType::Mono)
                    .build()
            } else {
                AudioPortInfo::builder()
                    .id((IN + index).try_into().unwrap())
                    .name(format!("Out {index}").as_str())
                    .channel_count(1)
                    .port_type(AudioPortType::Mono)
                    .build()
            })
        }
    }
}

/// Single static stereo port, in and out.
#[derive(Debug, Copy, Clone)]
pub struct StereoPorts<const IN: u32, const OUT: u32>;

impl<P, const IN: u32, const OUT: u32> AudioPorts<P> for StereoPorts<IN, OUT>
where
    P: Plugin,
{
    fn count(_: &P, is_input: bool) -> u32 {
        if is_input { IN } else { OUT }
    }

    fn get(_: &P, index: u32, is_input: bool) -> Option<AudioPortInfo> {
        if is_input {
            (index < IN).then_some(if index == 0 {
                AudioPortInfo::builder()
                    .id(index.try_into().unwrap())
                    .name("Main In")
                    .port_is_main()
                    .channel_count(2)
                    .port_type(AudioPortType::Stereo)
                    .build()
            } else {
                AudioPortInfo::builder()
                    .id(index.try_into().unwrap())
                    .name(format!("In {index}").as_str())
                    .channel_count(2)
                    .port_type(AudioPortType::Stereo)
                    .build()
            })
        } else {
            (index < OUT).then_some(if index == 0 {
                AudioPortInfo::builder()
                    .id((IN + index).try_into().unwrap())
                    .name("Main Out")
                    .port_is_main()
                    .channel_count(2)
                    .port_type(AudioPortType::Stereo)
                    .build()
            } else {
                AudioPortInfo::builder()
                    .id((IN + index).try_into().unwrap())
                    .name(format!("Out {index}").as_str())
                    .channel_count(2)
                    .port_type(AudioPortType::Stereo)
                    .build()
            })
        }
    }
}

pub(crate) struct ClapPluginAudioPorts<P> {
    #[allow(unused)]
    clap_plugin_audio_ports: clap_plugin_audio_ports,
    _marker: PhantomData<P>,
}

impl<P: Plugin> ClapPluginAudioPorts<P> {
    pub(crate) fn new<A: AudioPorts<P>>(_: A) -> Self {
        Self {
            clap_plugin_audio_ports: clap_plugin_audio_ports::<A, P>(),
            _marker: PhantomData,
        }
    }
}
