//! A CLAP plugin runtime. ⧉⧉⧉

pub mod audio_buffer;
#[doc(hidden)]
pub mod entry;
pub mod events;
pub mod ext;
#[doc(hidden)]
pub mod factory;
#[doc(hidden)]
pub mod ffi;
pub mod fixedpoint;
pub mod host;
pub mod id;
pub mod plugin;
pub mod plugin_features;
pub mod process;
pub mod stream;
pub mod string_sizes;
pub mod timestamp;
pub mod version;

pub mod prelude {
    #[doc(inline)]
    pub use crate::{
        Error, entry,
        events::{self, Event, EventBuilder, InputEvents, OutputEvents},
        ext::{
            self, Extensions,
            audio_ports::{
                self, AudioPortFlags, AudioPortInfo, AudioPortType, AudioPorts, MonoPorts,
                StereoPorts,
            },
            latency::{self, HostLatency, Latency},
            log::{self, Severity},
            note_ports::{self, NoteDialect, NotePortInfo, NotePorts},
            params::{self, ParamInfo, Params},
            state::{self, State},
        },
        host::{self, Host},
        id::ClapId,
        plugin::{self, AudioThread, Plugin},
        plugin_features,
        process::{self, Process, Status, Status::Continue},
        stream::{self, IStream, OStream},
    };
}

#[doc(hidden)]
#[macro_export]
// The type Flags must be #[repr(u32)].
macro_rules! impl_flags_u32 {
    ($($Flags:ty),* $(,)?) => {$(
        impl $Flags {
            pub const fn set(&self, flags: u32) -> u32 {
                *self as u32 | flags
            }

            pub const fn is_set(&self, flags: u32) -> bool {
                *self as u32 & flags != 0
            }

            pub const fn clear(&self, flags: u32) -> u32 {
                !(*self as u32) & flags
            }
        }

        impl From<$Flags> for u32 {
            fn from(value: $Flags) -> Self {
                value as u32
            }
        }
    )*};
}

#[derive(Debug)]
pub enum Error {
    Events(events::Error),
    Extension(ext::Error),
    Factory(factory::Error),
    Host(host::Error),
    Id(id::Error),
    IO(std::io::Error),
    Plugin(plugin::Error),
    User(Box<dyn std::error::Error + Send + 'static>),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Error::*;
        match self {
            Events(e) => write!(f, "events:  {e}"),
            Extension(e) => write!(f, "extension:  {e}"),
            Factory(e) => write!(f, "factory: {e}"),
            Host(e) => write!(f, "host: {e}"),
            Id(e) => write!(f, "id : {e}"),
            IO(e) => write!(f, "I/O: {e}"),
            Plugin(e) => write!(f, "plugin: {e}"),
            User(e) => write!(f, "user: {e}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::IO(value)
    }
}

impl From<std::num::ParseFloatError> for Error {
    fn from(value: std::num::ParseFloatError) -> Self {
        crate::ext::params::Error::from(value).into()
    }
}
