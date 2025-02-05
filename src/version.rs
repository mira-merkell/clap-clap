pub type ClapVersion = clap_sys::clap_version;

pub const CLAP_VERSION: ClapVersion = ClapVersion {
    major: clap_sys::CLAP_VERSION_MAJOR,
    minor: clap_sys::CLAP_VERSION_MINOR,
    revision: clap_sys::CLAP_VERSION_REVISION,
};
