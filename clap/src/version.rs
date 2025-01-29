pub type ClapVersion = clap_sys::clap_version;

pub const CLAP_VERSION: ClapVersion = ClapVersion {
    major: clap_sys::CLAP_VERSION_MAJOR,
    minor: clap_sys::CLAP_VERSION_MINOR,
    revision: clap_sys::CLAP_VERSION_REVISION,
};

#[cfg(test)]
mod tests {
    use super::*;

    // This is a copy of a test from clap/version.h
    #[test]
    fn clap_version_is_compatible() {
        assert!(
            CLAP_VERSION.major >= 1,
            "versions 0.x.y were used during development stage and aren't compatible"
        );
    }
}
