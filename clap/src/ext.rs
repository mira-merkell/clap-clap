use crate::ext::audio_ports::AudioPortInfo;
use crate::plugin::Plugin;

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
    fn inputs(plugin: &P) -> usize;
    fn outputs(plugin: &P) -> usize;

    fn input_info(plugin: &P, index: usize) -> Option<AudioPortInfo>;
    fn output_info(plugin: &P, index: usize) -> Option<AudioPortInfo>;
}

pub mod audio_ports;
