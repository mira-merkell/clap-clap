use std::{
    ffi::{CStr, c_void},
    fmt::{Display, Formatter},
    str::Utf8Error,
};

use clap_sys::{CLAP_EXT_LOG, clap_host, clap_host_log};

use crate::{
    ext::{log, log::Log},
    factory::FactoryHost,
    version::ClapVersion,
};

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

#[derive(Debug)]
struct HostDescriptor {
    name: String,
    vendor: String,
    url: String,
    version: String,
}

impl HostDescriptor {
    // Safety:
    // Must be a valid clap_host struct obtained from host via Factory.
    unsafe fn from_clap_host(clap_host: &clap_host) -> Result<Self, Error> {
        if clap_host.name.is_null()
            || clap_host.vendor.is_null()
            || clap_host.url.is_null()
            || clap_host.version.is_null()
        {
            return Err(Error::NullPtr);
        }

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
    clap_host: clap_host,
    descriptor: HostDescriptor,
}

unsafe impl Send for Host {}
unsafe impl Sync for Host {}

impl Host {
    // Safety:
    // The host argument must be a valid pointer (wrapped in FactoryHost)
    // to a CLAP host, obtained as the argument passed to plugin factory's
    // create_plugin().
    //
    // The function returns a Host struct if all host methods are non-null pointers,
    // and the Host description strings are properly validated UTF-8 strings.
    pub(crate) unsafe fn try_from_factory(host: FactoryHost) -> Result<Self, Error> {
        // Safety:
        // The user must uphold the safety requirement about the pointer to clap_host.
        let clap_host = unsafe { *host.into_inner().as_ptr() };
        let descriptor = unsafe { HostDescriptor::from_clap_host(&clap_host) }?;

        if clap_host.get_extension.is_none() {
            return Err(Error::MethodNotFound("get_extension"));
        }
        if clap_host.request_restart.is_none() {
            return Err(Error::MethodNotFound("request_restart"));
        }
        if clap_host.request_process.is_none() {
            return Err(Error::MethodNotFound("request_process"));
        }
        if clap_host.request_callback.is_none() {
            return Err(Error::MethodNotFound("request_callback"));
        }

        Ok(Self {
            descriptor,
            clap_host,
        })
    }

    pub fn clap_version(&self) -> ClapVersion {
        self.clap_host.clap_version
    }

    pub fn get_extension(&self) -> HostExtensions {
        // Safety:
        // The Host constructor guarantees that self.clap_host.get_extension
        // is non-null.
        unsafe { HostExtensions::new(&self.clap_host) }
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
    // Safety:
    // The reference to clap_host must point to a valid clap_host struct,
    // obtained from host via Factory.
    // Additionally, the clap_host.get_extension() method must be a non-null
    // pointer.
    unsafe fn new(clap_host: &'a clap_host) -> Self {
        Self { clap_host }
    }

    fn get_extension_ptr(&self, extension_id: &CStr) -> Option<*const c_void> {
        // HostExtensions::new() guarantees that unwrap won't panic.
        let callback = self.clap_host.get_extension.unwrap();
        // Safety:
        // HostExtension::new() guarantees that the call is safe.
        let ext_ptr = unsafe { callback(&raw const *self.clap_host, extension_id.as_ptr()) };
        (!ext_ptr.is_null()).then_some(ext_ptr)
    }

    pub fn log(self) -> Result<Log<'a>, Error> {
        let clap_host_log = self
            .get_extension_ptr(CLAP_EXT_LOG)
            .ok_or(Error::ExtensionNotFound("log"))?;
        let clap_host_log = clap_host_log as *const clap_host_log;

        // Safety:
        // We just checked if the pointer to clap_host_log is non-null.
        Ok(Log::new(self.clap_host, unsafe { *clap_host_log }))
    }
}
