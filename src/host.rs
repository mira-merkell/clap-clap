use std::{
    ffi::{CStr, c_void},
    fmt::{Display, Formatter},
};

use crate::{
    ext::host::{log, log::Log},
    ffi::{CLAP_EXT_LOG, clap_host},
    version::ClapVersion,
};

#[derive(Debug, PartialEq)]
pub struct Host {
    clap_host: *const clap_host,
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
    #[doc(hidden)]
    pub const unsafe fn new(clap_host: *const clap_host) -> Self {
        Self { clap_host }
    }

    pub const fn clap_host(&self) -> &clap_host {
        // SAFETY: by construction, we can obtain a shared reference to clap_host for
        // the lifetime of self.
        unsafe { &*self.clap_host }
    }

    pub const fn clap_version(&self) -> ClapVersion {
        self.clap_host().clap_version
    }

    pub const fn get_extension(&self) -> HostExtensions {
        HostExtensions::new(self)
    }
}

macro_rules! impl_host_get_str {
    ($($description:tt),*) => {
        impl Host {
            $(
                /// # Panic
                ///
                /// This method will panic if the host returns an invalid UTF-8 string.
                pub fn $description(&self) -> &str {
                    unsafe { CStr::from_ptr(self.clap_host().$description)
                                .to_str()
                                .expect("host description must be a valid UTF-8 string")
                    }
                }
            )*
        }
    };
}

impl_host_get_str!(name, vendor, url, version);

macro_rules! impl_host_request {
    ($($request_method:tt),*) => {
        impl Host {
            $(
                pub fn $request_method(&self) {
                    let clap_host = self.clap_host();
                    if let Some(callback) = clap_host.$request_method {
                        // SAFETY: The Host constructor checks if callback is
                        // non-null during the initialization. The pointer is a valid function
                        // obtained from the CLAP host. It is guaranteed be the host that the call
                        // is safe.
                        unsafe { callback(&raw const *self.clap_host()) }
                    }
                }
            )*
        }
    };
}

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
        let callback = self.host.clap_host().get_extension.unwrap();
        // SAFETY: ClapHost::try_new() guarantees that the call is safe.
        let ext_ptr = unsafe { callback(self.host.clap_host(), extension_id.as_ptr()) };
        (!ext_ptr.is_null()).then_some(ext_ptr)
    }

    pub fn log(&self) -> Result<Log<'a>, Error> {
        let clap_host_log = self
            .get_extension_ptr(CLAP_EXT_LOG)
            .ok_or(Error::ExtensionNotFound("log"))?;

        // SAFETY: We just checked if the pointer to clap_host_log is non-null.
        // We return a reference to log for the lifetime of Host.
        Ok(Log::new(self.host, unsafe { &*clap_host_log.cast() }))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    ExtensionNotFound(&'static str),
    Log(log::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::ExtensionNotFound(name) => write!(f, "extension not found: {name}"),
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
