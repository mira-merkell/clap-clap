use std::{
    ffi::{CStr, c_void},
    fmt::{Display, Formatter},
    str::Utf8Error,
};

use clap_sys::{CLAP_EXT_LOG, clap_host, clap_host_log};
use log::Log;

use crate::version::ClapVersion;

pub mod log;

#[derive(Debug)]
struct ClapHost(*const clap_host);

/// A safe wrapper around the pointer to `clap_host` obtained from host.
impl ClapHost {
    /// Wrap a pointer to host.
    ///
    /// # Safety
    ///
    /// 1. The pointer must be non-null.
    /// 2. The pointer to host must point to a valid clap_host structure
    ///    obtained from CLAP host.
    const unsafe fn new(host: *const clap_host) -> Self {
        Self(host)
    }

    const fn as_ref(&self) -> &clap_host {
        //  SAFETY: by construction, we can obtain a shared reference to clap_host for
        // the lifetime of self.
        unsafe { &*self.0 }
    }
}

// SAFETY: ClapHost is a wrapper around `*const` as represents a constant
// data structure that can be sent and referenced in a multithreaded context.
unsafe impl Send for ClapHost {}
unsafe impl Sync for ClapHost {}

#[derive(Debug)]
struct HostDescriptor {
    name: String,
    vendor: String,
    url: String,
    version: String,
}

impl HostDescriptor {
    fn from_clap_host(clap_host: &ClapHost) -> Result<Self, Error> {
        // SAFETY: clap_host points to a valid host with all struct fields non-null.
        Ok(Self {
            name: String::from(unsafe { CStr::from_ptr(clap_host.as_ref().name) }.to_str()?),
            vendor: String::from(unsafe { CStr::from_ptr(clap_host.as_ref().vendor) }.to_str()?),
            url: String::from(unsafe { CStr::from_ptr(clap_host.as_ref().url) }.to_str()?),
            version: String::from(unsafe { CStr::from_ptr(clap_host.as_ref().version) }.to_str()?),
        })
    }
}

#[derive(Debug)]
pub struct Host {
    clap_host: ClapHost,
    descriptor: HostDescriptor,
}

impl Host {
    /// # Safety
    ///
    /// The host argument must be a non-null pointer to a CLAP host, obtained as
    /// the argument passed to factory create_plugin(). In particular, all
    /// fields, except for host_data, of clap_host struct must be valid
    /// pointers.
    ///
    /// # Panic
    ///
    /// The function will panic if host description strings aren't properly
    /// validated UTF-8 strings.
    pub(crate) unsafe fn new(host: *const clap_host) -> Self {
        let clap_host = unsafe { ClapHost::new(host) };
        Self {
            descriptor: HostDescriptor::from_clap_host(&clap_host).expect("host descriptor"),
            clap_host,
        }
    }

    pub const fn clap_version(&self) -> ClapVersion {
        self.clap_host.as_ref().clap_version
    }

    pub const fn get_extension(&self) -> HostExtensions {
        HostExtensions::new(self)
    }

    const fn as_clap_host(&self) -> &ClapHost {
        &self.clap_host
    }
}

macro_rules! impl_host_get_str {
    ($($description:tt),*) => {
        impl Host {
            $(
                pub fn $description(&self) -> &str {
                    &self.descriptor.$description
                }
            )*
        }
    };
}

macro_rules! impl_host_request {
    ($($request_method:tt),*) => {
        impl Host {
            $(
                pub fn $request_method(&self) {
                    let clap_host = self.clap_host.as_ref();
                    if let Some(callback) = clap_host.$request_method {
                        // SAFETY: The Host constructor checks if callback is
                        // non-null during the initialization. The pointer is a valid function
                        // obtained from the CLAP host. It is guaranteed be the host that the call
                        // is safe.
                        unsafe { callback(&raw const *self.clap_host.as_ref()) }
                    }
                }
            )*
        }
    };
}

impl_host_get_str!(name, vendor, url, version);
impl_host_request!(request_process, request_restart, request_callback);

pub struct HostExtensions<'a> {
    host: &'a Host,
}

impl<'a> HostExtensions<'a> {
    const fn new(host: &'a Host) -> Self {
        Self { host }
    }

    fn get_extension_ptr(&self, extension_id: &CStr) -> Option<*const c_void> {
        // HostExtensions::new() guarantees that unwrap won't panic.
        let callback = self.host.clap_host.as_ref().get_extension.unwrap();
        // SAFETY: ClapHost::try_new() guarantees that the call is safe.
        let ext_ptr = unsafe {
            callback(
                &raw const *self.host.clap_host.as_ref(),
                extension_id.as_ptr(),
            )
        };
        (!ext_ptr.is_null()).then_some(ext_ptr)
    }

    pub fn log(&self) -> Result<Log<'a>, Error> {
        let clap_host_log = self
            .get_extension_ptr(CLAP_EXT_LOG)
            .ok_or(Error::ExtensionNotFound("log"))?;
        let clap_host_log = clap_host_log as *const clap_host_log;

        // SAFETY: We just checked if the pointer to clap_host_log is non-null.
        Ok(unsafe { Log::new(self.host, clap_host_log) })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    Utf8Conversion(Utf8Error),
    ExtensionNotFound(&'static str),
    Log(log::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Utf8Conversion(e) => write!(f, "error while converting C string: {e}"),
            Error::ExtensionNotFound(name) => write!(f, "extension not found: {name}"),
            Error::Log(e) => write!(f, "extension 'host_log': {e}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<Utf8Error> for Error {
    fn from(value: Utf8Error) -> Self {
        Self::Utf8Conversion(value)
    }
}

impl From<Error> for crate::Error {
    fn from(value: Error) -> Self {
        Self::Host(value)
    }
}
