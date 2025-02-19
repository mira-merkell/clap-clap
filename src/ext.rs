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
pub mod params;

use std::fmt::{Display, Formatter};

use crate::{
    ext::{audio_ports::AudioPorts, params::Params},
    plugin::Plugin,
};

/// Plugin extensions.
pub trait Extensions<P: Plugin> {
    fn audio_ports() -> Option<impl AudioPorts<P>> {
        None::<()>
    }

    fn params() -> Option<impl Params<P>> {
        None::<()>
    }
}

#[derive(Debug)]
pub enum Error {
    Log(log::Error),
    AudioPorts(audio_ports::Error),
    Params(params::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Log(e) => write!(f, "log: {e}"),
            Error::AudioPorts(e) => write!(f, "audio_ports: {e}"),
            Error::Params(e) => write!(f, "params: {e}"),
        }
    }
}

impl From<Error> for crate::Error {
    fn from(value: Error) -> Self {
        Self::Extension(value)
    }
}
