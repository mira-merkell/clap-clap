pub type ClapVersion = crate::ffi::clap_version;

pub const CLAP_VERSION: ClapVersion = ClapVersion {
    major: crate::ffi::CLAP_VERSION_MAJOR,
    minor: crate::ffi::CLAP_VERSION_MINOR,
    revision: crate::ffi::CLAP_VERSION_REVISION,
};
