pub type ClapVersion = crate::clap::clap_version;

pub const CLAP_VERSION: ClapVersion = ClapVersion {
    major: crate::clap::CLAP_VERSION_MAJOR,
    minor: crate::clap::CLAP_VERSION_MINOR,
    revision: crate::clap::CLAP_VERSION_REVISION,
};
