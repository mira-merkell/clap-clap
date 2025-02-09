use clap_clap::string_sizes::{CLAP_NAME_SIZE, CLAP_PATH_SIZE};

#[test]
fn name_size() {
    // This value is stable CLAP API, and should never change.
    assert_eq!(CLAP_NAME_SIZE, 256);
}

#[test]
fn path_size() {
    // This value is stable CLAP API, and should never change.
    assert_eq!(CLAP_PATH_SIZE, 1024);
}
