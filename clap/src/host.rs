use crate::host::log::Log;
use crate::version::ClapVersion;
use clap_sys::{CLAP_EXT_LOG, clap_host, clap_host_log};
use std::ffi::CStr;
use std::ptr::NonNull;

#[derive(Debug, Clone)]
pub enum Error {
    GetExtensions,
    Log(log::Error),
}

impl From<Error> for crate::Error {
    fn from(value: Error) -> Self {
        Self::Host(value)
    }
}

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

    pub fn get_extension(&self) -> HostExtensions {
        let clap_host = unsafe { &*self.clap_host.as_ptr() };
        clap_host
            .try_into()
            .expect("host.get_extension() should be non-null")
    }
}

pub struct HostExtensions<'a> {
    clap_host: &'a clap_host,
}

impl<'a> HostExtensions<'a> {
    pub fn log(self) -> Result<Log<'a>, Error> {
        let query = self.clap_host.get_extension.unwrap();
        let clap_host_log = unsafe {
            query(&raw const *self.clap_host, CLAP_EXT_LOG.as_ptr()) as *const clap_host_log
        };
        (!clap_host_log.is_null())
            .then_some(Log::new(self.clap_host, unsafe { &*clap_host_log }))
            .ok_or(Error::GetExtensions)
    }
}

impl<'a> TryFrom<&'a clap_host> for HostExtensions<'a> {
    type Error = Error;

    fn try_from(clap_host: &'a clap_host) -> Result<Self, Self::Error> {
        clap_host
            .get_extension
            .map(|_| Self { clap_host })
            .ok_or(Error::GetExtensions)
    }
}

pub mod log;
