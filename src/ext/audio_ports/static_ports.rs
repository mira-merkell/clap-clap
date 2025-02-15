use crate::{
    ext::audio_ports::{AudioPortInfo, AudioPortType},
    plugin::Plugin,
    prelude::AudioPorts,
};

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
