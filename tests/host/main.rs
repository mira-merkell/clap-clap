use clap_clap::host::Host;

use crate::test_host::TestHostConfig;

mod test_host;

#[test]
fn host_new() {
    let test_host = TestHostConfig {
        name: "test_host",
        url: "test_url",
        vendor: "test_vendor",
        version: "test_version",
    }
    .build();

    let host = unsafe { Host::new(test_host.as_clap_host()) };

    assert_eq!(host.name(), "test_host");
    assert_eq!(host.url(), "test_url");
    assert_eq!(host.vendor(), "test_vendor");
    assert_eq!(host.version(), "test_version");
}
