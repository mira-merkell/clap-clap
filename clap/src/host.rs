use crate::{ext::log, ext::log::Log, version::ClapVersion};
use clap_sys::{CLAP_EXT_LOG, clap_host, clap_host_log};
use std::ffi::CStr;
use std::fmt::{Display, Formatter};
use std::ptr::NonNull;

#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    Null,
    Log(log::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Null => write!(f, "method not found"),
            Error::Log(e) => write!(f, "extension 'host_log': {e}"),
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
    clap_host: clap_host,
}

unsafe impl Send for Host {}
unsafe impl Sync for Host {}

impl Host {
    // Safety:
    // The pointer must be a valid pointer to a CLAP host, obtained as
    // the argument passed to plugin factory's create_plugin().
    pub(crate) unsafe fn new(clap_host: NonNull<clap_host>) -> Self {
        Self {
            clap_host: unsafe { *clap_host.as_ptr() },
        }
    }

    pub fn clap_version(&self) -> ClapVersion {
        self.clap_host.clap_version
    }

    pub fn get_extension(&self) -> HostExtensions {
        let clap_host = &self.clap_host;

        clap_host
            .try_into()
            .expect("host.get_extension() should be non-null")
    }
}

macro_rules! impl_host_get_str {
    ($($cstr:tt),*) => {
        impl Host {
            $(
                pub fn $cstr(&self) -> Result<&str, std::str::Utf8Error> {
                   unsafe { CStr::from_ptr(self.clap_host.$cstr) }.to_str()
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
                    if let Some(callback) = self.clap_host.$method {
                        unsafe { callback(&raw const self.clap_host) }
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
            .ok_or(Error::Null)
    }
}

impl<'a> TryFrom<&'a clap_host> for HostExtensions<'a> {
    type Error = Error;

    fn try_from(clap_host: &'a clap_host) -> Result<Self, Self::Error> {
        clap_host
            .get_extension
            .map(|_| Self { clap_host })
            .ok_or(Error::Null)
    }
}
