//! A CLAP plugin runtime. ⧉⧉⧉

#[cfg(not(any(target_pointer_width = "32", target_pointer_width = "64")))]
compile_error!("target's pointer width must be at least 32");

#[doc(hidden)]
pub mod clap;
pub mod entry;
pub mod ext;
pub mod factory;
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
