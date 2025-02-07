//! Plugin extensions.

use crate::{ext::plugin::audio_ports::AudioPorts, plugin::Plugin};

pub mod audio_ports;

/// Plugin extensions.
pub trait Extensions<P: Plugin> {
    fn audio_ports() -> Option<impl AudioPorts<P>> {
        None::<()>
    }
}

impl<P: Plugin> Extensions<P> for () {}
