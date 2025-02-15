use std::{
    ffi::{CStr, CString, c_char, c_void},
    marker::PhantomPinned,
    pin::Pin,
    ptr::{null, null_mut},
    sync::Mutex,
};

use clap_clap::{
    ffi::{
        CLAP_EXT_AUDIO_PORTS, CLAP_EXT_LOG, clap_host, clap_host_audio_ports, clap_host_log,
        clap_log_severity,
    },
    host::Host,
    version::CLAP_VERSION,
};

#[derive(Debug, Default)]
pub struct TestHostConfig<'a> {
    pub name: &'a CStr,
    pub vendor: &'a CStr,
    pub url: &'a CStr,
    pub version: &'a CStr,

    pub impl_ext_log: bool,
    pub impl_ext_audio_ports: bool,
}

impl<'a> TestHostConfig<'a> {
    pub fn build(self) -> Pin<Box<TestHost<'a>>> {
        TestHost::new(self)
    }
}

#[derive(Debug)]
#[allow(unused)]
pub struct TestHost<'a> {
    clap_host: clap_host,

    pub clap_host_audio_ports: clap_host_audio_ports,
    pub ext_audio_port_call_rescan_flags: u32,

    pub clap_host_log: clap_host_log,
    pub ext_log_messages: Mutex<Vec<(clap_log_severity, CString)>>,

    config: TestHostConfig<'a>,
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

            if extension_id == CLAP_EXT_AUDIO_PORTS && test_host.config.impl_ext_audio_ports {
                return (&raw const test_host.clap_host_audio_ports).cast();
            }
            if extension_id == CLAP_EXT_LOG && test_host.config.impl_ext_log {
                return (&raw const test_host.clap_host_log).cast();
            }

            null()
        }
        extern "C-unwind" fn request_restart(_: *const clap_host) {}
        extern "C-unwind" fn request_reset(_: *const clap_host) {}
        extern "C-unwind" fn request_callback(_: *const clap_host) {}

        extern "C-unwind" fn audio_ports_is_rescan_flag_supported(
            host: *const clap_host,
            _: u32,
        ) -> bool {
            assert!(!host.is_null());
            true
        }

        extern "C-unwind" fn audio_ports_rescan(host: *const clap_host, flags: u32) {
            assert!(!host.is_null());
            let test_host: &mut TestHost = unsafe { &mut *(*host).host_data.cast() };
            test_host.ext_audio_port_call_rescan_flags = flags;
        }

        extern "C-unwind" fn log_log(
            host: *const clap_host,
            severity: clap_log_severity,
            msg: *const c_char,
        ) {
            assert!(!host.is_null());
            let test_host: &mut TestHost = unsafe { &mut *(*host).host_data.cast() };

            assert!(!msg.is_null());
            let msg = unsafe { CStr::from_ptr(msg) }.to_owned();

            let mut buf = test_host.ext_log_messages.lock().unwrap();
            buf.push((severity, msg))
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
            ext_log_messages: Mutex::new(Vec::new()),
            clap_host_audio_ports: clap_host_audio_ports {
                is_rescan_flag_supported: Some(audio_ports_is_rescan_flag_supported),
                rescan: Some(audio_ports_rescan),
            },
            ext_audio_port_call_rescan_flags: 0,
            config,
            _marker: PhantomPinned,
        });

        host.clap_host.host_data = (&raw mut *host).cast();
        host.into()
    }

    pub const fn clap_host(&self) -> &clap_host {
        &self.clap_host
    }
}

unsafe impl Send for TestHost<'_> {}
unsafe impl Sync for TestHost<'_> {}

#[test]
fn host_new() {
    let test_host = TestHostConfig {
        name: c"test_host",
        url: c"test_url",
        vendor: c"test_vendor",
        version: c"test_version",
        impl_ext_log: false,
        ..Default::default()
    }
    .build();

    let host = unsafe { Host::new(test_host.clap_host()) };

    assert_eq!(host.name(), "test_host");
    assert_eq!(host.url(), "test_url");
    assert_eq!(host.vendor(), "test_vendor");
    assert_eq!(host.version(), "test_version");
}
