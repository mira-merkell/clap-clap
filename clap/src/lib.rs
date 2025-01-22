use std::fmt::{Debug, Display, Formatter};

pub use clap_sys;

pub const CLAP_VERSION: (u32, u32, u32) = (
    clap_sys::CLAP_VERSION_MAJOR,
    clap_sys::CLAP_VERSION_MINOR,
    clap_sys::CLAP_VERSION_REVISION,
);

#[derive(Debug, Copy, Clone)]
pub enum Error {
    Entry,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Entry => {
                write!(f, "Entry")
            }
        }
    }
}

impl std::error::Error for Error {}

#[derive(Debug)]
pub struct PluginDestriptor {}

pub trait Plugin {}

pub trait Factory {
    type Plugin: Plugin;

    fn plugin_count(&self) -> u32;
    fn plugin_descriptor(&self) -> &'static PluginDestriptor;
    fn create_plugin(&self) -> Box<Self::Plugin>;
}

pub trait Entry {
    const CLAP_VERSION: (u32, u32, u32);

    type Factory: Factory;

    fn init(plugin_path: &str) -> Result<(), Error>;
    fn deinit();
    fn get_factory() -> &'static Self::Factory;
}

pub use entry::clap_entry;

mod entry {
    use crate::Entry;
    use std::ffi::{CStr, c_char, c_void};

    use clap_sys::{clap_plugin_entry, clap_version};

    extern "C" fn init<E: Entry>(plugin_path: *const c_char) -> bool {
        let plugin_path = unsafe { CStr::from_ptr(plugin_path) }
            .to_str()
            .expect("plugin_path should be a properly formatted C string");

        <E as Entry>::init(plugin_path).is_ok()
    }

    extern "C" fn deinit<E: Entry>() {
        <E as Entry>::deinit()
    }

    extern "C" fn get_factory<E: Entry>(_factory_id: *const c_char) -> *const c_void {
        todo!()
    }

    pub const fn clap_entry<E: Entry>() -> clap_plugin_entry {
        clap_plugin_entry {
            clap_version: clap_version {
                major: <E as Entry>::CLAP_VERSION.0,
                minor: <E as Entry>::CLAP_VERSION.1,
                revision: <E as Entry>::CLAP_VERSION.2,
            },
            init: Some(init::<E>),
            deinit: Some(deinit::<E>),
            get_factory: Some(get_factory::<E>),
        }
    }
}

#[macro_export]
macro_rules! entry {
    ($entry:ty) => {
        mod _clap_entry {

            use super::*;

            #[allow(non_upper_case_globals)]
            #[allow(non_camel_case_types)]
            #[allow(warnings, unused)]
            #[unsafe(no_mangle)]
            static clap_entry: $crate::clap_sys::clap_plugin_entry = $crate::clap_entry::<$entry>();
        }
    };
}
