use std::error;
use std::fmt::{Display, Formatter};

pub mod entry;
pub mod ext;
pub mod factory;
pub mod host;
pub mod id;
pub mod plugin;
pub mod process;

pub mod string_sizes {
    pub const CLAP_NAME_SIZE: usize = clap_sys::CLAP_NAME_SIZE;
    pub const CLAP_PATH_SIZE: usize = clap_sys::CLAP_PATH_SIZE;
}

pub mod version {
    pub type ClapVersion = clap_sys::clap_version;

    pub const CLAP_VERSION: ClapVersion = ClapVersion {
        major: clap_sys::CLAP_VERSION_MAJOR,
        minor: clap_sys::CLAP_VERSION_MINOR,
        revision: clap_sys::CLAP_VERSION_REVISION,
    };
}

#[derive(Debug, Clone)]
pub enum Error {
    Plugin(plugin::Error),
    Host(host::Error),
    Process(process::Error),
    User(i32),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use Error::*;
        match self {
            Plugin(e) => write!(f, "Plugin: {}", e),
            Host(e) => write!(f, "Host: {}", e),
            Process(e) => write!(f, "Process: {}", e),
            User(rc) => write!(f, "User: {rc}"),
        }
    }
}

impl error::Error for Error {}
