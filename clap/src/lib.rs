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
