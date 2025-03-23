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
pub mod string_sizes;
pub mod timestamp;
pub mod version;

pub mod prelude {
    #[doc(inline)]
    pub use crate::{
        Error, entry,
        events::{self, InputEvents, OutputEvents},
        ext::{
            self, Extensions,
            audio_ports::{
                self, AudioPortFlags, AudioPortInfo, AudioPortType, AudioPorts, HostAudioPorts,
                MonoPorts, StereoPorts,
            },
            log::{self, HostLog, Severity},
            note_ports::{self, HostNotePorts, NoteDialect, NotePortInfo, NotePorts},
            params::{self, HostParams, ParamInfo, Params},
        },
        host::Host,
        id::ClapId,
        plugin::{AudioThread, Plugin},
        plugin_features::*,
        process::{Process, Status},
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
    Factory(factory::Error),
    Events(events::Error),
    Extension(ext::Error),
    Host(host::Error),
    Id(id::Error),
    Plugin(plugin::Error),
    User(Box<dyn std::error::Error + Send + 'static>),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Error::*;
        match self {
            Factory(e) => write!(f, "factory error: {e}"),
            Plugin(e) => write!(f, "plugin error: {e}"),
            Events(e) => write!(f, "events error {e}"),
            Extension(e) => write!(f, "extension error {e}"),
            Host(e) => write!(f, "host error: {e}"),
            Id(e) => write!(f, "id error: {e}"),
            User(e) => write!(f, "user error: {e}"),
        }
    }
}

impl std::error::Error for Error {}
