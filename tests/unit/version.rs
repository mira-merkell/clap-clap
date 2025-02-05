use clap_clap::version::CLAP_VERSION;

// This is a copy of a test from clap/version.h
#[test]
fn clap_version_is_compatible() {
    if CLAP_VERSION.major < 1 {
        panic!("versions 0.x.y were used during development stage and aren't compatible")
    };
}
