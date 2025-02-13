use std::ops::Deref;

use clap_clap::ffi::{CLAP_PLUGIN_FACTORY_ID, clap_host, clap_plugin, clap_plugin_factory};

use crate::{ENTRY, fixtures::test_host::TestHost};

fn get_factory<'a>() -> &'a clap_plugin_factory {
    let factory = unsafe {
        &*(ENTRY.get_factory.unwrap()(CLAP_PLUGIN_FACTORY_ID.as_ptr())
            as *const clap_plugin_factory)
    };
    assert_eq!(unsafe { factory.get_plugin_count.unwrap()(factory) }, 1);

    factory
}

pub mod test_host {
    use std::{
        ffi::{CStr, CString, c_char, c_void},
        marker::PhantomPinned,
        pin::Pin,
        ptr::{null, null_mut},
        sync::Mutex,
    };

    use clap_clap::{
        ext::host::log::Severity,
        ffi::{CLAP_EXT_LOG, CLAP_VERSION, clap_host, clap_host_log, clap_log_severity},
    };

    pub struct TestHostConfig<'a> {
        pub name: &'a CStr,
        pub vendor: &'a CStr,
        pub url: &'a CStr,
        pub version: &'a CStr,
        pub implements_ext_log: bool,
    }

    impl<'a> TestHostConfig<'a> {
        pub fn build(self) -> Pin<Box<TestHost<'a>>> {
            TestHost::new(self)
        }
    }

    #[allow(unused)]
    pub struct TestHost<'a> {
        clap_host: clap_host,
        clap_host_log: clap_host_log,

        pub config: TestHostConfig<'a>,
        pub log_msg: Mutex<Vec<(Severity, String)>>,

        _marker: PhantomPinned,
    }

    extern "C-unwind" fn get_extension(
        host: *const clap_host,
        extension_id: *const c_char,
    ) -> *const c_void {
        assert!(!host.is_null());
        let host: &TestHost = unsafe { &*((*host).host_data as *const _) };

        assert!(!extension_id.is_null());
        let extension_id = unsafe { CStr::from_ptr(extension_id) };

        if extension_id == CLAP_EXT_LOG && host.config.implements_ext_log {
            return &raw const host.clap_host_log as *const _;
        }

        null()
    }

    extern "C-unwind" fn request_restart(_: *const clap_host) {}
    extern "C-unwind" fn request_reset(_: *const clap_host) {}
    extern "C-unwind" fn request_callback(_: *const clap_host) {}

    extern "C-unwind" fn log(
        host: *const clap_host,
        severity: clap_log_severity,
        msg: *const c_char,
    ) {
        assert!(!host.is_null());
        let host: &TestHost = unsafe { &*((*host).host_data as *const _) };

        assert!(!msg.is_null());
        let msg = CString::from(unsafe { CStr::from_ptr(msg) })
            .into_string()
            .unwrap();

        let mut log = host.log_msg.lock().unwrap();
        log.push((severity.try_into().unwrap(), msg))
    }

    impl<'a> TestHost<'a> {
        pub fn new(config: TestHostConfig<'a>) -> Pin<Box<Self>> {
            let mut host = Box::new(Self {
                clap_host: clap_host {
                    clap_version: CLAP_VERSION,
                    host_data: null_mut(),
                    name: config.name.as_ptr(),
                    vendor: config.vendor.as_ptr(),
                    url: config.url.as_ptr(),
                    version: config.version.as_ptr(),
                    get_extension: Some(get_extension),
                    request_restart: Some(request_restart),
                    request_process: Some(request_reset),
                    request_callback: Some(request_callback),
                },
                clap_host_log: clap_host_log { log: Some(log) },
                log_msg: Mutex::new(Vec::new()),
                config,
                _marker: PhantomPinned,
            });

            host.clap_host.host_data = &raw const *host as *mut _;

            Box::into_pin(host)
        }

        pub const fn as_clap_host(&self) -> &clap_host {
            &self.clap_host
        }
    }
}

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
