pub mod ffi {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    #![allow(warnings, unused)]
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

pub use crate::ffi::{
    CLAP_VERSION_MAJOR, CLAP_VERSION_MINOR, CLAP_VERSION_REVISION, clap_host, clap_plugin,
    clap_plugin_descriptor, clap_plugin_entry, clap_plugin_factory, clap_version,
};
use std::ffi::CStr;

macro_rules! pub_const_cstr {
    ($glob:ident) => {
        pub const $glob: &CStr = unsafe { CStr::from_bytes_with_nul_unchecked(ffi::$glob) };
    };
}

pub_const_cstr!(CLAP_PLUGIN_FACTORY_ID);

unsafe impl Send for clap_plugin_descriptor {}
unsafe impl Sync for clap_plugin_descriptor {}

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
