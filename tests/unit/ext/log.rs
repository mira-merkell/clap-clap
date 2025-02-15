use std::{ffi::CString, pin::Pin};

use clap_clap::{ext::log::Severity, host::Error::Callback};

use crate::host::{ExtLogConfig, Test, TestBed, TestBedConfig};

#[test]
fn host_doesnt_implement_log() {
    let mut bed = TestBedConfig {
        ext_log: None,
        ..Default::default()
    }
    .build();

    let host = bed.as_mut().host_mut();

    let _ = host.get_extension().log().unwrap_err();
}

#[test]
fn host_implements_log_null_callback() {
    let mut bed = TestBedConfig {
        ext_log: Some(ExtLogConfig {
            null_callback: true,
        }),
        ..Default::default()
    }
    .build();

    let host = bed.as_mut().host_mut();
    assert_eq!(host.get_extension().log().unwrap_err(), Callback("log"));
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

        let host = bed.as_mut().host_mut();
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
    CheckLogMsg {
        severity: Severity::Warning,
        msg: "this is a warning",
    }
    .test(
        TestBedConfig {
            ext_log: Some(ExtLogConfig::default()),
            ..Default::default()
        }
        .build()
        .as_mut(),
    );
}

macro_rules! check_host_log {
    ($name:tt, $severity:ident, $msg:literal) => {
        #[test]
        fn $name() {
            let mut bed = TestBedConfig {
                ext_log: Some(ExtLogConfig::default()),
                ..Default::default()
            }
            .build();

            CheckLogMsg {
                severity: Severity::$severity,
                msg: $msg,
            }
            .test(bed.as_mut());
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
    let mut bed = TestBedConfig {
        ext_log: Some(ExtLogConfig::default()),
        ..Default::default()
    }
    .build();

    CheckLogMsg {
        severity: Severity::ClapHostMisbehaving,
        msg: "this is a host misbehaving",
    }
    .test(bed.as_mut());

    CheckLogMsg {
        severity: Severity::ClapPluginMisbehaving,
        msg: "this is a plugin misbehaving",
    }
    .test(bed.as_mut());
}
