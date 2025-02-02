use crate::{ext::audio_ports::AudioPortInfo, plugin::Plugin};

pub mod audio_ports;

pub trait Extensions<P: Plugin> {
    fn audio_ports() -> Option<impl AudioPorts<P>> {
        None::<()>
    }
}

impl<P: Plugin> Extensions<P> for () {}

pub trait AudioPorts<P>
where
    P: Plugin,
{
    fn inputs(plugin: &P) -> u32;
    fn outputs(plugin: &P) -> u32;

    fn input_info(plugin: &P, index: u32) -> Option<AudioPortInfo>;
    fn output_info(plugin: &P, index: u32) -> Option<AudioPortInfo>;
}
