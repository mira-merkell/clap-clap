use std::{
    ffi::{CString, c_char, c_void},
    ops::Deref,
    ptr::{null, null_mut},
};

use clap_clap::ffi::{
    CLAP_PLUGIN_FACTORY_ID, CLAP_VERSION, clap_host, clap_plugin, clap_plugin_factory,
};

use crate::ENTRY;

fn get_factory<'a>() -> &'a clap_plugin_factory {
    let factory = unsafe {
        &*(ENTRY.get_factory.unwrap()(CLAP_PLUGIN_FACTORY_ID.as_ptr())
            as *const clap_plugin_factory)
    };
    assert_eq!(unsafe { factory.get_plugin_count.unwrap()(factory) }, 1);

    factory
}

pub struct TestHostConfig<'a> {
    pub name: &'a str,
    pub vendor: &'a str,
    pub url: &'a str,
    pub version: &'a str,
}

impl TestHostConfig<'_> {
    pub fn build(self) -> TestHost {
        TestHost::new(self)
    }
}

#[derive(Debug)]
#[allow(unused)]
pub struct TestHost {
    name: CString,
    vendor: CString,
    url: CString,
    version: CString,

    clap_host: clap_host,
}

impl TestHost {
    pub fn new(config: TestHostConfig) -> Self {
        extern "C" fn get_extension(_: *const clap_host, _: *const c_char) -> *const c_void {
            null()
        }
        extern "C" fn request_restart(_: *const clap_host) {}
        extern "C" fn request_reset(_: *const clap_host) {}
        extern "C" fn request_callback(_: *const clap_host) {}

        let name = CString::new(config.name).unwrap();
        let vendor = CString::new(config.vendor).unwrap();
        let url = CString::new(config.url).unwrap();
        let version = CString::new(config.version).unwrap();

        Self {
            clap_host: clap_host {
                clap_version: CLAP_VERSION,
                host_data: null_mut(),
                // Points to the string buffer on the heap.
                // The string can still be moved.
                name: name.as_ptr(),
                vendor: vendor.as_ptr(),
                url: url.as_ptr(),
                version: version.as_ptr(),
                get_extension: Some(get_extension),
                request_restart: Some(request_restart),
                request_process: Some(request_reset),
                request_callback: Some(request_callback),
            },
            name,
            vendor,
            url,
            version,
        }
    }

    pub const fn as_clap_host(&self) -> &clap_host {
        &self.clap_host
    }
}

unsafe impl Send for TestHost {}
unsafe impl Sync for TestHost {}

unsafe fn create_plugin(host: &clap_host) -> &clap_plugin {
    let factory = get_factory();

    let plugin_desc = unsafe { factory.get_plugin_descriptor.unwrap()(factory, 0) };
    assert!(!plugin_desc.is_null());
    let plugin_id = unsafe { *plugin_desc }.id;
    assert!(!plugin_id.is_null());
    let plugin = unsafe { factory.create_plugin.unwrap()(factory, host, plugin_id) };
    assert!(!plugin.is_null());

    unsafe { &*plugin }
}

pub struct TestPlugin<'a>(&'a clap_plugin);

impl<'a> TestPlugin<'a> {
    pub fn new(host: &'a TestHost) -> Self {
        Self(unsafe { create_plugin(host.as_clap_host()) })
    }
}

impl Drop for TestPlugin<'_> {
    fn drop(&mut self) {
        unsafe { self.0.destroy.unwrap()(self.0) }
    }
}

impl Deref for TestPlugin<'_> {
    type Target = clap_plugin;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}
