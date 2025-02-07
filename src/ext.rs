//! CLAP Extensions.
//!
//! The [CLAP API] defines the interface between plugins and hosts, treating
//! both extensions equally—there's no distinction in how extensions are used by
//! either.
//!
//! This library adopts the plugin's perspective, meaning there is only one
//! host. Host extensions are concrete types found in the [`ext::host`] module.
//!
//! Plugin extensions, on the other hand, are implemented by the library user as
//! traits in the [`ext::plugin`] module. This module also provides some
//! concrete implementations for convenience, such as the [`StereoPorts`] type,
//! which defines a static stereo port layout.
//!
//! [CLAP API]: https://github.com/free-audio/clap/tree/main/include/clap
//! [`ext::host`]: crate::ext::host
//! [`ext::plugin`]: crate::ext::plugin
//! [`StereoPorts`]: crate::ext::plugin::audio_ports::StereoPorts

pub mod host;
pub mod plugin;
