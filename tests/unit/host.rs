use std::{
    ffi::{CStr, CString, c_char, c_void},
    marker::PhantomPinned,
    pin::Pin,
    ptr::{null, null_mut},
    sync::Mutex,
};

use clap_clap::{
    ffi::{
        CLAP_EXT_AUDIO_PORTS, CLAP_EXT_LATENCY, CLAP_EXT_LOG, CLAP_EXT_NOTE_PORTS, CLAP_EXT_PARAMS,
        CLAP_EXT_STATE, CLAP_EXT_TAIL, clap_host, clap_host_audio_ports, clap_host_latency,
        clap_host_log, clap_host_note_ports, clap_host_params, clap_host_state, clap_host_tail,
        clap_id, clap_log_severity,
    },
    host::Host,
    version::CLAP_VERSION,
};

pub trait Test {
    fn test(self, bed: Pin<&mut TestBed>);
}

#[derive(Debug, Default, Copy, Clone)]
pub struct TestConfig<'a> {
    pub name: &'a CStr,
    pub vendor: &'a CStr,
    pub url: &'a CStr,
    pub version: &'a CStr,

    pub ext_audio_ports: Option<ExtAudioPortsConfig>,
    pub ext_latency: Option<ExtLatencyConfig>,
    pub ext_log: Option<ExtLogConfig>,
    pub ext_note_ports: Option<ExtNotePortsConfig>,
    pub ext_params: Option<ExtParamsConfig>,
    pub ext_state: Option<ExtStateConfig>,
    pub ext_tail: Option<ExtTailConfig>,
}

impl TestConfig<'_> {
    pub fn test(self, case: impl Test) -> Self {
        TestBed::new(self).as_mut().test(case);
        self
    }
}

#[derive(Debug, Default, PartialEq)]
struct CallRequest {
    restart: bool,
    process: bool,
    callback: bool,
}

#[derive(Debug)]
pub struct TestBed<'a> {
    config: TestConfig<'a>,
    clap_host: clap_host,
    host: Option<Host>,

    call_request: CallRequest,

    pub ext_audio_ports: Option<ExtAudioPorts>,
    pub ext_log: Option<ExtLog>,
    pub ext_latency: Option<ExtLatency>,
    pub ext_note_ports: Option<ExtNotePorts>,
    pub ext_params: Option<ExtParams>,
    pub ext_state: Option<ExtState>,
    pub ext_tail: Option<ExtTail>,

    _marker: PhantomPinned,
}

impl<'a> TestBed<'a> {
    pub fn new(config: TestConfig<'a>) -> Pin<Box<Self>> {
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
                request_process: Some(request_process),
                request_callback: Some(request_callback),
            },
            call_request: CallRequest::default(),

            ext_audio_ports: config.ext_audio_ports.map(ExtAudioPorts::new),
            ext_latency: config.ext_latency.map(ExtLatency::new),
            ext_log: config.ext_log.map(ExtLog::new),
            ext_note_ports: config.ext_note_ports.map(ExtNotePorts::new),
            ext_params: config.ext_params.map(ExtParams::new),
            ext_state: config.ext_state.map(ExtState::new),
            ext_tail: config.ext_tail.map(ExtTail::new),

            config,
            _marker: PhantomPinned,
        });

        // Self-referential fields.
        bed.clap_host.host_data = (&raw mut *bed).cast();
        bed.host = Some(unsafe { Host::new_unchecked(&raw mut bed.clap_host) });

        Box::into_pin(bed)
    }

    /// # Safety
    ///
    /// You must not use Host to get a pointer to this test bed and move it
    /// out ouf the Pin.
    pub const unsafe fn host_mut(self: Pin<&mut Self>) -> &mut Host {
        unsafe { self.get_unchecked_mut().host.as_mut().unwrap() }
    }

    pub fn test(mut self: Pin<&mut Self>, case: impl Test) -> Pin<&mut Self> {
        case.test(self.as_mut());
        self
    }
}

extern "C-unwind" fn get_extension(
    host: *const clap_host,
    extension_id: *const c_char,
) -> *const c_void {
    assert!(!host.is_null());
    let bed: &TestBed = unsafe { &*(*host).host_data.cast() };
    let extension_id = unsafe { CStr::from_ptr(extension_id) };

    if extension_id == CLAP_EXT_AUDIO_PORTS {
        if let Some(ext) = &bed.ext_audio_ports {
            return (&raw const ext.clap_host_audio_ports).cast();
        }
    }
    if extension_id == CLAP_EXT_LATENCY {
        if let Some(ext) = &bed.ext_latency {
            return (&raw const ext.clap_host_latency).cast();
        }
    }
    if extension_id == CLAP_EXT_LOG {
        if let Some(ext) = &bed.ext_log {
            return (&raw const ext.clap_host_log).cast();
        }
    }
    if extension_id == CLAP_EXT_NOTE_PORTS {
        if let Some(ext) = &bed.ext_note_ports {
            return (&raw const ext.clap_host_note_ports).cast();
        }
    }
    if extension_id == CLAP_EXT_PARAMS {
        if let Some(ext) = &bed.ext_params {
            return (&raw const ext.clap_host_params).cast();
        }
    }
    if extension_id == CLAP_EXT_STATE {
        if let Some(ext) = &bed.ext_state {
            return (&raw const ext.clap_host_state).cast();
        }
    }
    if extension_id == CLAP_EXT_TAIL {
        if let Some(ext) = &bed.ext_tail {
            return (&raw const ext.clap_host_tail).cast();
        }
    }

    null()
}

