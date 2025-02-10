//! A CLAP plugin runtime. ⧉⧉⧉
//!
//! # Compatibility
//!
//! This library aims at full compatibility with the CLAP API. A lot of
//! functionality, however, is still missing. As a workaround, the library's [
//! `Host`] type provides two methods that you can use to obtain raw pointers to
//! CLAP's structs [ `clap_host`] and [`clap_plugin`]. During the initialization
//! of the plugin, use the host handle like this:
//!
//! ```no_compile
//! impl Plugin for MyPlug {
//!     fn init(&mut self, host: Arc<clap::Host>) -> Result<(), clap_clap::Error> {
//!         let clap_host = unsafe { host._raw_clap_host() };
//!         let clap_plugin = unsafe { host._raw_clap_plugin() };
//!
//!         // (...)
//!
//!         Ok(())
//!     }
//! }
//! ```
//!
//! You can then take the pointer to the raw `clap_plugin` to, e.g., wrap
//! `clap_plugin.get_extension()` in your own function providing additional
//! functionality. Note that the pointer: `clap_plugin.plugin_data` is reserved
//! by this library's runtime, and overriding it will most surely result in
//! undefined behavior.
//!
//! Since it would be difficult to define safety requirements for those methods,
//! they should be considered a last resort, generally unsafe, and their use is
//! discouraged.
//!
//! See also the module: [`clap_clap::ffi`] for the definitions of raw CLAP
//! structures.
//!
//!
//! [`Host`]: crate::host::Host
//!
//! [`clap_host`]: https://github.com/free-audio/clap/blob/main/include/clap/host.h
//!
//! [`clap_plugin`]: https://github.com/free-audio/clap/blob/main/include/clap/plugin.h
//!
//! [`clap_clap::ffi`]: crate::ffi

#[doc(hidden)]
pub mod entry;
pub mod ext;
#[doc(hidden)]
pub mod factory;
pub mod ffi;
pub mod host;
pub mod id;
pub mod plugin;
pub mod plugin_features;
pub mod process;
pub mod string_sizes;
pub mod version;

pub mod prelude {
    #[doc(inline)]
    pub use crate::{
        Error, entry,
        ext::plugin::{
            Extensions,
            audio_ports::{AudioPorts, MonoPorts, StereoPorts},
        },
        host::Host,
        plugin::{AudioThread, Plugin},
        process::{Process, Status},
    };
}

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
            User(ec) => write!(f, "user error: {ec}"),
        }
    }
}

impl std::error::Error for Error {}
