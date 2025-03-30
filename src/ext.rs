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

use std::fmt::{Display, Formatter};

use crate::{
    ext::{
        audio_ports::AudioPorts, latency::Latency, note_ports::NotePorts, params::Params,
        state::State,
    },
    plugin::Plugin,
};

pub mod audio_ports;
pub mod latency;
pub mod log;
pub mod note_ports;
pub mod params;
pub mod state;

/// Plugin extensions.
pub trait Extensions<P: Plugin> {
    fn audio_ports() -> Option<impl AudioPorts<P>> {
        None::<()>
    }

    fn latency() -> Option<impl Latency<P>> {
        None::<()>
    }

    fn note_ports() -> Option<impl NotePorts<P>> {
        None::<()>
    }

    fn params() -> Option<impl Params<P>> {
        None::<()>
    }

    fn state() -> Option<impl State<P>> {
        None::<()>
    }
}

#[derive(Debug)]
pub enum Error {
    Log(log::Error),
    AudioPorts(audio_ports::Error),
    NotePorts(note_ports::Error),
    Params(params::Error),
    State(state::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Log(e) => write!(f, "log: {e}"),
            Error::AudioPorts(e) => write!(f, "audio_ports: {e}"),
            Error::NotePorts(e) => write!(f, "note_ports: {e}"),
            Error::Params(e) => write!(f, "params: {e}"),
            Error::State(e) => write!(f, "state: {e}"),
        }
    }
}

impl From<Error> for crate::Error {
    fn from(value: Error) -> Self {
        Self::Extension(value)
    }
}
