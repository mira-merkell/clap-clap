mod ffi {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    #![allow(warnings, unused)]
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

pub use crate::ffi::{CLAP_VERSION_MAJOR, CLAP_VERSION_MINOR, CLAP_VERSION_REVISION, clap_version};

pub const CLAP_VERSION: clap_version = clap_version {
    major: CLAP_VERSION_MAJOR,
    minor: CLAP_VERSION_MINOR,
    revision: CLAP_VERSION_REVISION,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clap_version() {
        assert_eq!(CLAP_VERSION.major, 1);
        assert_eq!(CLAP_VERSION.minor, 2);
        assert_eq!(CLAP_VERSION.revision, 3);
    }
}