use std::{
    ffi::{CStr, c_void},
    fmt::{Display, Formatter},
    ops::Deref,
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
    /// This function checks if host and all host struct's fields are non-null
    /// pointers (except the private host_data).
    ///
    /// # Safety
    ///
    /// The pointer to host must point to a valid clap_host structure obtained
    /// from CLAP host.
    unsafe fn try_new(host: *const clap_host) -> Result<Self, Error> {
        if host.is_null() {
            return Err(Error::NullPtr);
        };
        // The pointer is convertible to a reference, because it points
        // to a genuine clap_host structure obtained from the host.
        let host_ref = unsafe { &*host };

        if host_ref.name.is_null()
            || host_ref.vendor.is_null()
            || host_ref.url.is_null()
            || host_ref.version.is_null()
        {
            return Err(Error::NullPtr);
        }

        if host_ref.get_extension.is_none() {
            return Err(Error::MethodNotFound("get_extension"));
        }
        if host_ref.request_restart.is_none() {
            return Err(Error::MethodNotFound("request_restart"));
        }
        if host_ref.request_process.is_none() {
            return Err(Error::MethodNotFound("request_process"));
        }
        if host_ref.request_callback.is_none() {
            return Err(Error::MethodNotFound("request_callback"));
        }

        Ok(Self(host))
    }
}

impl Deref for ClapHost {
    type Target = clap_host;

    fn deref(&self) -> &Self::Target {
        // Safety:
        // self.0 is a pointer to a constant struct obtained from CLAP
        // host, which is well aligned and convertible to a reference.
        unsafe { &*self.0 }
    }
}

// Safety:
// ClapHost is a wrapper around `*const` as represents a constant
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
        // Safety:
        // clap_host points to a valid host with all struct fields non-null.
        Ok(Self {
            name: String::from(unsafe { CStr::from_ptr(clap_host.name) }.to_str()?),
            vendor: String::from(unsafe { CStr::from_ptr(clap_host.vendor) }.to_str()?),
            url: String::from(unsafe { CStr::from_ptr(clap_host.url) }.to_str()?),
            version: String::from(unsafe { CStr::from_ptr(clap_host.version) }.to_str()?),
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
    /// The host argument must be a valid pointer
    /// to a CLAP host, obtained as the argument passed to plugin factory's
    /// create_plugin().
    ///
    /// The function returns a Host struct if all host methods are non-null
    /// pointers, and the Host description strings are properly validated
    /// UTF-8 strings.
    pub(crate) unsafe fn try_from_factory(host: *const clap_host) -> Result<Self, Error> {
        let clap_host = unsafe { ClapHost::try_new(host) }?;
        Ok(Self {
            descriptor: HostDescriptor::from_clap_host(&clap_host)?,
            clap_host,
        })
    }

    pub fn clap_version(&self) -> ClapVersion {
        self.clap_host.clap_version
    }

    pub fn get_extension(&self) -> HostExtensions {
        HostExtensions::new(self)
    }

    fn as_clap_host(&self) -> &ClapHost {
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
                    if let Some(callback) = self.clap_host.$request_method {
                        // Safety:
                        // The Host constructor checks if callback is non-null during the initialization.
                        // The pointer is a valid function obtained from the CLAP host.
                        // It is guaranteed be the host that the call is safe.
                        unsafe { callback(&raw const *self.clap_host) }
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
    fn new(host: &'a Host) -> Self {
        Self { host }
    }

    fn get_extension_ptr(&self, extension_id: &CStr) -> Option<*const c_void> {
        // HostExtensions::new() guarantees that unwrap won't panic.
        let callback = self.host.clap_host.get_extension.unwrap();
        // Safety:
        // ClapHost::try_new() guarantees that the call is safe.
        let ext_ptr = unsafe { callback(&raw const *self.host.clap_host, extension_id.as_ptr()) };
        (!ext_ptr.is_null()).then_some(ext_ptr)
    }

    pub fn log(&self) -> Result<Log<'a>, Error> {
        let clap_host_log = self
            .get_extension_ptr(CLAP_EXT_LOG)
            .ok_or(Error::ExtensionNotFound("log"))?;
        let clap_host_log = clap_host_log as *const clap_host_log;

        // Safety:
        // We just checked if the pointer to clap_host_log is non-null.
        Ok(unsafe { Log::new(self.host, clap_host_log) })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    NullPtr,
    Utf8Conversion(Utf8Error),
    MethodNotFound(&'static str),
    ExtensionNotFound(&'static str),
    Log(log::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::NullPtr => write!(f, "null pointer"),
            Error::Utf8Conversion(e) => write!(f, "error while converting C string: {e}"),
            Error::MethodNotFound(name) => write!(f, "method not found: {name}"),
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
