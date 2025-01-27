use crate::version::ClapVersion;
use clap_sys::{CLAP_EXT_LOG, clap_host, clap_host_log};
use std::ffi::CStr;
use std::ptr::NonNull;

pub struct Host {
    clap_host: NonNull<clap_host>,
}

unsafe impl Send for Host {}
unsafe impl Sync for Host {}

impl Host {
    pub(crate) fn new(clap_host: NonNull<clap_host>) -> Self {
        Self { clap_host }
    }

    pub fn clap_version(&self) -> ClapVersion {
        unsafe { &*self.clap_host.as_ptr() }.clap_version
    }

    pub fn name(&self) -> Result<&str, std::str::Utf8Error> {
        let clap_host = unsafe { &*self.clap_host.as_ptr() };
        unsafe { CStr::from_ptr(clap_host.name) }.to_str()
    }

    pub fn vendor(&self) -> Result<&str, std::str::Utf8Error> {
        let clap_host = unsafe { &*self.clap_host.as_ptr() };
        unsafe { CStr::from_ptr(clap_host.vendor) }.to_str()
    }

    pub fn url(&self) -> Result<&str, std::str::Utf8Error> {
        let clap_host = unsafe { &*self.clap_host.as_ptr() };
        unsafe { CStr::from_ptr(clap_host.url) }.to_str()
    }

    pub fn version(&self) -> Result<&str, std::str::Utf8Error> {
        let clap_host = unsafe { &*self.clap_host.as_ptr() };
        unsafe { CStr::from_ptr(clap_host.version) }.to_str()
    }

    pub fn request_restart(&self) {
        let clap_host = unsafe { &*self.clap_host.as_ptr() };
        if let Some(callback) = clap_host.request_restart {
            unsafe { callback(self.clap_host.as_ptr()) }
        }
    }

    pub fn request_process(&self) {
        let clap_host = unsafe { &*self.clap_host.as_ptr() };
        if let Some(callback) = clap_host.request_process {
            unsafe { callback(self.clap_host.as_ptr()) }
        }
    }

    pub fn request_callback(&self) {
        let clap_host = unsafe { &*self.clap_host.as_ptr() };
        if let Some(callback) = clap_host.request_callback {
            unsafe { callback(self.clap_host.as_ptr()) }
        }
    }

    pub fn get_extension(&self) -> Option<HostExtensions> {
        let clap_host = unsafe { &*self.clap_host.as_ptr() };
        clap_host
            .get_extension
            .map(|_| HostExtensions::new(clap_host))
    }
}

pub struct HostExtensions<'a> {
    clap_host: &'a clap_host,
}

impl<'a> HostExtensions<'a> {
    fn new(clap_host: &'a clap_host) -> Self {
        Self { clap_host }
    }

    pub fn log(self) -> Option<Log<'a>> {
        let query = self.clap_host.get_extension.unwrap();
        let clap_host_log = unsafe {
            query(&raw const *self.clap_host, CLAP_EXT_LOG.as_ptr()) as *const clap_host_log
        };
        (!clap_host_log.is_null()).then_some(Log::new(unsafe { &*clap_host_log }))
    }
}

pub struct Log<'a> {
    clap_host_log: &'a clap_host_log,
}

impl<'a> Log<'a> {
    fn new(clap_host_log: &'a clap_host_log) -> Self {
        Self { clap_host_log }
    }

    fn log(&self, severity: Severity, msg: &str) -> Result<(), Error> {
        todo!()
    }
}

pub enum Severity {
    Info,
}

pub enum Error {
    Log,
}
