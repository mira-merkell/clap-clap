use crate::{
    ext::AudioPorts, ext::audio_ports::ffi::clap_plugin_audio_ports, id::ClapId, plugin::Plugin,
};
use clap_sys::{
    CLAP_AUDIO_PORT_IS_MAIN, CLAP_AUDIO_PORT_PREFERS_64BITS,
    CLAP_AUDIO_PORT_REQUIRES_COMMON_SAMPLE_SIZE, CLAP_AUDIO_PORT_SUPPORTS_64BITS, CLAP_INVALID_ID,
    CLAP_PORT_AMBISONIC, CLAP_PORT_MONO, CLAP_PORT_STEREO, CLAP_PORT_SURROUND,
    clap_audio_port_info, clap_plugin_audio_ports,
};
use std::{marker::PhantomData, ptr::null};

#[derive(Debug, Default, Clone)]
pub struct AudioPortInfo {
    id: ClapId,
    name: Option<String>,
    flags: u32,
    channel_count: Option<u32>,
    port_type: Option<AudioPortType>,
    in_place_pair: Option<u32>,
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

        info.in_place_pair = self.in_place_pair.unwrap_or(CLAP_INVALID_ID);
    }
}

pub struct AudioPortInfoBuilder {
    info: AudioPortInfo,
}

impl AudioPortInfoBuilder {
    fn new(info: AudioPortInfo) -> Self {
        Self { info }
    }

    pub fn id(&mut self, id: usize) -> Option<&mut Self> {
        self.info.id = id.try_into().ok()?;
        Some(self)
    }

    pub fn name(&mut self, name: &str) -> &mut Self {
        self.info.name = Some(name.to_string());
        self
    }

    pub fn port_is_main(&mut self) -> &mut Self {
        self.info.flags |= CLAP_AUDIO_PORT_IS_MAIN;
        self
    }

    pub fn prefers_64_bits(&mut self) -> &mut Self {
        self.info.flags |= CLAP_AUDIO_PORT_PREFERS_64BITS;
        self
    }

    pub fn supports_64_bits(&mut self) -> &mut Self {
        self.info.flags |= CLAP_AUDIO_PORT_SUPPORTS_64BITS;
        self
    }

    pub fn requires_common_sample_size(&mut self) -> &mut Self {
        self.info.flags |= CLAP_AUDIO_PORT_REQUIRES_COMMON_SAMPLE_SIZE;
        self
    }

    pub fn channel_count(&mut self, n: usize) -> Option<&mut Self> {
        u32::try_from(n).ok().map(|n| {
            self.info.channel_count = Some(n);
            self
        })
    }

    pub fn port_type(&mut self, port_type: AudioPortType) -> &mut Self {
        self.info.port_type = Some(port_type);
        self
    }

    pub fn in_place_pair(&mut self, id: u32) -> &mut Self {
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
    fn inputs(_plugin: &P) -> usize {
        0
    }

    fn outputs(_plugin: &P) -> usize {
        0
    }

    fn input_info(_plugin: &P, _index: usize) -> Option<AudioPortInfo> {
        None
    }

    fn output_info(_plugin: &P, _index: usize) -> Option<AudioPortInfo> {
        None
    }
}

/// Static mono ports, in and out.
#[derive(Debug, Copy, Clone)]
pub struct MonoPorts<const IN: usize, const OUT: usize>;

impl<P, const IN: usize, const OUT: usize> AudioPorts<P> for MonoPorts<IN, OUT>
where
    P: Plugin,
{
    fn inputs(_: &P) -> usize {
        IN
    }

    fn outputs(_: &P) -> usize {
        OUT
    }

    fn input_info(_: &P, index: usize) -> Option<AudioPortInfo> {
        (index < IN).then_some(if index == 0 {
            AudioPortInfo::builder()
                .id(index)?
                .name("Main In")
                .port_is_main()
                .channel_count(1)?
                .port_type(AudioPortType::Mono)
                .build()
        } else {
            AudioPortInfo::builder()
                .id(index)?
                .name(format!("In {index}").as_str())
                .channel_count(1)?
                .port_type(AudioPortType::Mono)
                .build()
        })
    }

    fn output_info(_: &P, index: usize) -> Option<AudioPortInfo> {
        (index < OUT).then_some(if index == 0 {
            AudioPortInfo::builder()
                .id(index)?
                .name("Main Out")
                .port_is_main()
                .channel_count(1)?
                .port_type(AudioPortType::Mono)
                .build()
        } else {
            AudioPortInfo::builder()
                .id(index)?
                .name(format!("Out {index}").as_str())
                .channel_count(1)?
                .port_type(AudioPortType::Mono)
                .build()
        })
    }
}

/// Single static stereo port, in and out.
#[derive(Debug, Copy, Clone)]
pub struct StereoPorts<const IN: usize, const OUT: usize>;

impl<P, const IN: usize, const OUT: usize> AudioPorts<P> for StereoPorts<IN, OUT>
where
    P: Plugin,
{
    fn inputs(_: &P) -> usize {
        IN
    }

    fn outputs(_: &P) -> usize {
        OUT
    }

    fn input_info(_: &P, index: usize) -> Option<AudioPortInfo> {
        (index < IN).then_some(if index == 0 {
            AudioPortInfo::builder()
                .id(index)?
                .name("Main In")
                .port_is_main()
                .channel_count(2)?
                .port_type(AudioPortType::Stereo)
                .build()
        } else {
            AudioPortInfo::builder()
                .id(index)?
                .name(format!("In {index}").as_str())
                .channel_count(2)?
                .port_type(AudioPortType::Stereo)
                .build()
        })
    }

    fn output_info(_: &P, index: usize) -> Option<AudioPortInfo> {
        (index < OUT).then_some(if index == 0 {
            AudioPortInfo::builder()
                .id(index)?
                .name("Main Out")
                .port_is_main()
                .channel_count(2)?
                .port_type(AudioPortType::Stereo)
                .build()
        } else {
            AudioPortInfo::builder()
                .id(index)?
                .name(format!("Out {index}").as_str())
                .channel_count(2)?
                .port_type(AudioPortType::Stereo)
                .build()
        })
    }
}

pub(crate) struct ClapPluginAudioPorts<P> {
    pub(crate) raw: clap_plugin_audio_ports,
    _marker: PhantomData<P>,
}

impl<P: Plugin> ClapPluginAudioPorts<P> {
    pub(crate) fn new<A: AudioPorts<P>>(_ports: A) -> Self {
        Self {
            raw: clap_plugin_audio_ports::<A, P>(),
            _marker: PhantomData,
        }
    }
}

mod ffi;
