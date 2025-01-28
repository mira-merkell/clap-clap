use crate::host;
use clap_sys::{CLAP_LOG_INFO, clap_host, clap_host_log, clap_log_severity};
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
        unsafe { callback(&raw const *self.clap_host, severity.into(), msg.as_ptr()) };
        Ok(())
    }

    pub fn info(&self, msg: &str) -> Result<(), Error> {
        self.log(Severity::Info, msg)
    }
}

pub enum Severity {
    Info,
}

impl From<Severity> for clap_log_severity {
    fn from(value: Severity) -> Self {
        match value {
            Severity::Info => CLAP_LOG_INFO,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Error {
    Callback,
    NulError(NulError),
}

impl From<Error> for crate::Error {
    fn from(value: Error) -> Self {
        host::Error::Log(value).into()
    }
}

impl From<NulError> for Error {
    fn from(value: NulError) -> Self {
        Self::NulError(value)
    }
}
