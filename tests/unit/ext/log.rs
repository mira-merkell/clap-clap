use clap_clap::{
    ext::log::Severity,
    ffi::{
        CLAP_LOG_DEBUG, CLAP_LOG_ERROR, CLAP_LOG_FATAL, CLAP_LOG_HOST_MISBEHAVING, CLAP_LOG_INFO,
        CLAP_LOG_PLUGIN_MISBEHAVING, CLAP_LOG_WARNING,
    },
    host::{Error::Callback, Host},
};

use crate::host::TestHostConfig;

#[test]
fn host_doesnt_implement_log() {
    let test_host = TestHostConfig {
        impl_ext_log: false,
        ..Default::default()
    }
    .build();

    let host = unsafe { Host::new(test_host.clap_host()) };

    let _ = host.get_extension().log().unwrap_err();
}

#[test]
fn host_implements_log_null_callback() {
    let mut test_host = TestHostConfig {
        impl_ext_log: true,
        ..Default::default()
    }
    .build();
    unsafe { test_host.as_mut().get_unchecked_mut().clap_host_log.log = None };

    let host = unsafe { Host::new(test_host.clap_host()) };
    assert_eq!(host.get_extension().log().unwrap_err(), Callback("log"));
}

#[test]
fn host_implements_log() {
    let test_host = TestHostConfig {
        impl_ext_log: true,
        ..Default::default()
    }
    .build();

    let host = unsafe { Host::new(test_host.clap_host()) };
    let log = host.get_extension().log().unwrap();

    log.warning("this is a warning").unwrap();
}

#[test]
fn host_log_01() {
    let test_host = TestHostConfig {
        impl_ext_log: true,
        ..Default::default()
    }
    .build();

    let host = unsafe { Host::new(test_host.clap_host()) };
    let log = host.get_extension().log().unwrap();

    log.warning("this is a warning").unwrap();

    let buf = test_host.ext_log_messages.lock().unwrap();
    assert_eq!(buf.len(), 1);
    assert_eq!(buf[0], (CLAP_LOG_WARNING, c"this is a warning".to_owned()));
}

#[test]
fn host_log_02() {
    let test_host = TestHostConfig {
        impl_ext_log: true,
        ..Default::default()
    }
    .build();

    let host = unsafe { Host::new(test_host.clap_host()) };
    let log = host.get_extension().log().unwrap();

    log.debug("this is a debug").unwrap();
    log.info("this is an info").unwrap();
    log.warning("this is a warning").unwrap();
    log.error("this is an error").unwrap();
    log.fatal("this is a fatal").unwrap();

    let buf = test_host.ext_log_messages.lock().unwrap();
    assert_eq!(buf.len(), 5);
    assert_eq!(buf[0], (CLAP_LOG_DEBUG, c"this is a debug".to_owned()));
    assert_eq!(buf[1], (CLAP_LOG_INFO, c"this is an info".to_owned()));
    assert_eq!(buf[2], (CLAP_LOG_WARNING, c"this is a warning".to_owned()));
    assert_eq!(buf[3], (CLAP_LOG_ERROR, c"this is an error".to_owned()));
    assert_eq!(buf[4], (CLAP_LOG_FATAL, c"this is a fatal".to_owned()));
}

#[test]
fn host_log_03() {
    let test_host = TestHostConfig {
        impl_ext_log: true,
        ..Default::default()
    }
    .build();

    let host = unsafe { Host::new(test_host.clap_host()) };
    let log = host.get_extension().log().unwrap();

    log.log(Severity::ClapHostMisbehaving, "this is host misbehaving")
        .unwrap();
    log.log(
        Severity::ClapPluginMisbehaving,
        "this is plugin misbehaving",
    )
    .unwrap();

    let buf = test_host.ext_log_messages.lock().unwrap();
    assert_eq!(buf.len(), 2);
    assert_eq!(
        buf[0],
        (
            CLAP_LOG_HOST_MISBEHAVING,
            c"this is host misbehaving".to_owned()
        )
    );
    assert_eq!(
        buf[1],
        (
            CLAP_LOG_PLUGIN_MISBEHAVING,
            c"this is plugin misbehaving".to_owned()
        )
    );
}
