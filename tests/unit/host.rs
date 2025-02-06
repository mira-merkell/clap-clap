use std::{
    ffi::{CString, c_char, c_void},
    pin::Pin,
    ptr::{null, null_mut},
};

use clap_clap::{host::Host, version::CLAP_VERSION};
use clap_sys::clap_host;

pub struct TestHostConfig<'a> {
    pub name: &'a str,
    pub vendor: &'a str,
    pub url: &'a str,
    pub version: &'a str,
}

impl TestHostConfig<'_> {
    pub fn build(self) -> Pin<Box<TestHost>> {
        TestHost::new(self)
    }
}

#[allow(unused)]
pub struct TestHost {
    name: CString,
    vendor: CString,
    url: CString,
    version: CString,

    clap_host: clap_host,
}

impl TestHost {
    fn new(config: TestHostConfig) -> Pin<Box<Self>> {
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

        Box::pin(Self {
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
        })
    }

    pub const fn as_clap_host(&self) -> &clap_host {
        &self.clap_host
    }
}

unsafe impl Send for TestHost {}
unsafe impl Sync for TestHost {}

#[test]
fn host_new() {
    let test_host = TestHostConfig {
        name: "test_host",
        url: "test_url",
        vendor: "test_vendor",
        version: "test_version",
    }
    .build();

    let host = unsafe { Host::new(test_host.as_clap_host()) };

    assert_eq!(host.name(), "test_host");
    assert_eq!(host.url(), "test_url");
    assert_eq!(host.vendor(), "test_vendor");
    assert_eq!(host.version(), "test_version");
}
