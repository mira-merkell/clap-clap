use std::{
    ffi::{CStr, c_char, c_void},
    ptr::{null, null_mut},
};

use clap_clap::{ffi::clap_host, host::Host, version::CLAP_VERSION};

#[derive(Debug)]
pub struct TestHostConfig<'a> {
    pub name: &'a CStr,
    pub vendor: &'a CStr,
    pub url: &'a CStr,
    pub version: &'a CStr,
}

impl<'a> TestHostConfig<'a> {
    pub fn build(self) -> TestHost<'a> {
        TestHost::new(self)
    }
}

#[derive(Debug)]
#[allow(unused)]
pub struct TestHost<'a> {
    clap_host: clap_host,
    config: TestHostConfig<'a>,
}

impl<'a> TestHost<'a> {
    fn new(config: TestHostConfig<'a>) -> TestHost<'a> {
        extern "C-unwind" fn get_extension(_: *const clap_host, _: *const c_char) -> *const c_void {
            null()
        }
        extern "C-unwind" fn request_restart(_: *const clap_host) {}
        extern "C-unwind" fn request_reset(_: *const clap_host) {}
        extern "C-unwind" fn request_callback(_: *const clap_host) {}

        Self {
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
            config,
        }
    }

    pub const fn as_clap_host(&self) -> &clap_host {
        &self.clap_host
    }
}

unsafe impl<'a> Send for TestHost<'a> {}
unsafe impl<'a> Sync for TestHost<'a> {}

#[test]
fn host_new() {
    let test_host = TestHostConfig {
        name: c"test_host",
        url: c"test_url",
        vendor: c"test_vendor",
        version: c"test_version",
    }
    .build();

    let host = unsafe { Host::new(test_host.as_clap_host()) };

    assert_eq!(host.name(), "test_host");
    assert_eq!(host.url(), "test_url");
    assert_eq!(host.vendor(), "test_vendor");
    assert_eq!(host.version(), "test_version");
}
