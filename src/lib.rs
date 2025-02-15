//! A CLAP plugin runtime. ⧉⧉⧉

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
pub mod version;

pub mod prelude {
    #[doc(inline)]
    pub use crate::ext::audio_ports::{AudioPorts, MonoPorts, StereoPorts};
    #[doc(inline)]
    pub use crate::{
        Error, entry,
        ext::Extensions,
        host::Host,
        plugin::{AudioThread, Plugin},
        process::{Process, Status},
    };
}

#[derive(Debug)]
pub enum Error {
    Factory(factory::Error),
    Events(events::Error),
    Host(host::Error),
    Id(id::Error),
    Plugin(plugin::Error),
    Process(process::Error),
    User(Box<dyn std::error::Error + Send + 'static>),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Error::*;
        match self {
            Factory(e) => write!(f, "factory error: {e}"),
            Plugin(e) => write!(f, "plugin error: {e}"),
            Events(e) => write!(f, "events error {e}"),
            Host(e) => write!(f, "host error: {e}"),
            Process(e) => write!(f, "process error: {e}"),
            Id(e) => write!(f, "id error: {e}"),
            User(e) => write!(f, "user error: {e}"),
        }
    }
}

impl std::error::Error for Error {}
