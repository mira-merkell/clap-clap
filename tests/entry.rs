use std::{
    ffi::{CStr, c_char, c_void},
    ptr::{null, null_mut},
};

use clap_clap::{
    entry::{CLAP_PLUGIN_FACTORY_ID, clap_plugin_factory},
    plugin::Plugin,
};

macro_rules! impl_dummy_plugin {
    ($plug:tt, $id:literal) => {
        #[derive(Default)]
        struct $plug;

        impl Plugin for $plug {
            const ID: &'static str = $id;
            const NAME: &'static str = $id;
            type AudioThread = ();
            type Extensions = ();

            fn activate(
                &mut self,
                _: f64,
                _: u32,
                _: u32,
            ) -> Result<Self::AudioThread, clap_clap::Error> {
                Ok(())
            }
        }
    };
}
impl_dummy_plugin!(Dummy, "dummy");
impl_dummy_plugin!(Dummier, "dummier");

clap_clap::entry!(Dummy, Dummier);
use _clap_entry::clap_entry;
use clap_clap::clap::{CLAP_VERSION, clap_host};

struct DummyHost(clap_host);

impl DummyHost {
    fn new() -> Self {
        extern "C" fn get_extension(_: *const clap_host, _: *const c_char) -> *const c_void {
            null()
        }
        extern "C" fn request_restart(_: *const clap_host) {}
        extern "C" fn request_process(_: *const clap_host) {}
        extern "C" fn request_callback(_: *const clap_host) {}

        Self(clap_host {
            clap_version: CLAP_VERSION,
            host_data: null_mut(),
            name: c"".as_ptr(),
            vendor: c"".as_ptr(),
            url: c"".as_ptr(),
            version: c"".as_ptr(),
            get_extension: Some(get_extension),
            request_restart: Some(request_restart),
            request_process: Some(request_process),
            request_callback: Some(request_callback),
        })
    }

    fn as_clap_host(&self) -> &clap_host {
        &self.0
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
    assert_eq!(n, 2);

    let get_plugin_descriptor = unsafe { *factory }.get_plugin_descriptor.unwrap();
    let desc = unsafe { get_plugin_descriptor(factory, 2) };
    assert!(desc.is_null());

    let desc = unsafe { get_plugin_descriptor(factory, 0) };
    assert!(!desc.is_null());
    let id = unsafe { CStr::from_ptr((*desc).id) };
    assert_eq!(id, c"dummy");

    let desc = unsafe { get_plugin_descriptor(factory, 1) };
    assert!(!desc.is_null());
    let id = unsafe { CStr::from_ptr((*desc).id) };
    assert_eq!(id, c"dummier");

    let create_plugin = unsafe { *factory }.create_plugin.unwrap();
    let host = DummyHost::new();
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

    let entry_deinit = clap_entry.deinit.unwrap();
    unsafe { entry_deinit() }
}
