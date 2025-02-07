pub type ClapVersion = crate::clap_sys::clap_version;

pub const CLAP_VERSION: ClapVersion = ClapVersion {
    major: crate::clap_sys::CLAP_VERSION_MAJOR,
    minor: crate::clap_sys::CLAP_VERSION_MINOR,
    revision: crate::clap_sys::CLAP_VERSION_REVISION,
};
