//! CLAP Extensions.
//!
//! The [CLAP API] defines the interface between plugins and hosts as similarly
//! structured C interfaces. This library adopts the plugin's perspective,
//! meaning that host extensions can be implemented as concrete types that the
//! plugin can use by querying: [`Host::get_extension()']. Plugin extensions, on
//! the other hand, are to be specified by the user as trait implementations.
//! The traits describing plugin extensions are declared in this module.
//!
//! You can also find here some concrete implementations provided as a
//! convenience -- e.g., [`StereoPorts`]  defines a static stereo port layout.
//!
//! [CLAP API]: https://github.com/free-audio/clap/tree/main/include/clap
//! [`Host::get_extension()']: crate::host::Host::get_extension
//! [`StereoPorts`]: audio_ports::StereoPorts

pub mod audio_ports;
pub mod log;

use crate::{ext::audio_ports::AudioPorts, plugin::Plugin};

/// Plugin extensions.
pub trait Extensions<P: Plugin> {
    fn audio_ports() -> Option<impl AudioPorts<P>> {
        None::<()>
    }
}

impl<P: Plugin> Extensions<P> for () {}
