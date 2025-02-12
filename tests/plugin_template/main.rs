#[path = "../../examples/plugin_template.rs"]
mod plugin;

use clap_clap::ext::host::log::Severity;
use plugin::_clap_entry::clap_entry as ENTRY;

use crate::fixtures::{TestPlugin, test_host::TestHostConfig};

mod fixtures;

#[test]
fn create_plugin_no_log() {
    let host = TestHostConfig {
        name: c"",
        vendor: c"",
        url: c"",
        version: c"",
        implements_ext_log: false,
    }
    .build();
    let plugin = TestPlugin::new(&host);

    assert!(!unsafe { plugin.init.unwrap()(&*plugin) });
}

#[test]
fn create_plugin_log() {
    let host = TestHostConfig {
        name: c"test host",
        vendor: c"",
        url: c"",
        version: c"[32",
        implements_ext_log: true,
    }
    .build();

    let plugin = TestPlugin::new(&host);

    assert!(unsafe { plugin.init.unwrap()(&*plugin) });
}

#[test]
fn plugin_init_log() {
    let host = TestHostConfig {
        name: c"test host",
        vendor: c"",
        url: c"",
        version: c"[32",
        implements_ext_log: true,
    }
    .build();

    let plugin = TestPlugin::new(&host);
    assert!(unsafe { plugin.init.unwrap()(&*plugin) });

    let log_msg = &host.log_msg.lock().unwrap()[0];
    assert_eq!(log_msg.0, Severity::Info);
    assert_eq!(log_msg.1, "hello, sonic world");
}