extern "C-unwind" fn request_restart(host: *const clap_host) {
    assert!(!host.is_null());
    let bed: &mut TestBed = unsafe { &mut *(*host).host_data.cast() };
    bed.call_request.restart = true;
}

extern "C-unwind" fn request_process(host: *const clap_host) {
    assert!(!host.is_null());
    let bed: &mut TestBed = unsafe { &mut *(*host).host_data.cast() };
    bed.call_request.process = true;
}

extern "C-unwind" fn request_callback(host: *const clap_host) {
    assert!(!host.is_null());
    let bed: &mut TestBed = unsafe { &mut *(*host).host_data.cast() };
    bed.call_request.callback = true;
}

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
        ext.call_rescan_flags = flags;
    }
}

#[derive(Debug, Default, Copy, Clone)]
pub struct ExtLatencyConfig {
    pub null_callback: bool,
}

#[derive(Debug)]
pub struct ExtLatency {
    clap_host_latency: clap_host_latency,
    pub call_changed: bool,
}

impl ExtLatency {
    fn new(config: ExtLatencyConfig) -> Self {
        Self {
            clap_host_latency: clap_host_latency {
                changed: (!config.null_callback).then_some(ext_latency_changed),
            },
            call_changed: false,
        }
    }
}

extern "C-unwind" fn ext_latency_changed(host: *const clap_host) {
    assert!(!host.is_null());
    let bed: &mut TestBed = unsafe { &mut *(*host).host_data.cast() };

    if let Some(ext) = &mut bed.ext_latency {
        ext.call_changed = true;
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

#[derive(Debug, Default, Copy, Clone)]
pub struct ExtNotePortsConfig {
    pub supported_dialects: u32,
    pub null_supported_dialects: bool,
    pub null_rescan: bool,
}

#[derive(Debug)]
pub struct ExtNotePorts {
    config: ExtNotePortsConfig,
    pub clap_host_note_ports: clap_host_note_ports,
    pub call_rescan_flags: u32,
}

impl ExtNotePorts {
    fn new(config: ExtNotePortsConfig) -> Self {
        Self {
            config,
            clap_host_note_ports: clap_host_note_ports {
                supported_dialects: (!config.null_supported_dialects)
                    .then_some(ext_note_ports_supported_dialects),
                rescan: (!config.null_rescan).then_some(ext_note_ports_rescan),
            },
            call_rescan_flags: 0,
        }
    }
}

extern "C-unwind" fn ext_note_ports_supported_dialects(host: *const clap_host) -> u32 {
    assert!(!host.is_null());
    let bed: &mut TestBed = unsafe { &mut *(*host).host_data.cast() };
    if let Some(ext) = &bed.ext_note_ports {
        ext.config.supported_dialects
    } else {
        0
    }
}

extern "C-unwind" fn ext_note_ports_rescan(host: *const clap_host, flags: u32) {
    assert!(!host.is_null());
    let bed: &mut TestBed = unsafe { &mut *(*host).host_data.cast() };
    if let Some(ext) = &mut bed.ext_note_ports {
        ext.call_rescan_flags = flags;
    }
}

#[derive(Debug, Default, Copy, Clone)]
pub struct ExtParamsConfig {
    pub null_callback: (bool, bool, bool),
}

#[derive(Debug)]
pub struct ExtParams {
    clap_host_params: clap_host_params,
    pub call_rescan_flags: u32,
    pub call_clear: u32,
    pub call_request_flush: bool,
}

impl ExtParams {
    fn new(config: ExtParamsConfig) -> Self {
        Self {
            clap_host_params: clap_host_params {
                rescan: (!config.null_callback.0).then_some(ext_params_rescan),
                clear: (!config.null_callback.1).then_some(ext_params_clear),
                request_flush: (!config.null_callback.2).then_some(ext_params_request_flush),
            },
            call_rescan_flags: 0,
            call_clear: 0,
            call_request_flush: false,
        }
    }
}

extern "C-unwind" fn ext_params_rescan(host: *const clap_host, flags: u32) {
    assert!(!host.is_null());
    let bed: &mut TestBed = unsafe { &mut *(*host).host_data.cast() };
    if let Some(ext) = &mut bed.ext_params {
        ext.call_rescan_flags = flags;
    }
}

extern "C-unwind" fn ext_params_clear(host: *const clap_host, _: clap_id, flags: u32) {
    assert!(!host.is_null());
    let bed: &mut TestBed = unsafe { &mut *(*host).host_data.cast() };
    if let Some(ext) = &mut bed.ext_params {
        ext.call_clear = flags;
    }
}

extern "C-unwind" fn ext_params_request_flush(host: *const clap_host) {
    assert!(!host.is_null());
    let bed: &mut TestBed = unsafe { &mut *(*host).host_data.cast() };
    if let Some(ext) = &mut bed.ext_params {
        ext.call_request_flush = true;
    }
}

#[derive(Debug, Default, Copy, Clone)]
pub struct ExtStateConfig {
    pub null_callback: bool,
}

#[derive(Debug)]
pub struct ExtState {
    clap_host_state: clap_host_state,
    pub call_make_dirty: bool,
}

impl ExtState {
    fn new(config: ExtStateConfig) -> Self {
        Self {
            clap_host_state: clap_host_state {
                mark_dirty: (!config.null_callback).then_some(ext_state_make_dirty),
            },
            call_make_dirty: false,
        }
    }
}

extern "C-unwind" fn ext_state_make_dirty(host: *const clap_host) {
    assert!(!host.is_null());
    let bed: &mut TestBed = unsafe { &mut *(*host).host_data.cast() };
    if let Some(ext) = &mut bed.ext_state {
        ext.call_make_dirty = true;
    }
}

#[derive(Debug, Default, Copy, Clone)]
pub struct ExtTailConfig {
    pub null_callback: bool,
}

#[derive(Debug)]
pub struct ExtTail {
    clap_host_tail: clap_host_tail,
    pub call_changed: bool,
}

impl ExtTail {
    fn new(config: ExtTailConfig) -> Self {
        Self {
            clap_host_tail: clap_host_tail {
                changed: (!config.null_callback).then_some(ext_tail_changed),
            },
            call_changed: false,
        }
    }
}

extern "C-unwind" fn ext_tail_changed(host: *const clap_host) {
    assert!(!host.is_null());
    let bed: &mut TestBed = unsafe { &mut *(*host).host_data.cast() };
    if let Some(ext) = &mut bed.ext_tail {
        ext.call_changed = true;
    }
}

struct CheckDescription;

impl Test for CheckDescription {
    fn test(self, bed: Pin<&mut TestBed>) {
        let name = bed.config.name.to_str().unwrap();
        let vendor = bed.config.vendor.to_str().unwrap();
        let url = bed.config.url.to_str().unwrap();
        let version = bed.config.version.to_str().unwrap();

        let host = unsafe { bed.host_mut() };

        assert_eq!(host.name(), name);
        assert_eq!(host.vendor(), vendor);
        assert_eq!(host.url(), url);
        assert_eq!(host.version(), version);
    }
}

#[test]
fn check_description_01() {
    TestConfig::default()
        .test(CheckDescription)
        .test(CheckDescription);
}

#[test]
fn check_description_02() {
    TestConfig {
        name: c"test_name",
        vendor: c"⧉⧉⧉",
        url: c"test_url",
        version: c"82[p",
        ..Default::default()
    }
    .test(CheckDescription);
}

enum CheckRequest {
    Restart,
    Process,
    Callback,
}

impl Test for CheckRequest {
    fn test(self, mut bed: Pin<&mut TestBed>) {
        assert_eq!(bed.call_request, CallRequest::default());

        let host = unsafe { bed.as_mut().host_mut() };

        match self {
            CheckRequest::Restart => {
                host.request_restart();

                assert_eq!(
                    bed.call_request,
                    CallRequest {
                        restart: true,
                        process: false,
                        callback: false,
                    }
                );
            }
            CheckRequest::Process => {
                host.request_process();

                assert_eq!(
                    bed.call_request,
                    CallRequest {
                        restart: false,
                        process: true,
                        callback: false,
                    }
                );
            }
            CheckRequest::Callback => {
                host.request_callback();

                assert_eq!(
                    bed.call_request,
                    CallRequest {
                        restart: false,
                        process: false,
                        callback: true,
                    }
                );
            }
        }
    }
}

#[test]
fn check_request_restart() {
    TestConfig::default().test(CheckRequest::Restart);
}

#[test]
fn check_request_process() {
    TestConfig::default().test(CheckRequest::Process);
}

#[test]
fn check_request_callback() {
    TestConfig::default().test(CheckRequest::Callback);
}
