#[allow(non_camel_case_types)]
#[allow(non_upper_case_globals)]
mod bindings;

pub use bindings::*;

pub const CLAP_VERSION: clap_version = clap_version {
    major: CLAP_VERSION_MAJOR,
    minor: CLAP_VERSION_MINOR,
    revision: CLAP_VERSION_REVISION,
};
