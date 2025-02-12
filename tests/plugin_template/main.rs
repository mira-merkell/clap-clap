#[path = "../../examples/plugin_template.rs"]
mod plugin;

use plugin::_clap_entry::clap_entry as ENTRY;

use crate::fixtures::{TestHostConfig, TestPlugin};

mod fixtures;

#[test]
fn create_plugin_01() {
    let host = TestHostConfig {
        name: "",
        vendor: "",
        url: "",
        version: "",
    }
    .build();
    let plugin = TestPlugin::new(&host);

    assert!(unsafe { plugin.init.unwrap()(&*plugin) });
}
