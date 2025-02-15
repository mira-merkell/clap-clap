use std::{
    ffi::{CStr, CString, c_char, c_void},
    marker::PhantomPinned,
    pin::Pin,
    ptr::{null, null_mut},
    sync::Mutex,
};

use clap_clap::{
    ffi::{
        CLAP_EXT_AUDIO_PORTS, CLAP_EXT_LOG, clap_host, clap_host_audio_ports, clap_host_log,
        clap_log_severity,
    },
    host::Host,
    version::CLAP_VERSION,
};

pub trait Test {
    fn test(self, bed: Pin<&mut TestBed>);
}

#[derive(Debug, Default, Copy, Clone)]
pub struct TestBedConfig<'a> {
    pub name: &'a CStr,
    pub vendor: &'a CStr,
    pub url: &'a CStr,
    pub version: &'a CStr,

    pub ext_log: Option<ExtLogConfig>,
    pub ext_audio_ports: Option<ExtAudioPortsConfig>,
}

impl<'a> TestBedConfig<'a> {
    pub fn build(self) -> Pin<Box<TestBed<'a>>> {
        TestBed::new(self)
    }
}

#[derive(Debug)]
pub struct TestBed<'a> {
    config: TestBedConfig<'a>,
    clap_host: clap_host,
    host: Option<Host>,

    pub ext_audio_ports: Option<ExtAudioPorts>,
    pub ext_log: Option<ExtLog>,

    _marker: PhantomPinned,
}

impl<'a> TestBed<'a> {
    pub fn new(config: TestBedConfig<'a>) -> Pin<Box<Self>> {
        let mut bed = Box::new(Self {
            host: None,
            clap_host: clap_host {
                clap_version: CLAP_VERSION,
                host_data: null_mut(),
                name: config.name.as_ptr(),
                vendor: config.vendor.as_ptr(),
                url: config.url.as_ptr(),
                version: config.version.as_ptr(),
                get_extension: Some(get_extension),
                request_restart: Some(request_restart),
                request_process: Some(request_reset),
                request_callback: Some(request_callback),
            },
            ext_audio_ports: config.ext_audio_ports.map(ExtAudioPorts::new),
            ext_log: config.ext_log.map(ExtLog::new),

            config,
            _marker: PhantomPinned,
        });

        // Self-referential fields.
        bed.clap_host.host_data = (&raw mut *bed).cast();
        bed.host = Some(unsafe { Host::new(&raw mut bed.clap_host) });

        Box::into_pin(bed)
    }

    pub const fn host_mut(self: Pin<&mut Self>) -> &mut Host {
        unsafe { self.get_unchecked_mut().host.as_mut().unwrap() }
    }
}

unsafe impl Send for TestBed<'_> {}
unsafe impl Sync for TestBed<'_> {}

extern "C-unwind" fn get_extension(
    host: *const clap_host,
    extension_id: *const c_char,
) -> *const c_void {
    assert!(!host.is_null());
    let test_host: &TestBed = unsafe { &*(*host).host_data.cast() };
    let extension_id = unsafe { CStr::from_ptr(extension_id) };

    if extension_id == CLAP_EXT_AUDIO_PORTS {
        if let Some(ext) = &test_host.ext_audio_ports {
            return (&raw const ext.clap_host_audio_ports).cast();
        }
    }
    if extension_id == CLAP_EXT_LOG {
        if let Some(ext) = &test_host.ext_log {
            return (&raw const ext.clap_host_log).cast();
        }
    }

    null()
}

extern "C-unwind" fn request_restart(_: *const clap_host) {}
extern "C-unwind" fn request_reset(_: *const clap_host) {}
extern "C-unwind" fn request_callback(_: *const clap_host) {}

#[derive(Debug, Default, Copy, Clone)]
pub struct ExtAudioPortsConfig {
    pub supported_flags: u32,
    pub null_is_rescan_flag_supported: bool,
    pub null_rescan: bool,
}

#[derive(Debug)]
pub struct ExtAudioPorts {
    config: ExtAudioPortsConfig,
    pub clap_host_audio_ports: clap_host_audio_ports,
    pub call_rescan_flags: u32,
}

impl ExtAudioPorts {
    fn new(config: ExtAudioPortsConfig) -> Self {
        Self {
            config,
            clap_host_audio_ports: clap_host_audio_ports {
                is_rescan_flag_supported: (!config.null_is_rescan_flag_supported)
                    .then_some(ext_audio_ports_is_rescan_flag_supported),
                rescan: (!config.null_rescan).then_some(ext_audio_ports_rescan),
            },
            call_rescan_flags: 0,
        }
    }
}

extern "C-unwind" fn ext_audio_ports_is_rescan_flag_supported(
    host: *const clap_host,
    flag: u32,
) -> bool {
    assert!(!host.is_null());
    let bed: &mut TestBed = unsafe { &mut *(*host).host_data.cast() };
    if let Some(ext) = &bed.ext_audio_ports {
        ext.config.supported_flags & flag != 0
    } else {
        false
    }
}

extern "C-unwind" fn ext_audio_ports_rescan(host: *const clap_host, flags: u32) {
    assert!(!host.is_null());
    let bed: &mut TestBed = unsafe { &mut *(*host).host_data.cast() };
    if let Some(ext) = &mut bed.ext_audio_ports {
        ext.call_rescan_flags |= flags;
    }
}

#[derive(Debug, Default, Copy, Clone)]
pub struct ExtLogConfig {
    pub null_callback: bool,
}

#[derive(Debug)]
pub struct ExtLog {
    clap_host_log: clap_host_log,
    pub log_msg: Mutex<Vec<(clap_log_severity, CString)>>,
}

impl ExtLog {
    fn new(config: ExtLogConfig) -> Self {
        Self {
            clap_host_log: clap_host_log {
                log: (!config.null_callback).then_some(ext_log_log),
            },
            log_msg: Mutex::new(Vec::new()),
        }
    }
}

extern "C-unwind" fn ext_log_log(
    host: *const clap_host,
    severity: clap_log_severity,
    msg: *const c_char,
) {
    assert!(!host.is_null());
    let bed: &mut TestBed = unsafe { &mut *(*host).host_data.cast() };

    assert!(!msg.is_null());
    let msg = unsafe { CStr::from_ptr(msg) }.to_owned();

    if let Some(ext) = &bed.ext_log {
        let mut buf = ext.log_msg.lock().unwrap();
        buf.push((severity, msg))
    }
}

struct CheckDescription;

impl Test for CheckDescription {
    fn test(self, bed: Pin<&mut TestBed>) {
        let name = bed.config.name.to_str().unwrap();
        let vendor = bed.config.vendor.to_str().unwrap();
        let url = bed.config.url.to_str().unwrap();
        let version = bed.config.version.to_str().unwrap();

        let host = bed.host_mut();

        assert_eq!(host.name(), name);
        assert_eq!(host.vendor(), vendor);
        assert_eq!(host.url(), url);
        assert_eq!(host.version(), version);
    }
}

#[test]
fn check_description_01() {
    CheckDescription {}.test(TestBedConfig::default().build().as_mut());
}

#[test]
fn check_description_02() {
    CheckDescription {}.test(
        TestBedConfig {
            name: c"test_name",
            vendor: c"test_vendor",
            url: c"test_url",
            version: c"test_version",
            ..Default::default()
        }
        .build()
        .as_mut(),
    );
}
