use std::{ffi::CString, pin::Pin};

use clap_clap::{
    ext::log::Severity,
    host,
    host::Error::{Callback, ExtensionNotFound},
};

use crate::host::{ExtLogConfig, Test, TestBed, TestConfig};

struct CheckImplLog {
    error: host::Error,
}

impl Test for CheckImplLog {
    fn test(self, bed: Pin<&mut TestBed>) {
        let host = unsafe { bed.host_mut() };

        assert_eq!(host.get_extension().log().unwrap_err(), self.error);
    }
}

#[test]
fn host_doesnt_implement_log() {
    TestConfig::default().test(CheckImplLog {
        error: ExtensionNotFound("log"),
    });
}

#[test]
fn host_implements_log_null_callback() {
    TestConfig {
        ext_log: Some(ExtLogConfig {
            null_callback: true,
        }),
        ..Default::default()
    }
    .test(CheckImplLog {
        error: Callback("log"),
    });
}

struct CheckLogMsg<'a> {
    severity: Severity,
    msg: &'a str,
}

impl Test for CheckLogMsg<'_> {
    fn test(self, mut bed: Pin<&mut TestBed>) {
        {
            let mut buf = bed.ext_log.as_ref().unwrap().log_msg.lock().unwrap();
            buf.clear();
        }

        let host = unsafe { bed.as_mut().host_mut() };
        let log = host.get_extension().log().unwrap();

        match self.severity {
            Severity::Debug => log.debug(self.msg),
            Severity::Info => log.info(self.msg),
            Severity::Warning => log.warning(self.msg),
            Severity::Error => log.error(self.msg),
            Severity::Fatal => log.fatal(self.msg),
            _ => log.log(self.severity, self.msg),
        }
        .unwrap();

        let buf = bed.ext_log.as_ref().unwrap().log_msg.lock().unwrap();
        assert_eq!(buf.len(), 1);
        assert_eq!(
            buf[0],
            (self.severity.into(), CString::new(self.msg).unwrap())
        )
    }
}

#[test]
fn host_implements_log() {
    TestConfig {
        ext_log: Some(ExtLogConfig::default()),
        ..Default::default()
    }
    .test(CheckLogMsg {
        severity: Severity::Warning,
        msg: "this is a warning",
    });
}

macro_rules! check_host_log {
    ($name:tt, $severity:ident, $msg:literal) => {
        #[test]
        fn $name() {
            TestConfig {
                ext_log: Some(ExtLogConfig::default()),
                ..Default::default()
            }
            .test(CheckLogMsg {
                severity: Severity::$severity,
                msg: $msg,
            });
        }
    };
}

check_host_log!(host_log_debug, Debug, "this as a debug");
check_host_log!(host_log_warning, Warning, "this as a warning");
check_host_log!(host_log_info, Info, "this as an info");
check_host_log!(host_log_error, Error, "this as an error");
check_host_log!(host_log_fatal, Fatal, "this as a fatal");

#[test]
fn host_log_misbehaving() {
    TestConfig {
        ext_log: Some(ExtLogConfig::default()),
        ..Default::default()
    }
    .test(CheckLogMsg {
        severity: Severity::ClapHostMisbehaving,
        msg: "this is a host misbehaving",
    })
    .test(CheckLogMsg {
        severity: Severity::ClapPluginMisbehaving,
        msg: "this is a plugin misbehaving",
    });
}
