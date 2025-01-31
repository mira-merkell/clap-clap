use std::{
    ffi::{CStr, c_char, c_void},
    ptr::{null, null_mut},
};

use clap::{
    entry::{CLAP_PLUGIN_FACTORY_ID, clap_plugin_factory},
    plugin::Plugin,
};

#[derive(Default)]
struct Dummy;

impl Plugin for Dummy {
    const ID: &'static str = "dummy";
    type AudioThread = ();
    type Extensions = ();

    fn activate(&mut self, _: f64, _: usize, _: usize) -> Result<Self::AudioThread, clap::Error> {
        Ok(())
    }
}

clap::entry!(Dummy);
use _clap_entry::clap_entry;
use clap_sys::clap_host;

struct DummyHost(clap_sys::clap_host);

impl DummyHost {
    const fn new() -> Self {
        extern "C" fn get_extensions(
            _: *const clap_sys::clap_host,
            _: *const c_char,
        ) -> *const c_void {
            null()
        }
        extern "C" fn request_restart(_: *const clap_sys::clap_host) {}
        extern "C" fn request_process(_: *const clap_sys::clap_host) {}
        extern "C" fn request_callback(_: *const clap_sys::clap_host) {}

        Self(clap_sys::clap_host {
            clap_version: clap_sys::CLAP_VERSION,
            host_data: null_mut(),
            name: c"dummy".as_ptr(),
            vendor: c"⧉⧉⧉".as_ptr(),
            url: c"".as_ptr(),
            version: c"1".as_ptr(),
            get_extension: Some(get_extensions),
            request_restart: Some(request_restart),
            request_process: Some(request_process),
            request_callback: Some(request_callback),
        })
    }

    const fn as_clap_host_ptr(&self) -> *const clap_host {
        &raw const self.0
    }
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
    let host = DummyHost::new();
    let plug = unsafe { create_plugin(factory, host.as_clap_host_ptr(), id.as_ptr()) };
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
