use crate::{ext::log, ext::log::Log, version::ClapVersion};
use clap_sys::{clap_host, clap_host_log, CLAP_EXT_LOG};
use std::ffi::CStr;
use std::fmt::{Display, Formatter};
use std::ptr::NonNull;

#[derive(Debug, Clone)]
pub enum Error {
    GetExtensions,
    Log(log::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::GetExtensions => f.write_str("method 'get_extensions()' not found"),
            Error::Log(e) => write!(f, "Host extension (Log): {}", e),
        }
    }
}

impl std::error::Error for Error {}

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

    pub fn get_extension(&self) -> HostExtensions {
        let clap_host = unsafe { &*self.clap_host.as_ptr() };
        clap_host
            .try_into()
            .expect("host.get_extension() should be non-null")
    }
}

macro_rules! impl_host_get_str {
    ($($method:tt),*) => {
        impl Host {
            $(
                pub fn $method(&self) -> Result<&str, std::str::Utf8Error> {
                   let clap_host = unsafe { &*self.clap_host.as_ptr() };
                   unsafe { CStr::from_ptr(clap_host.$method) }.to_str()
                }
            )*
        }
    };
}

macro_rules! impl_host_request {
    ($($method:tt),*) => {
        impl Host {
            $(
                pub fn $method(&self) {
                    let clap_host = unsafe { &*self.clap_host.as_ptr() };
                    if let Some(callback) = clap_host.$method {
                        unsafe { callback(self.clap_host.as_ptr()) }
                    }
                }
            )*
        }
    };
}

impl_host_get_str!(name, vendor, url, version);
impl_host_request!(request_process, request_restart, request_callback);

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
