use std::{
    ffi::{CStr, CString, c_char, c_void},
    marker::PhantomPinned,
    ops::Deref,
    pin::Pin,
    ptr::{null, null_mut},
};

use clap_clap::ffi::{
    CLAP_EXT_LOG, CLAP_PLUGIN_FACTORY_ID, CLAP_VERSION, clap_host, clap_host_log,
    clap_log_severity, clap_plugin, clap_plugin_factory,
};

use crate::ENTRY;

fn get_factory<'a>() -> &'a clap_plugin_factory {
    let factory = unsafe {
        ENTRY.get_factory.unwrap()(CLAP_PLUGIN_FACTORY_ID.as_ptr()) as *const clap_plugin_factory
    };
    assert!(!factory.is_null());

    let factory = unsafe { &*factory };
    assert_eq!(unsafe { factory.get_plugin_count.unwrap()(factory) }, 1);

    factory
}

#[derive(Debug, Default)]
pub struct TestHostConfig<'a> {
    pub name: &'a CStr,
    pub vendor: &'a CStr,
    pub url: &'a CStr,
    pub version: &'a CStr,

    pub ext_log_messages: Option<&'a mut Vec<(clap_log_severity, CString)>>,
}

impl<'a> TestHostConfig<'a> {
    pub fn build(self) -> Pin<Box<TestHost<'a>>> {
        TestHost::new(self)
    }
}

#[derive(Debug)]
#[allow(unused)]
pub struct TestHost<'a> {
    config: TestHostConfig<'a>,

    clap_host: clap_host,
    clap_host_log: clap_host_log,

    _marker: PhantomPinned,
}

impl<'a> TestHost<'a> {
    pub fn new(config: TestHostConfig<'a>) -> Pin<Box<Self>> {
        extern "C-unwind" fn get_extension(
            host: *const clap_host,
            extension_id: *const c_char,
        ) -> *const c_void {
            assert!(!host.is_null());
            let test_host: &TestHost = unsafe { &*(*host).host_data.cast() };
            let extension_id = unsafe { CStr::from_ptr(extension_id) };

            if extension_id == CLAP_EXT_LOG && test_host.config.ext_log_messages.is_some() {
                return &raw const test_host.clap_host_log as *const _;
            }

            null()
        }
        extern "C-unwind" fn request_restart(_: *const clap_host) {}
        extern "C-unwind" fn request_reset(_: *const clap_host) {}
        extern "C-unwind" fn request_callback(_: *const clap_host) {}

        extern "C-unwind" fn log_log(
            host: *const clap_host,
            severity: clap_log_severity,
            msg: *const c_char,
        ) {
            assert!(!host.is_null());
            let test_host: &mut TestHost = unsafe { &mut *(*host).host_data.cast() };

            assert!(!msg.is_null());
            if let Some(buf) = &mut test_host.config.ext_log_messages {
                let msg = unsafe { CStr::from_ptr(msg) }.to_owned();
                buf.push((severity, msg))
            }
        }

        let mut host = Box::new(Self {
            clap_host: clap_host {
                clap_version: CLAP_VERSION,
                host_data: null_mut(),
                // Points to the string buffer on the heap.
                // The string can still be moved.
                name: config.name.as_ptr(),
                vendor: config.vendor.as_ptr(),
                url: config.url.as_ptr(),
                version: config.version.as_ptr(),
                get_extension: Some(get_extension),
                request_restart: Some(request_restart),
                request_process: Some(request_reset),
                request_callback: Some(request_callback),
            },
            clap_host_log: clap_host_log { log: Some(log_log) },
            config,
            _marker: PhantomPinned,
        });

        host.clap_host.host_data = (&raw mut *host).cast();
        host.into()
    }

    pub const fn as_clap_host(&self) -> &clap_host {
        &self.clap_host
    }
}

unsafe impl Send for TestHost<'_> {}
unsafe impl Sync for TestHost<'_> {}

pub struct TestPlugin<'a>(&'a clap_plugin);

impl<'a> TestPlugin<'a> {
    pub fn new(host: &'a TestHost) -> Self {
        let factory = get_factory();
        let plugin_desc = unsafe { factory.get_plugin_descriptor.unwrap()(factory, 0) };
        assert!(!plugin_desc.is_null());
        let plugin_id = unsafe { *plugin_desc }.id;
        assert!(!plugin_id.is_null());
        let plugin =
            unsafe { factory.create_plugin.unwrap()(factory, host.as_clap_host(), plugin_id) };
        assert!(!plugin.is_null());

        Self(unsafe { &*plugin })
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
