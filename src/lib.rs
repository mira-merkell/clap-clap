//! A CLAP plugin runtime. ⧉⧉⧉

#[doc(hidden)]
pub mod entry;
pub mod ext;
#[doc(hidden)]
pub mod factory;
#[doc(hidden)]
pub mod ffi;
pub mod host;
pub mod id;
pub mod plugin;
pub mod process;
pub mod string_sizes;
pub mod version;

#[derive(Debug, Clone)]
pub enum Error {
    Factory(factory::Error),
    Plugin(plugin::Error),
    Host(host::Error),
    Process(process::Error),
    Id(id::Error),
    User(i32),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Error::*;
        match self {
            Factory(e) => write!(f, "factory module: {e}"),
            Plugin(e) => write!(f, "plugin module: {e}"),
            Host(e) => write!(f, "host module: {e}"),
            Process(e) => write!(f, "process module: {e}"),
            Id(e) => write!(f, "id: {e}"),
            User(ec) => write!(f, "user error, error code: {ec}"),
        }
    }
}

impl std::error::Error for Error {}
