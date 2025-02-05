use std::{ffi::CStr, pin::Pin, ptr::null};

use clap_clap::entry::{CLAP_PLUGIN_FACTORY_ID, clap_plugin_factory};

#[path = "../plugin/dummy.rs"]
mod dummy_plugin;

#[path = "../host/test_host.rs"]
mod test_host;

use crate::dummy_plugin::Dummy;

clap_clap::entry!(Dummy);
use _clap_entry::clap_entry;

use crate::test_host::{TestHost, TestHostConfig};

fn dummy_host() -> Pin<Box<TestHost>> {
    TestHostConfig {
        name: "",
        vendor: "",
        url: "",
        version: "",
    }
    .build()
}

#[test]
fn export_clap_entry() {
    let entry_init = clap_entry.init.unwrap();
    assert!(!unsafe { entry_init(null()) });
    assert!(unsafe { entry_init(c"".as_ptr()) });

    let get_factory = clap_entry.get_factory.unwrap();
    let factory = unsafe { get_factory(null()) };
    assert!(factory.is_null());
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

    let id = unsafe { CStr::from_ptr((*desc).id) };
    assert_eq!(id, c"dummy");

    let create_plugin = unsafe { *factory }.create_plugin.unwrap();
    let host = dummy_host();
    let plug = unsafe { create_plugin(factory, host.as_clap_host(), id.as_ptr()) };
    assert!(!plug.is_null());

    let plug_desc = unsafe { *plug }.desc;
    assert!(!plug_desc.is_null());
    let plug_id = unsafe { CStr::from_ptr((*plug_desc).id) };
    assert_eq!(id, plug_id);

    let plugin_destroy = unsafe { *plug }.destroy.unwrap();
    unsafe { plugin_destroy(plug) };

    let entry_deinit = clap_entry.deinit.unwrap();
    unsafe { entry_deinit() }
}
