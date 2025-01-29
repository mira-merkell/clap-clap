use crate::host;
use clap_sys::{
    CLAP_LOG_DEBUG, CLAP_LOG_ERROR, CLAP_LOG_FATAL, CLAP_LOG_HOST_MISBEHAVING, CLAP_LOG_INFO,
    CLAP_LOG_PLUGIN_MISBEHAVING, CLAP_LOG_WARNING, clap_host, clap_host_log, clap_log_severity,
};
use std::ffi::{CString, NulError};

pub struct Log<'a> {
    clap_host: &'a clap_host,
    clap_host_log: &'a clap_host_log,
}

impl<'a> Log<'a> {
    pub(crate) fn new(clap_host: &'a clap_host, clap_host_log: &'a clap_host_log) -> Self {
        Self {
            clap_host,
            clap_host_log,
        }
    }

    pub fn log(&self, severity: Severity, msg: &str) -> Result<(), Error> {
        let msg = CString::new(msg)?;
        let callback = self.clap_host_log.log.ok_or(Error::Callback)?;
        
        // Safety:
        // We just checked if callback is non-null.  The callback is thread-safe,
        // and we own the pointer to msg until the callback returns.
        // So the call is safe.
        unsafe { callback(&raw const *self.clap_host, severity.into(), msg.as_ptr()) };
        Ok(())
    }
}

macro_rules! impl_log_severity {
    ($(($method:tt, $severity:ident)),*) => {
        impl<'a> Log<'a> {
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
    ClapHostMisbehaving,
    ClapPluginMisbehaving,
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
            ClapHostMisbehaving => CLAP_LOG_HOST_MISBEHAVING,
            ClapPluginMisbehaving => CLAP_LOG_PLUGIN_MISBEHAVING,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Error {
    Callback,
    NulError(NulError),
}

impl From<NulError> for Error {
    fn from(value: NulError) -> Self {
        Self::NulError(value)
    }
}

impl From<Error> for crate::Error {
    fn from(value: Error) -> Self {
        host::Error::Log(value).into()
    }
}

