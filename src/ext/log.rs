use std::{
    ffi::{CStr, CString, NulError},
    fmt::{Display, Formatter},
};

use crate::{
    ffi::{
        CLAP_LOG_DEBUG, CLAP_LOG_ERROR, CLAP_LOG_FATAL, CLAP_LOG_HOST_MISBEHAVING, CLAP_LOG_INFO,
        CLAP_LOG_PLUGIN_MISBEHAVING, CLAP_LOG_WARNING, clap_host_log, clap_log_severity,
    },
    host::Host,
};

#[derive(Debug)]
pub struct HostLog<'a> {
    host: &'a Host,
    clap_host_log: &'a clap_host_log,
}

impl<'a> HostLog<'a> {
    /// # Safety
    ///
    /// All extension interface function pointers must be non-null (Some), and
    /// the functions must be thread-safe.
    pub(crate) const unsafe fn new_unchecked(
        host: &'a Host,
        clap_host_log: &'a clap_host_log,
    ) -> Self {
        Self {
            host,
            clap_host_log,
        }
    }

    /// Send a `CStr` to the host's log.  
    ///
    /// By logging a `CStr`, instead of `&str`, the function avoids memory
    /// allocation, and a fallible Rust-string-to-C-string conversion.
    pub fn log_cstr(&self, severity: Severity, msg: &CStr) {
        // SAFETY: By construction, the callback must be a valid function pointer,
        // and the call is thread-safe.
        let callback = self.clap_host_log.log.unwrap();
        unsafe { callback(self.host.clap_host(), severity.into(), msg.as_ptr()) }
    }

    pub fn log(&self, severity: Severity, msg: &str) -> Result<(), Error> {
        self.log_cstr(severity, &CString::new(msg)?);
        Ok(())
    }
}

macro_rules! impl_log_severity {
    ($(($method:tt, $severity:ident)),*) => {
        impl<'a> HostLog<'a> {
            $(
                pub fn $method(&self, msg: &str) -> Result<(), Error> {
                    self.log(Severity::$severity, msg)
                }
            )*
        }
    };
}

impl_log_severity!(
    (debug, Debug),
    (info, Info),
    (warning, Warning),
    (error, Error),
    (fatal, Fatal)
);

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Severity {
    Debug,
    Info,
    Warning,
    Error,
    Fatal,
    HostMisbehaving,
    PluginMisbehaving,
}

impl From<Severity> for clap_log_severity {
    fn from(value: Severity) -> Self {
        use Severity::*;

        match value {
            Debug => CLAP_LOG_DEBUG,
            Info => CLAP_LOG_INFO,
            Warning => CLAP_LOG_WARNING,
            Error => CLAP_LOG_ERROR,
            Fatal => CLAP_LOG_FATAL,
            HostMisbehaving => CLAP_LOG_HOST_MISBEHAVING,
            PluginMisbehaving => CLAP_LOG_PLUGIN_MISBEHAVING,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    NulError(NulError),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::NulError(e) => write!(f, "error converting to C string: {e}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<NulError> for Error {
    fn from(value: NulError) -> Self {
        Self::NulError(value)
    }
}

impl From<Error> for crate::Error {
    fn from(value: Error) -> Self {
        crate::ext::Error::Log(value).into()
    }
}
