#![cfg(test)]

use clap_clap::clap::entry::{CLAP_PLUGIN_FACTORY_ID, clap_plugin_entry, clap_plugin_factory};

#[link(name = "test_dummy_plugin")]
unsafe extern "C" {
    static clap_entry: clap_plugin_entry;
}

// There should be only one test function initializing the dynamic library:
// test_dummy_plugin.
#[test]
fn export_clap_entry() {
    let get_factory = unsafe { clap_entry.get_factory }.unwrap();
    let factory = unsafe { get_factory(CLAP_PLUGIN_FACTORY_ID.as_ptr()) };

    assert!(!factory.is_null());
    let factory = factory as *const clap_plugin_factory;

    let get_plugin_count = unsafe { *factory }.get_plugin_count.unwrap();
    let n = unsafe { get_plugin_count(factory) };
    assert_eq!(n, 1);

    let get_plugin_descriptor = unsafe { *factory }.get_plugin_descriptor.unwrap();
    let desc = unsafe { get_plugin_descriptor(factory, 1) };
    assert!(desc.is_null());

    let desc = unsafe { get_plugin_descriptor(factory, 0) };
    assert!(!desc.is_null());
}
