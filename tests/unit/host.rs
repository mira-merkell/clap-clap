use std::{
    ffi::{CStr, CString},
    marker::PhantomPinned,
    pin::Pin,
    ptr::null_mut,
};

use clap_clap::{
    ext::host::log::{Error::Callback, Severity},
    ffi::{
        CLAP_LOG_DEBUG, CLAP_LOG_ERROR, CLAP_LOG_FATAL, CLAP_LOG_HOST_MISBEHAVING, CLAP_LOG_INFO,
        CLAP_LOG_PLUGIN_MISBEHAVING, CLAP_LOG_WARNING, clap_host, clap_host_log, clap_log_severity,
    },
    host::Host,
    version::CLAP_VERSION,
};

use crate::host::test_host_ffi::{
    get_extension, log_log, request_callback, request_process, request_restart,
};

#[derive(Debug, Default)]
pub struct TestHostConfig<'a> {
    pub name: &'a CStr,
    pub vendor: &'a CStr,
    pub url: &'a CStr,
    pub version: &'a CStr,

    pub log_implements: bool,
    pub log_messages: Option<&'a mut Vec<(clap_log_severity, CString)>>,
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
    clap_host_log: clap_host_log,
    config: TestHostConfig<'a>,
    _marker: PhantomPinned,
}

mod test_host_ffi {
    use std::{
        ffi::{CStr, c_char, c_void},
        ptr::null,
    };

    use clap_clap::ffi::{CLAP_EXT_LOG, clap_host, clap_log_severity};

    use crate::host::TestHost;

    pub extern "C-unwind" fn get_extension(
        host: *const clap_host,
        extension_id: *const c_char,
    ) -> *const c_void {
        assert!(!host.is_null());
        let test_host: &TestHost = unsafe { &*((*host).host_data as *const _) };
        let extension_id = unsafe { CStr::from_ptr(extension_id) };

        if extension_id == CLAP_EXT_LOG && test_host.config.log_implements {
            return &raw const test_host.clap_host_log as *const _;
        }

        null()
    }

    pub extern "C-unwind" fn request_restart(_: *const clap_host) {}

    pub extern "C-unwind" fn request_process(_: *const clap_host) {}

    pub extern "C-unwind" fn request_callback(_: *const clap_host) {}

    pub extern "C-unwind" fn log_log(
        host: *const clap_host,
        severity: clap_log_severity,
        msg: *const c_char,
    ) {
        assert!(!host.is_null());
        let test_host: &mut TestHost = unsafe { &mut *((*host).host_data as *mut _) };
        if let Some(buf) = &mut test_host.config.log_messages {
            assert!(!msg.is_null());
            let msg = unsafe { CStr::from_ptr(msg) }.to_owned();
            buf.push((severity, msg));
        }
    }
}

impl<'a> TestHost<'a> {
    fn new(config: TestHostConfig<'a>) -> Pin<Box<TestHost<'a>>> {
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
                request_process: Some(request_process),
                request_callback: Some(request_callback),
            },
            clap_host_log: clap_host_log { log: Some(log_log) },
            config,
            _marker: PhantomPinned,
        });
        host.clap_host.host_data = &raw mut *host as *mut _;
        Box::into_pin(host)
    }

    pub const fn as_clap_host(&self) -> &clap_host {
        &self.clap_host
    }
}

#[test]
fn host_new() {
    let test_host = TestHostConfig {
        name: c"test_host",
        url: c"test_url",
        vendor: c"test_vendor",
        version: c"test_version",
        log_implements: false,
        ..Default::default()
    }
    .build();

    let host = unsafe { Host::new(test_host.as_clap_host()) };

    assert_eq!(host.name(), "test_host");
    assert_eq!(host.url(), "test_url");
    assert_eq!(host.vendor(), "test_vendor");
    assert_eq!(host.version(), "test_version");
}

#[test]
fn host_doesnt_implement_log() {
    let test_host = TestHostConfig {
        log_implements: false,
        ..Default::default()
    }
    .build();

    let host = unsafe { Host::new(test_host.as_clap_host()) };

    let _ = host.get_extension().log().unwrap_err();
}

#[test]
fn host_implements_log_null_callback() {
    let mut test_host = TestHostConfig {
        log_implements: true,
        ..Default::default()
    }
    .build();
    unsafe { test_host.as_mut().get_unchecked_mut().clap_host_log.log = None };

    let host = unsafe { Host::new(test_host.as_clap_host()) };
    let log = host.get_extension().log().unwrap();

    assert_eq!(log.error("").unwrap_err(), Callback);
}

#[test]
fn host_implements_log() {
    let test_host = TestHostConfig {
        log_implements: true,
        ..Default::default()
    }
    .build();

    let host = unsafe { Host::new(test_host.as_clap_host()) };
    let log = host.get_extension().log().unwrap();

    log.warning("this is a warning").unwrap();
}

#[test]
fn host_log_01() {
    let mut buf = Vec::new();
    let test_host = TestHostConfig {
        log_implements: true,
        log_messages: Some(&mut buf),
        ..Default::default()
    }
    .build();

    let host = unsafe { Host::new(test_host.as_clap_host()) };
    let log = host.get_extension().log().unwrap();

    log.warning("this is a warning").unwrap();

    assert_eq!(buf.len(), 1);
    assert_eq!(buf[0], (CLAP_LOG_WARNING, c"this is a warning".to_owned()));
}

#[test]
fn host_log_02() {
    let mut buf = Vec::new();
    let test_host = TestHostConfig {
        log_implements: true,
        log_messages: Some(&mut buf),
        ..Default::default()
    }
    .build();

    let host = unsafe { Host::new(test_host.as_clap_host()) };
    let log = host.get_extension().log().unwrap();

    log.debug("this is a debug").unwrap();
    log.info("this is an info").unwrap();
    log.warning("this is a warning").unwrap();
    log.error("this is an error").unwrap();
    log.fatal("this is a fatal").unwrap();

    assert_eq!(buf.len(), 5);
    assert_eq!(buf[0], (CLAP_LOG_DEBUG, c"this is a debug".to_owned()));
    assert_eq!(buf[1], (CLAP_LOG_INFO, c"this is an info".to_owned()));
    assert_eq!(buf[2], (CLAP_LOG_WARNING, c"this is a warning".to_owned()));
    assert_eq!(buf[3], (CLAP_LOG_ERROR, c"this is an error".to_owned()));
    assert_eq!(buf[4], (CLAP_LOG_FATAL, c"this is a fatal".to_owned()));
}

#[test]
fn host_log_03() {
    let mut buf = Vec::new();
    let test_host = TestHostConfig {
        log_implements: true,
        log_messages: Some(&mut buf),
        ..Default::default()
    }
    .build();

    let host = unsafe { Host::new(test_host.as_clap_host()) };
    let log = host.get_extension().log().unwrap();

    log.log(Severity::ClapHostMisbehaving, "this is host misbehaving")
        .unwrap();
    log.log(
        Severity::ClapPluginMisbehaving,
        "this is plugin misbehaving",
    )
    .unwrap();

    assert_eq!(buf.len(), 2);
    assert_eq!(
        buf[0],
        (
            CLAP_LOG_HOST_MISBEHAVING,
            c"this is host misbehaving".to_owned()
        )
    );
    assert_eq!(
        buf[1],
        (
            CLAP_LOG_PLUGIN_MISBEHAVING,
            c"this is plugin misbehaving".to_owned()
        )
    );
}
