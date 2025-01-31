use std::{ffi::CStr, ptr::null};

use clap::{
    entry::{CLAP_PLUGIN_FACTORY_ID, clap_plugin_factory},
    plugin::Plugin,
};

macro_rules! impl_dummy_plugin {
    ($plug:tt, $id:literal) => {
        #[derive(Default)]
        struct $plug;

        impl Plugin for $plug {
            const ID: &'static str = $id;
            type AudioThread = ();
            type Extensions = ();

            fn activate(
                &mut self,
                _: f64,
                _: usize,
                _: usize,
            ) -> Result<Self::AudioThread, clap::Error> {
                Ok(())
            }
        }
    };
}
impl_dummy_plugin!(Dummy, "dummy");
impl_dummy_plugin!(Dummier, "dummier");

clap::entry!(Dummy, Dummier);
use _clap_entry::clap_entry;

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

    let entry_deinit = clap_entry.deinit.unwrap();
    unsafe { entry_deinit() }
}
