//! Export ffi bindings to CLAP C API, version 1.2.3.

#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

use std::ffi::{CStr, c_char, c_int, c_uint, c_ulong, c_void};

pub const CLAP_VERSION_MAJOR: u32 = 1;
pub const CLAP_VERSION_MINOR: u32 = 2;
pub const CLAP_VERSION_REVISION: u32 = 3;
pub const CLAP_PLUGIN_FEATURE_INSTRUMENT: &CStr = c"instrument";
pub const CLAP_PLUGIN_FEATURE_AUDIO_EFFECT: &CStr = c"audio-effect";
pub const CLAP_PLUGIN_FEATURE_NOTE_EFFECT: &CStr = c"note-effect";
pub const CLAP_PLUGIN_FEATURE_NOTE_DETECTOR: &CStr = c"note-detector";
pub const CLAP_PLUGIN_FEATURE_ANALYZER: &CStr = c"analyzer";
pub const CLAP_PLUGIN_FEATURE_SYNTHESIZER: &CStr = c"synthesizer";
pub const CLAP_PLUGIN_FEATURE_SAMPLER: &CStr = c"sampler";
pub const CLAP_PLUGIN_FEATURE_DRUM: &CStr = c"drum";
pub const CLAP_PLUGIN_FEATURE_DRUM_MACHINE: &CStr = c"drum-machine";
pub const CLAP_PLUGIN_FEATURE_FILTER: &CStr = c"filter";
pub const CLAP_PLUGIN_FEATURE_PHASER: &CStr = c"phaser";
pub const CLAP_PLUGIN_FEATURE_EQUALIZER: &CStr = c"equalizer";
pub const CLAP_PLUGIN_FEATURE_DEESSER: &CStr = c"de-esser";
pub const CLAP_PLUGIN_FEATURE_PHASE_VOCODER: &CStr = c"phase-vocoder";
pub const CLAP_PLUGIN_FEATURE_GRANULAR: &CStr = c"granular";
pub const CLAP_PLUGIN_FEATURE_FREQUENCY_SHIFTER: &CStr = c"frequency-shifter";
pub const CLAP_PLUGIN_FEATURE_PITCH_SHIFTER: &CStr = c"pitch-shifter";
pub const CLAP_PLUGIN_FEATURE_DISTORTION: &CStr = c"distortion";
pub const CLAP_PLUGIN_FEATURE_TRANSIENT_SHAPER: &CStr = c"transient-shaper";
pub const CLAP_PLUGIN_FEATURE_COMPRESSOR: &CStr = c"compressor";
pub const CLAP_PLUGIN_FEATURE_EXPANDER: &CStr = c"expander";
pub const CLAP_PLUGIN_FEATURE_GATE: &CStr = c"gate";
pub const CLAP_PLUGIN_FEATURE_LIMITER: &CStr = c"limiter";
pub const CLAP_PLUGIN_FEATURE_FLANGER: &CStr = c"flanger";
pub const CLAP_PLUGIN_FEATURE_CHORUS: &CStr = c"chorus";
pub const CLAP_PLUGIN_FEATURE_DELAY: &CStr = c"delay";
pub const CLAP_PLUGIN_FEATURE_REVERB: &CStr = c"reverb";
pub const CLAP_PLUGIN_FEATURE_TREMOLO: &CStr = c"tremolo";
pub const CLAP_PLUGIN_FEATURE_GLITCH: &CStr = c"glitch";
pub const CLAP_PLUGIN_FEATURE_UTILITY: &CStr = c"utility";
pub const CLAP_PLUGIN_FEATURE_PITCH_CORRECTION: &CStr = c"pitch-correction";
pub const CLAP_PLUGIN_FEATURE_RESTORATION: &CStr = c"restoration";
pub const CLAP_PLUGIN_FEATURE_MULTI_EFFECTS: &CStr = c"multi-effects";
pub const CLAP_PLUGIN_FEATURE_MIXING: &CStr = c"mixing";
pub const CLAP_PLUGIN_FEATURE_MASTERING: &CStr = c"mastering";
pub const CLAP_PLUGIN_FEATURE_MONO: &CStr = c"mono";
pub const CLAP_PLUGIN_FEATURE_STEREO: &CStr = c"stereo";
pub const CLAP_PLUGIN_FEATURE_SURROUND: &CStr = c"surround";
pub const CLAP_PLUGIN_FEATURE_AMBISONIC: &CStr = c"ambisonic";

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_version {
    pub major: u32,
    pub minor: u32,
    pub revision: u32,
}

pub const CLAP_VERSION: clap_version = clap_version {
    major: CLAP_VERSION_MAJOR,
    minor: CLAP_VERSION_MINOR,
    revision: CLAP_VERSION_REVISION,
};

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_plugin_entry {
    pub clap_version: clap_version,
    pub init: Option<unsafe extern "C" fn(plugin_path: *const c_char) -> bool>,
    pub deinit: Option<unsafe extern "C" fn()>,
    pub get_factory: Option<unsafe extern "C" fn(factory_id: *const c_char) -> *const c_void>,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_host {
    pub clap_version: clap_version,
    pub host_data: *mut c_void,
    pub name: *const c_char,
    pub vendor: *const c_char,
    pub url: *const c_char,
    pub version: *const c_char,
    pub get_extension: Option<
        unsafe extern "C" fn(host: *const clap_host, extension_id: *const c_char) -> *const c_void,
    >,
    pub request_restart: Option<unsafe extern "C" fn(host: *const clap_host)>,
    pub request_process: Option<unsafe extern "C" fn(host: *const clap_host)>,
    pub request_callback: Option<unsafe extern "C" fn(host: *const clap_host)>,
}

#[doc = " We use fixed point representation of beat time and seconds time\n Usage:\n   double x = ...; // in beats\n   clap_beattime y = round(CLAP_BEATTIME_FACTOR * x);"]
pub const CLAP_BEATTIME_FACTOR: i64 = 2147483648; // 1 << 31
pub const CLAP_SECTIME_FACTOR: i64 = 2147483648;
pub type clap_beattime = i64;
pub type clap_sectime = i64;
pub type clap_id = u32;
pub const CLAP_INVALID_ID: clap_id = u32::MAX;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_event_header {
    pub size: u32,
    pub time: u32,
    pub space_id: u16,
    pub type_: u16,
    pub flags: u32,
}

pub const CLAP_CORE_EVENT_SPACE_ID: u16 = 0;
pub const clap_event_flags_CLAP_EVENT_IS_LIVE: clap_event_flags = 1;
pub const clap_event_flags_CLAP_EVENT_DONT_RECORD: clap_event_flags = 2;
pub type clap_event_flags = c_uint;
pub const CLAP_EVENT_NOTE_ON: clap_event_flags = 0;
pub const CLAP_EVENT_NOTE_OFF: clap_event_flags = 1;
pub const CLAP_EVENT_NOTE_CHOKE: clap_event_flags = 2;
pub const CLAP_EVENT_NOTE_END: clap_event_flags = 3;
pub const CLAP_EVENT_NOTE_EXPRESSION: clap_event_flags = 4;
pub const CLAP_EVENT_PARAM_VALUE: clap_event_flags = 5;
pub const CLAP_EVENT_PARAM_MOD: clap_event_flags = 6;
pub const CLAP_EVENT_PARAM_GESTURE_BEGIN: clap_event_flags = 7;
pub const CLAP_EVENT_PARAM_GESTURE_END: clap_event_flags = 8;
pub const CLAP_EVENT_TRANSPORT: clap_event_flags = 9;
pub const CLAP_EVENT_MIDI: clap_event_flags = 10;
pub const CLAP_EVENT_MIDI_SYSEX: clap_event_flags = 11;
pub const CLAP_EVENT_MIDI2: clap_event_flags = 12;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_event_note {
    pub header: clap_event_header,
    pub note_id: i32,
    pub port_index: i16,
    pub channel: i16,
    pub key: i16,
    pub velocity: f64,
}

pub type clap_note_expression = i32;
pub const CLAP_NOTE_EXPRESSION_VOLUME: clap_note_expression = 0;
pub const CLAP_NOTE_EXPRESSION_PAN: clap_note_expression = 1;
pub const CLAP_NOTE_EXPRESSION_TUNING: clap_note_expression = 2;
pub const CLAP_NOTE_EXPRESSION_VIBRATO: clap_note_expression = 3;
pub const CLAP_NOTE_EXPRESSION_EXPRESSION: clap_note_expression = 4;
pub const CLAP_NOTE_EXPRESSION_BRIGHTNESS: clap_note_expression = 5;
pub const CLAP_NOTE_EXPRESSION_PRESSURE: clap_note_expression = 6;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_event_note_expression {
    pub header: clap_event_header,
    pub expression_id: clap_note_expression,
    pub note_id: i32,
    pub port_index: i16,
    pub channel: i16,
    pub key: i16,
    pub value: f64,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_event_param_value {
    pub header: clap_event_header,
    pub param_id: clap_id,
    pub cookie: *mut c_void,
    pub note_id: i32,
    pub port_index: i16,
    pub channel: i16,
    pub key: i16,
    pub value: f64,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_event_param_mod {
    pub header: clap_event_header,
    pub param_id: clap_id,
    pub cookie: *mut c_void,
    pub note_id: i32,
    pub port_index: i16,
    pub channel: i16,
    pub key: i16,
    pub amount: f64,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_event_param_gesture {
    pub header: clap_event_header,
    pub param_id: clap_id,
}

pub const clap_transport_flags_CLAP_TRANSPORT_HAS_TEMPO: clap_transport_flags = 1;
pub const clap_transport_flags_CLAP_TRANSPORT_HAS_BEATS_TIMELINE: clap_transport_flags = 2;
pub const clap_transport_flags_CLAP_TRANSPORT_HAS_SECONDS_TIMELINE: clap_transport_flags = 4;
pub const clap_transport_flags_CLAP_TRANSPORT_HAS_TIME_SIGNATURE: clap_transport_flags = 8;
pub const clap_transport_flags_CLAP_TRANSPORT_IS_PLAYING: clap_transport_flags = 16;
pub const clap_transport_flags_CLAP_TRANSPORT_IS_RECORDING: clap_transport_flags = 32;
pub const clap_transport_flags_CLAP_TRANSPORT_IS_LOOP_ACTIVE: clap_transport_flags = 64;
pub const clap_transport_flags_CLAP_TRANSPORT_IS_WITHIN_PRE_ROLL: clap_transport_flags = 128;
pub type clap_transport_flags = c_uint;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_event_transport {
    pub header: clap_event_header,
    pub flags: u32,
    pub song_pos_beats: clap_beattime,
    pub song_pos_seconds: clap_sectime,
    pub tempo: f64,
    pub tempo_inc: f64,
    pub loop_start_beats: clap_beattime,
    pub loop_end_beats: clap_beattime,
    pub loop_start_seconds: clap_sectime,
    pub loop_end_seconds: clap_sectime,
    pub bar_start: clap_beattime,
    pub bar_number: i32,
    pub tsig_num: u16,
    pub tsig_denom: u16,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_event_midi {
    pub header: clap_event_header,
    pub port_index: u16,
    pub data: [u8; 3usize],
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_event_midi_sysex {
    pub header: clap_event_header,
    pub port_index: u16,
    pub buffer: *const u8,
    pub size: u32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_event_midi2 {
    pub header: clap_event_header,
    pub port_index: u16,
    pub data: [u32; 4usize],
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_input_events {
    pub ctx: *mut c_void,
    pub size: Option<unsafe extern "C" fn(list: *const clap_input_events) -> u32>,
    pub get: Option<
        unsafe extern "C" fn(
            list: *const clap_input_events,
            index: u32,
        ) -> *const clap_event_header,
    >,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_output_events {
    pub ctx: *mut c_void,
    pub try_push: Option<
        unsafe extern "C" fn(
            list: *const clap_output_events,
            event: *const clap_event_header,
        ) -> bool,
    >,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_audio_buffer {
    pub data32: *mut *mut f32,
    pub data64: *mut *mut f64,
    pub channel_count: u32,
    pub latency: u32,
    pub constant_mask: u64,
}

pub type clap_process_status = i32;
pub const CLAP_PROCESS_ERROR: clap_process_status = 0;
pub const CLAP_PROCESS_CONTINUE: clap_process_status = 1;
pub const CLAP_PROCESS_CONTINUE_IF_NOT_QUIET: clap_process_status = 2;
pub const CLAP_PROCESS_TAIL: clap_process_status = 3;
pub const CLAP_PROCESS_SLEEP: clap_process_status = 4;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_process {
    pub steady_time: i64,
    pub frames_count: u32,
    pub transport: *const clap_event_transport,
    pub audio_inputs: *const clap_audio_buffer,
    pub audio_outputs: *mut clap_audio_buffer,
    pub audio_inputs_count: u32,
    pub audio_outputs_count: u32,
    pub in_events: *const clap_input_events,
    pub out_events: *const clap_output_events,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_plugin_descriptor {
    pub clap_version: clap_version,
    pub id: *const c_char,
    pub name: *const c_char,
    pub vendor: *const c_char,
    pub url: *const c_char,
    pub manual_url: *const c_char,
    pub support_url: *const c_char,
    pub version: *const c_char,
    pub description: *const c_char,
    pub features: *const *const c_char,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_plugin {
    pub desc: *const clap_plugin_descriptor,
    pub plugin_data: *mut c_void,
    pub init: Option<unsafe extern "C" fn(plugin: *const clap_plugin) -> bool>,
    pub destroy: Option<unsafe extern "C" fn(plugin: *const clap_plugin)>,
    pub activate: Option<
        unsafe extern "C" fn(
            plugin: *const clap_plugin,
            sample_rate: f64,
            min_frames_count: u32,
            max_frames_count: u32,
        ) -> bool,
    >,
    pub deactivate: Option<unsafe extern "C" fn(plugin: *const clap_plugin)>,
    pub start_processing: Option<unsafe extern "C" fn(plugin: *const clap_plugin) -> bool>,
    pub stop_processing: Option<unsafe extern "C" fn(plugin: *const clap_plugin)>,
    pub reset: Option<unsafe extern "C" fn(plugin: *const clap_plugin)>,
    pub process: Option<
        unsafe extern "C" fn(
            plugin: *const clap_plugin,
            process: *const clap_process,
        ) -> clap_process_status,
    >,
    pub get_extension: Option<
        unsafe extern "C" fn(plugin: *const clap_plugin, id: *const c_char) -> *const c_void,
    >,
    pub on_main_thread: Option<unsafe extern "C" fn(plugin: *const clap_plugin)>,
}

pub const CLAP_PLUGIN_FACTORY_ID: &CStr = c"clap.plugin-factory";

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_plugin_factory {
    pub get_plugin_count: Option<unsafe extern "C" fn(factory: *const clap_plugin_factory) -> u32>,
    pub get_plugin_descriptor: Option<
        unsafe extern "C" fn(
            factory: *const clap_plugin_factory,
            index: u32,
        ) -> *const clap_plugin_descriptor,
    >,
    pub create_plugin: Option<
        unsafe extern "C" fn(
            factory: *const clap_plugin_factory,
            host: *const clap_host,
            plugin_id: *const c_char,
        ) -> *const clap_plugin,
    >,
}

pub type clap_timestamp = u64;
pub const CLAP_TIMESTAMP_UNKNOWN: clap_timestamp = 0;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_universal_plugin_id {
    pub abi: *const c_char,
    pub id: *const c_char,
}

pub const CLAP_PRESET_DISCOVERY_FACTORY_ID: &CStr = c"clap.preset-discovery-factory/2";
pub const CLAP_PRESET_DISCOVERY_FACTORY_ID_COMPAT: &CStr = c"clap.preset-discovery-factory/draft-2";
pub const clap_preset_discovery_location_kind_CLAP_PRESET_DISCOVERY_LOCATION_FILE:
    clap_preset_discovery_location_kind = 0;
pub const clap_preset_discovery_location_kind_CLAP_PRESET_DISCOVERY_LOCATION_PLUGIN:
    clap_preset_discovery_location_kind = 1;
pub type clap_preset_discovery_location_kind = c_uint;
pub const clap_preset_discovery_flags_CLAP_PRESET_DISCOVERY_IS_FACTORY_CONTENT:
    clap_preset_discovery_flags = 1;
pub const clap_preset_discovery_flags_CLAP_PRESET_DISCOVERY_IS_USER_CONTENT:
    clap_preset_discovery_flags = 2;
pub const clap_preset_discovery_flags_CLAP_PRESET_DISCOVERY_IS_DEMO_CONTENT:
    clap_preset_discovery_flags = 4;
pub const clap_preset_discovery_flags_CLAP_PRESET_DISCOVERY_IS_FAVORITE:
    clap_preset_discovery_flags = 8;
pub type clap_preset_discovery_flags = c_uint;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_preset_discovery_metadata_receiver {
    pub receiver_data: *mut c_void,
    pub on_error: Option<
        unsafe extern "C" fn(
            receiver: *const clap_preset_discovery_metadata_receiver,
            os_error: i32,
            error_message: *const c_char,
        ),
    >,
    pub begin_preset: Option<
        unsafe extern "C" fn(
            receiver: *const clap_preset_discovery_metadata_receiver,
            name: *const c_char,
            load_key: *const c_char,
        ) -> bool,
    >,
    pub add_plugin_id: Option<
        unsafe extern "C" fn(
            receiver: *const clap_preset_discovery_metadata_receiver,
            plugin_id: *const clap_universal_plugin_id,
        ),
    >,
    pub set_soundpack_id: Option<
        unsafe extern "C" fn(
            receiver: *const clap_preset_discovery_metadata_receiver,
            soundpack_id: *const c_char,
        ),
    >,
    pub set_flags: Option<
        unsafe extern "C" fn(receiver: *const clap_preset_discovery_metadata_receiver, flags: u32),
    >,
    pub add_creator: Option<
        unsafe extern "C" fn(
            receiver: *const clap_preset_discovery_metadata_receiver,
            creator: *const c_char,
        ),
    >,
    pub set_description: Option<
        unsafe extern "C" fn(
            receiver: *const clap_preset_discovery_metadata_receiver,
            description: *const c_char,
        ),
    >,
    pub set_timestamps: Option<
        unsafe extern "C" fn(
            receiver: *const clap_preset_discovery_metadata_receiver,
            creation_time: clap_timestamp,
            modification_time: clap_timestamp,
        ),
    >,
    pub add_feature: Option<
        unsafe extern "C" fn(
            receiver: *const clap_preset_discovery_metadata_receiver,
            feature: *const c_char,
        ),
    >,
    pub add_extra_info: Option<
        unsafe extern "C" fn(
            receiver: *const clap_preset_discovery_metadata_receiver,
            key: *const c_char,
            value: *const c_char,
        ),
    >,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_preset_discovery_filetype {
    pub name: *const c_char,
    pub description: *const c_char,
    pub file_extension: *const c_char,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_preset_discovery_location {
    pub flags: u32,
    pub name: *const c_char,
    pub kind: u32,
    pub location: *const c_char,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_preset_discovery_soundpack {
    pub flags: u32,
    pub id: *const c_char,
    pub name: *const c_char,
    pub description: *const c_char,
    pub homepage_url: *const c_char,
    pub vendor: *const c_char,
    pub image_path: *const c_char,
    pub release_timestamp: clap_timestamp,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_preset_discovery_provider_descriptor {
    pub clap_version: clap_version,
    pub id: *const c_char,
    pub name: *const c_char,
    pub vendor: *const c_char,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_preset_discovery_provider {
    pub desc: *const clap_preset_discovery_provider_descriptor,
    pub provider_data: *mut c_void,
    pub init: Option<unsafe extern "C" fn(provider: *const clap_preset_discovery_provider) -> bool>,
    pub destroy: Option<unsafe extern "C" fn(provider: *const clap_preset_discovery_provider)>,
    pub get_metadata: Option<
        unsafe extern "C" fn(
            provider: *const clap_preset_discovery_provider,
            location_kind: u32,
            location: *const c_char,
            metadata_receiver: *const clap_preset_discovery_metadata_receiver,
        ) -> bool,
    >,
    pub get_extension: Option<
        unsafe extern "C" fn(
            provider: *const clap_preset_discovery_provider,
            extension_id: *const c_char,
        ) -> *const c_void,
    >,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_preset_discovery_indexer {
    pub clap_version: clap_version,
    pub name: *const c_char,
    pub vendor: *const c_char,
    pub url: *const c_char,
    pub version: *const c_char,
    pub indexer_data: *mut c_void,
    pub declare_filetype: Option<
        unsafe extern "C" fn(
            indexer: *const clap_preset_discovery_indexer,
            filetype: *const clap_preset_discovery_filetype,
        ) -> bool,
    >,
    pub declare_location: Option<
        unsafe extern "C" fn(
            indexer: *const clap_preset_discovery_indexer,
            location: *const clap_preset_discovery_location,
        ) -> bool,
    >,
    pub declare_soundpack: Option<
        unsafe extern "C" fn(
            indexer: *const clap_preset_discovery_indexer,
            soundpack: *const clap_preset_discovery_soundpack,
        ) -> bool,
    >,
    pub get_extension: Option<
        unsafe extern "C" fn(
            indexer: *const clap_preset_discovery_indexer,
            extension_id: *const c_char,
        ) -> *const c_void,
    >,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_preset_discovery_factory {
    pub count: Option<unsafe extern "C" fn(factory: *const clap_preset_discovery_factory) -> u32>,
    pub get_descriptor: Option<
        unsafe extern "C" fn(
            factory: *const clap_preset_discovery_factory,
            index: u32,
        ) -> *const clap_preset_discovery_provider_descriptor,
    >,
    pub create: Option<
        unsafe extern "C" fn(
            factory: *const clap_preset_discovery_factory,
            indexer: *const clap_preset_discovery_indexer,
            provider_id: *const c_char,
        ) -> *const clap_preset_discovery_provider,
    >,
}

pub const CLAP_EXT_AMBISONIC: &CStr = c"clap.ambisonic/3";
pub const CLAP_EXT_AMBISONIC_COMPAT: &CStr = c"clap.ambisonic.draft/3";
pub const CLAP_PORT_AMBISONIC: &CStr = c"ambisonic";
pub const clap_ambisonic_ordering_CLAP_AMBISONIC_ORDERING_FUMA: clap_ambisonic_ordering = 0;
pub const clap_ambisonic_ordering_CLAP_AMBISONIC_ORDERING_ACN: clap_ambisonic_ordering = 1;
pub type clap_ambisonic_ordering = c_uint;
pub const clap_ambisonic_normalization_CLAP_AMBISONIC_NORMALIZATION_MAXN:
    clap_ambisonic_normalization = 0;
pub const clap_ambisonic_normalization_CLAP_AMBISONIC_NORMALIZATION_SN3D:
    clap_ambisonic_normalization = 1;
pub const clap_ambisonic_normalization_CLAP_AMBISONIC_NORMALIZATION_N3D:
    clap_ambisonic_normalization = 2;
pub const clap_ambisonic_normalization_CLAP_AMBISONIC_NORMALIZATION_SN2D:
    clap_ambisonic_normalization = 3;
pub const clap_ambisonic_normalization_CLAP_AMBISONIC_NORMALIZATION_N2D:
    clap_ambisonic_normalization = 4;
pub type clap_ambisonic_normalization = c_uint;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_ambisonic_config {
    pub ordering: u32,
    pub normalization: u32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_plugin_ambisonic {
    pub is_config_supported: Option<
        unsafe extern "C" fn(
            plugin: *const clap_plugin,
            config: *const clap_ambisonic_config,
        ) -> bool,
    >,
    pub get_config: Option<
        unsafe extern "C" fn(
            plugin: *const clap_plugin,
            is_input: bool,
            port_index: u32,
            config: *mut clap_ambisonic_config,
        ) -> bool,
    >,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_host_ambisonic {
    pub changed: Option<unsafe extern "C" fn(host: *const clap_host)>,
}

#[doc = " @page Audio Ports Activation\n\n This extension provides a way for the host to activate and de-activate audio ports.\n Deactivating a port provides the following benefits:\n - the plugin knows ahead of time that a given input is not present and can choose\n   an optimized computation path,\n - the plugin knows that an output is not consumed by the host, and doesn't need to\n   compute it.\n\n Audio ports can only be activated or deactivated when the plugin is deactivated, unless\n can_activate_while_processing() returns true.\n\n Audio buffers must still be provided if the audio port is deactivated.\n In such case, they shall be filled with 0 (or whatever is the neutral value in your context)\n and the constant_mask shall be set.\n\n Audio ports are initially in the active state after creating the plugin instance.\n Audio ports state are not saved in the plugin state, so the host must restore the\n audio ports state after creating the plugin instance.\n\n Audio ports state is invalidated by clap_plugin_audio_ports_config.select() and\n clap_host_audio_ports.rescan(CLAP_AUDIO_PORTS_RESCAN_LIST)."]
pub const CLAP_EXT_AUDIO_PORTS_ACTIVATION: &CStr = c"clap.audio-ports-activation/2";
pub const CLAP_EXT_AUDIO_PORTS_ACTIVATION_COMPAT: &CStr = c"clap.audio-ports-activation/draft-2";

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_plugin_audio_ports_activation {
    pub can_activate_while_processing:
        Option<unsafe extern "C" fn(plugin: *const clap_plugin) -> bool>,
    pub set_active: Option<
        unsafe extern "C" fn(
            plugin: *const clap_plugin,
            is_input: bool,
            port_index: u32,
            is_active: bool,
            sample_size: u32,
        ) -> bool,
    >,
}

pub const CLAP_NAME_SIZE: c_uint = 256;
pub const CLAP_PATH_SIZE: c_uint = 1024;

#[doc = " @page Audio Ports\n\n This extension provides a way for the plugin to describe its current audio ports.\n\n If the plugin does not implement this extension, it won't have audio ports.\n\n 32 bits support is required for both host and plugins. 64 bits audio is optional.\n\n The plugin is only allowed to change its ports configuration while it is deactivated."]
pub const CLAP_EXT_AUDIO_PORTS: &CStr = c"clap.audio-ports";
pub const CLAP_PORT_MONO: &CStr = c"mono";
pub const CLAP_PORT_STEREO: &CStr = c"stereo";
pub const CLAP_AUDIO_PORT_IS_MAIN: c_uint = 1;
pub const CLAP_AUDIO_PORT_SUPPORTS_64BITS: c_uint = 2;
pub const CLAP_AUDIO_PORT_PREFERS_64BITS: c_uint = 4;
pub const CLAP_AUDIO_PORT_REQUIRES_COMMON_SAMPLE_SIZE: c_uint = 8;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_audio_port_info {
    pub id: clap_id,
    pub name: [c_char; 256usize],
    pub flags: u32,
    pub channel_count: u32,
    pub port_type: *const c_char,
    pub in_place_pair: clap_id,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_plugin_audio_ports {
    pub count: Option<unsafe extern "C" fn(plugin: *const clap_plugin, is_input: bool) -> u32>,
    pub get: Option<
        unsafe extern "C" fn(
            plugin: *const clap_plugin,
            index: u32,
            is_input: bool,
            info: *mut clap_audio_port_info,
        ) -> bool,
    >,
}

pub const CLAP_AUDIO_PORTS_RESCAN_NAMES: c_uint = 1;
pub const CLAP_AUDIO_PORTS_RESCAN_FLAGS: c_uint = 2;
pub const CLAP_AUDIO_PORTS_RESCAN_CHANNEL_COUNT: c_uint = 4;
pub const CLAP_AUDIO_PORTS_RESCAN_PORT_TYPE: c_uint = 8;
pub const CLAP_AUDIO_PORTS_RESCAN_IN_PLACE_PAIR: c_uint = 16;
pub const CLAP_AUDIO_PORTS_RESCAN_LIST: c_uint = 32;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_host_audio_ports {
    pub is_rescan_flag_supported:
        Option<unsafe extern "C" fn(host: *const clap_host, flag: u32) -> bool>,
    pub rescan: Option<unsafe extern "C" fn(host: *const clap_host, flags: u32)>,
}

#[doc = " @page Audio Ports Config\n\n This extension let the plugin provide port configurations presets.\n For example mono, stereo, surround, ambisonic, ...\n\n After the plugin initialization, the host may scan the list of configurations and eventually\n select one that fits the plugin context. The host can only select a configuration if the plugin\n is deactivated.\n\n A configuration is a very simple description of the audio ports:\n - it describes the main input and output ports\n - it has a name that can be displayed to the user\n\n The idea behind the configurations, is to let the user choose one via a menu.\n\n Plugins with very complex configuration possibilities should let the user configure the ports\n from the plugin GUI, and call @ref clap_host_audio_ports.rescan(CLAP_AUDIO_PORTS_RESCAN_ALL).\n\n To inquire the exact bus layout, the plugin implements the clap_plugin_audio_ports_config_info_t\n extension where all busses can be retrieved in the same way as in the audio-port extension."]
pub const CLAP_EXT_AUDIO_PORTS_CONFIG: &CStr = c"clap.audio-ports-config";
pub const CLAP_EXT_AUDIO_PORTS_CONFIG_INFO: &CStr = c"clap.audio-ports-config-info/1";
pub const CLAP_EXT_AUDIO_PORTS_CONFIG_INFO_COMPAT: &CStr = c"clap.audio-ports-config-info/draft-0";

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_audio_ports_config {
    pub id: clap_id,
    pub name: [c_char; 256usize],
    pub input_port_count: u32,
    pub output_port_count: u32,
    pub has_main_input: bool,
    pub main_input_channel_count: u32,
    pub main_input_port_type: *const c_char,
    pub has_main_output: bool,
    pub main_output_channel_count: u32,
    pub main_output_port_type: *const c_char,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_plugin_audio_ports_config {
    pub count: Option<unsafe extern "C" fn(plugin: *const clap_plugin) -> u32>,
    pub get: Option<
        unsafe extern "C" fn(
            plugin: *const clap_plugin,
            index: u32,
            config: *mut clap_audio_ports_config,
        ) -> bool,
    >,
    pub select:
        Option<unsafe extern "C" fn(plugin: *const clap_plugin, config_id: clap_id) -> bool>,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_plugin_audio_ports_config_info {
    pub current_config: Option<unsafe extern "C" fn(plugin: *const clap_plugin) -> clap_id>,
    pub get: Option<
        unsafe extern "C" fn(
            plugin: *const clap_plugin,
            config_id: clap_id,
            port_index: u32,
            is_input: bool,
            info: *mut clap_audio_port_info,
        ) -> bool,
    >,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_host_audio_ports_config {
    pub rescan: Option<unsafe extern "C" fn(host: *const clap_host)>,
}

pub const CLAP_EXT_CONFIGURABLE_AUDIO_PORTS: &CStr = c"clap.configurable-audio-ports/1";
pub const CLAP_EXT_CONFIGURABLE_AUDIO_PORTS_COMPAT: &CStr = c"clap.configurable-audio-ports.draft1";

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_audio_port_configuration_request {
    pub is_input: bool,
    pub port_index: u32,
    pub channel_count: u32,
    pub port_type: *const c_char,
    pub port_details: *const c_void,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_plugin_configurable_audio_ports {
    pub can_apply_configuration: Option<
        unsafe extern "C" fn(
            plugin: *const clap_plugin,
            requests: *const clap_audio_port_configuration_request,
            request_count: u32,
        ) -> bool,
    >,
    pub apply_configuration: Option<
        unsafe extern "C" fn(
            plugin: *const clap_plugin,
            requests: *const clap_audio_port_configuration_request,
            request_count: u32,
        ) -> bool,
    >,
}

pub const CLAP_EXT_CONTEXT_MENU: &CStr = c"clap.context-menu/1";
pub const CLAP_EXT_CONTEXT_MENU_COMPAT: &CStr = c"clap.context-menu.draft/0";
pub const CLAP_CONTEXT_MENU_TARGET_KIND_GLOBAL: c_uint = 0;
pub const CLAP_CONTEXT_MENU_TARGET_KIND_PARAM: c_uint = 1;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_context_menu_target {
    pub kind: u32,
    pub id: clap_id,
}

pub const CLAP_CONTEXT_MENU_ITEM_ENTRY: c_uint = 0;
pub const CLAP_CONTEXT_MENU_ITEM_CHECK_ENTRY: c_uint = 1;
pub const CLAP_CONTEXT_MENU_ITEM_SEPARATOR: c_uint = 2;
pub const CLAP_CONTEXT_MENU_ITEM_BEGIN_SUBMENU: c_uint = 3;
pub const CLAP_CONTEXT_MENU_ITEM_END_SUBMENU: c_uint = 4;
pub const CLAP_CONTEXT_MENU_ITEM_TITLE: c_uint = 5;
pub type clap_context_menu_item_kind = u32;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_context_menu_entry {
    pub label: *const c_char,
    pub is_enabled: bool,
    pub action_id: clap_id,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_context_menu_check_entry {
    pub label: *const c_char,
    pub is_enabled: bool,
    pub is_checked: bool,
    pub action_id: clap_id,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_context_menu_item_title {
    pub title: *const c_char,
    pub is_enabled: bool,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_context_menu_submenu {
    pub label: *const c_char,
    pub is_enabled: bool,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_context_menu_builder {
    pub ctx: *mut c_void,
    pub add_item: Option<
        unsafe extern "C" fn(
            builder: *const clap_context_menu_builder,
            item_kind: clap_context_menu_item_kind,
            item_data: *const c_void,
        ) -> bool,
    >,
    pub supports: Option<
        unsafe extern "C" fn(
            builder: *const clap_context_menu_builder,
            item_kind: clap_context_menu_item_kind,
        ) -> bool,
    >,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_plugin_context_menu {
    pub populate: Option<
        unsafe extern "C" fn(
            plugin: *const clap_plugin,
            target: *const clap_context_menu_target,
            builder: *const clap_context_menu_builder,
        ) -> bool,
    >,
    pub perform: Option<
        unsafe extern "C" fn(
            plugin: *const clap_plugin,
            target: *const clap_context_menu_target,
            action_id: clap_id,
        ) -> bool,
    >,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_host_context_menu {
    pub populate: Option<
        unsafe extern "C" fn(
            host: *const clap_host,
            target: *const clap_context_menu_target,
            builder: *const clap_context_menu_builder,
        ) -> bool,
    >,
    pub perform: Option<
        unsafe extern "C" fn(
            host: *const clap_host,
            target: *const clap_context_menu_target,
            action_id: clap_id,
        ) -> bool,
    >,
    pub can_popup: Option<unsafe extern "C" fn(host: *const clap_host) -> bool>,
    pub popup: Option<
        unsafe extern "C" fn(
            host: *const clap_host,
            target: *const clap_context_menu_target,
            screen_index: i32,
            x: i32,
            y: i32,
        ) -> bool,
    >,
}

pub const CLAP_EXT_EVENT_REGISTRY: &CStr = c"clap.event-registry";

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_host_event_registry {
    pub query: Option<
        unsafe extern "C" fn(
            host: *const clap_host,
            space_name: *const c_char,
            space_id: *mut u16,
        ) -> bool,
    >,
}

#[doc = " @page GUI\n\n This extension defines how the plugin will present its GUI.\n\n There are two approaches:\n 1. the plugin creates a window and embeds it into the host's window\n 2. the plugin creates a floating window\n\n Embedding the window gives more control to the host, and feels more integrated.\n Floating window are sometimes the only option due to technical limitations.\n\n The Embedding protocol is by far the most common, supported by all hosts to date,\n and a plugin author should support at least that case.\n\n Showing the GUI works as follow:\n  1. clap_plugin_gui->is_api_supported(), check what can work\n  2. clap_plugin_gui->create(), allocates gui resources\n  3. if the plugin window is floating\n  4.    -> clap_plugin_gui->set_transient()\n  5.    -> clap_plugin_gui->suggest_title()\n  6. else\n  7.    -> clap_plugin_gui->set_scale()\n  8.    -> clap_plugin_gui->can_resize()\n  9.    -> if resizable and has known size from previous session, clap_plugin_gui->set_size()\n 10.    -> else clap_plugin_gui->get_size(), gets initial size\n 11.    -> clap_plugin_gui->set_parent()\n 12. clap_plugin_gui->show()\n 13. clap_plugin_gui->hide()/show() ...\n 14. clap_plugin_gui->destroy() when done with the gui\n\n Resizing the window (initiated by the plugin, if embedded):\n 1. Plugins calls clap_host_gui->request_resize()\n 2. If the host returns true the new size is accepted,\n    the host doesn't have to call clap_plugin_gui->set_size().\n    If the host returns false, the new size is rejected.\n\n Resizing the window (drag, if embedded)):\n 1. Only possible if clap_plugin_gui->can_resize() returns true\n 2. Mouse drag -> new_size\n 3. clap_plugin_gui->adjust_size(new_size) -> working_size\n 4. clap_plugin_gui->set_size(working_size)"]
pub const CLAP_EXT_GUI: &CStr = c"clap.gui";
pub const CLAP_WINDOW_API_WIN32: &CStr = c"win32";
pub const CLAP_WINDOW_API_COCOA: &CStr = c"cocoa";
pub const CLAP_WINDOW_API_X11: &CStr = c"x11";
pub const CLAP_WINDOW_API_WAYLAND: &CStr = c"wayland";

pub type clap_hwnd = *mut c_void;
pub type clap_nsview = *mut c_void;
pub type clap_xwnd = c_ulong;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct clap_window {
    pub api: *const c_char,
    pub __bindgen_anon_1: clap_window__bindgen_ty_1,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union clap_window__bindgen_ty_1 {
    pub cocoa: clap_nsview,
    pub x11: clap_xwnd,
    pub win32: clap_hwnd,
    pub ptr: *mut c_void,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_gui_resize_hints {
    pub can_resize_horizontally: bool,
    pub can_resize_vertically: bool,
    pub preserve_aspect_ratio: bool,
    pub aspect_ratio_width: u32,
    pub aspect_ratio_height: u32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_plugin_gui {
    pub is_api_supported: Option<
        unsafe extern "C" fn(
            plugin: *const clap_plugin,
            api: *const c_char,
            is_floating: bool,
        ) -> bool,
    >,
    pub get_preferred_api: Option<
        unsafe extern "C" fn(
            plugin: *const clap_plugin,
            api: *mut *const c_char,
            is_floating: *mut bool,
        ) -> bool,
    >,
    pub create: Option<
        unsafe extern "C" fn(
            plugin: *const clap_plugin,
            api: *const c_char,
            is_floating: bool,
        ) -> bool,
    >,
    pub destroy: Option<unsafe extern "C" fn(plugin: *const clap_plugin)>,
    pub set_scale: Option<unsafe extern "C" fn(plugin: *const clap_plugin, scale: f64) -> bool>,
    pub get_size: Option<
        unsafe extern "C" fn(plugin: *const clap_plugin, width: *mut u32, height: *mut u32) -> bool,
    >,
    pub can_resize: Option<unsafe extern "C" fn(plugin: *const clap_plugin) -> bool>,
    pub get_resize_hints: Option<
        unsafe extern "C" fn(plugin: *const clap_plugin, hints: *mut clap_gui_resize_hints) -> bool,
    >,
    pub adjust_size: Option<
        unsafe extern "C" fn(plugin: *const clap_plugin, width: *mut u32, height: *mut u32) -> bool,
    >,
    pub set_size:
        Option<unsafe extern "C" fn(plugin: *const clap_plugin, width: u32, height: u32) -> bool>,
    pub set_parent: Option<
        unsafe extern "C" fn(plugin: *const clap_plugin, window: *const clap_window) -> bool,
    >,
    pub set_transient: Option<
        unsafe extern "C" fn(plugin: *const clap_plugin, window: *const clap_window) -> bool,
    >,
    pub suggest_title:
        Option<unsafe extern "C" fn(plugin: *const clap_plugin, title: *const c_char)>,
    pub show: Option<unsafe extern "C" fn(plugin: *const clap_plugin) -> bool>,
    pub hide: Option<unsafe extern "C" fn(plugin: *const clap_plugin) -> bool>,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_host_gui {
    pub resize_hints_changed: Option<unsafe extern "C" fn(host: *const clap_host)>,
    pub request_resize:
        Option<unsafe extern "C" fn(host: *const clap_host, width: u32, height: u32) -> bool>,
    pub request_show: Option<unsafe extern "C" fn(host: *const clap_host) -> bool>,
    pub request_hide: Option<unsafe extern "C" fn(host: *const clap_host) -> bool>,
    pub closed: Option<unsafe extern "C" fn(host: *const clap_host, was_destroyed: bool)>,
}

pub const CLAP_EXT_LATENCY: &CStr = c"clap.latency";

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_plugin_latency {
    pub get: Option<unsafe extern "C" fn(plugin: *const clap_plugin) -> u32>,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_host_latency {
    pub changed: Option<unsafe extern "C" fn(host: *const clap_host)>,
}

pub const CLAP_EXT_LOG: &CStr = c"clap.log";
pub const CLAP_LOG_DEBUG: clap_log_severity = 0;
pub const CLAP_LOG_INFO: clap_log_severity = 1;
pub const CLAP_LOG_WARNING: clap_log_severity = 2;
pub const CLAP_LOG_ERROR: clap_log_severity = 3;
pub const CLAP_LOG_FATAL: clap_log_severity = 4;
pub const CLAP_LOG_HOST_MISBEHAVING: clap_log_severity = 5;
pub const CLAP_LOG_PLUGIN_MISBEHAVING: clap_log_severity = 6;
pub type clap_log_severity = i32;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_host_log {
    pub log: Option<
        unsafe extern "C" fn(
            host: *const clap_host,
            severity: clap_log_severity,
            msg: *const c_char,
        ),
    >,
}

pub const CLAP_EXT_NOTE_NAME: &CStr = c"clap.note-name";
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_note_name {
    pub name: [c_char; 256usize],
    pub port: i16,
    pub key: i16,
    pub channel: i16,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_plugin_note_name {
    pub count: Option<unsafe extern "C" fn(plugin: *const clap_plugin) -> u32>,
    pub get: Option<
        unsafe extern "C" fn(
            plugin: *const clap_plugin,
            index: u32,
            note_name: *mut clap_note_name,
        ) -> bool,
    >,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_host_note_name {
    pub changed: Option<unsafe extern "C" fn(host: *const clap_host)>,
}

#[doc = " @page Note Ports\n\n This extension provides a way for the plugin to describe its current note ports.\n If the plugin does not implement this extension, it won't have note input or output.\n The plugin is only allowed to change its note ports configuration while it is deactivated."]
pub const CLAP_EXT_NOTE_PORTS: &CStr = c"clap.note-ports";

pub const clap_note_dialect_CLAP_NOTE_DIALECT_CLAP: clap_note_dialect = 1;
pub const clap_note_dialect_CLAP_NOTE_DIALECT_MIDI: clap_note_dialect = 2;
pub const clap_note_dialect_CLAP_NOTE_DIALECT_MIDI_MPE: clap_note_dialect = 4;
pub const clap_note_dialect_CLAP_NOTE_DIALECT_MIDI2: clap_note_dialect = 8;
pub type clap_note_dialect = c_uint;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_note_port_info {
    pub id: clap_id,
    pub supported_dialects: u32,
    pub preferred_dialect: u32,
    pub name: [c_char; 256usize],
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_plugin_note_ports {
    pub count: Option<unsafe extern "C" fn(plugin: *const clap_plugin, is_input: bool) -> u32>,
    pub get: Option<
        unsafe extern "C" fn(
            plugin: *const clap_plugin,
            index: u32,
            is_input: bool,
            info: *mut clap_note_port_info,
        ) -> bool,
    >,
}

pub const CLAP_NOTE_PORTS_RESCAN_ALL: c_uint = 1;
pub const CLAP_NOTE_PORTS_RESCAN_NAMES: c_uint = 2;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_host_note_ports {
    pub supported_dialects: Option<unsafe extern "C" fn(host: *const clap_host) -> u32>,
    pub rescan: Option<unsafe extern "C" fn(host: *const clap_host, flags: u32)>,
}

#[doc = " @page Parameters\n @brief parameters management\n\n Main idea:\n\n The host sees the plugin as an atomic entity; and acts as a controller on top of its parameters.\n The plugin is responsible for keeping its audio processor and its GUI in sync.\n\n The host can at any time read parameters' value on the [main-thread] using\n @ref clap_plugin_params.get_value().\n\n There are two options to communicate parameter value changes, and they are not concurrent.\n - send automation points during clap_plugin.process()\n - send automation points during clap_plugin_params.flush(), for parameter changes\n   without processing audio\n\n When the plugin changes a parameter value, it must inform the host.\n It will send @ref CLAP_EVENT_PARAM_VALUE event during process() or flush().\n If the user is adjusting the value, don't forget to mark the beginning and end\n of the gesture by sending CLAP_EVENT_PARAM_GESTURE_BEGIN and CLAP_EVENT_PARAM_GESTURE_END\n events.\n\n @note MIDI CCs are tricky because you may not know when the parameter adjustment ends.\n Also if the host records incoming MIDI CC and parameter change automation at the same time,\n there will be a conflict at playback: MIDI CC vs Automation.\n The parameter automation will always target the same parameter because the param_id is stable.\n The MIDI CC may have a different mapping in the future and may result in a different playback.\n\n When a MIDI CC changes a parameter's value, set the flag CLAP_EVENT_DONT_RECORD in\n clap_event_param.header.flags. That way the host may record the MIDI CC automation, but not the\n parameter change and there won't be conflict at playback.\n\n Scenarios:\n\n I. Loading a preset\n - load the preset in a temporary state\n - call @ref clap_host_params.rescan() if anything changed\n - call @ref clap_host_latency.changed() if latency changed\n - invalidate any other info that may be cached by the host\n - if the plugin is activated and the preset will introduce breaking changes\n   (latency, audio ports, new parameters, ...) be sure to wait for the host\n   to deactivate the plugin to apply those changes.\n   If there are no breaking changes, the plugin can apply them them right away.\n   The plugin is responsible for updating both its audio processor and its gui.\n\n II. Turning a knob on the DAW interface\n - the host will send an automation event to the plugin via a process() or flush()\n\n III. Turning a knob on the Plugin interface\n - the plugin is responsible for sending the parameter value to its audio processor\n - call clap_host_params->request_flush() or clap_host->request_process().\n - when the host calls either clap_plugin->process() or clap_plugin_params->flush(),\n   send an automation event and don't forget to wrap the parameter change(s)\n   with CLAP_EVENT_PARAM_GESTURE_BEGIN and CLAP_EVENT_PARAM_GESTURE_END to define the\n   beginning and end of the gesture.\n\n IV. Turning a knob via automation\n - host sends an automation point during clap_plugin->process() or clap_plugin_params->flush().\n - the plugin is responsible for updating its GUI\n\n V. Turning a knob via plugin's internal MIDI mapping\n - the plugin sends a CLAP_EVENT_PARAM_VALUE output event, set should_record to false\n - the plugin is responsible for updating its GUI\n\n VI. Adding or removing parameters\n - if the plugin is activated call clap_host->restart()\n - once the plugin isn't active:\n   - apply the new state\n   - if a parameter is gone or is created with an id that may have been used before,\n     call clap_host_params.clear(host, param_id, CLAP_PARAM_CLEAR_ALL)\n   - call clap_host_params->rescan(CLAP_PARAM_RESCAN_ALL)\n\n CLAP allows the plugin to change the parameter range, yet the plugin developer\n should be aware that doing so isn't without risk, especially if you made the\n promise to never change the sound. If you want to be 100% certain that the\n sound will not change with all host, then simply never change the range.\n\n There are two approaches to automations, either you automate the plain value,\n or you automate the knob position. The first option will be robust to a range\n increase, while the second won't be.\n\n If the host goes with the second approach (automating the knob position), it means\n that the plugin is hosted in a relaxed environment regarding sound changes (they are\n accepted, and not a concern as long as they are reasonable). Though, stepped parameters\n should be stored as plain value in the document.\n\n If the host goes with the first approach, there will still be situation where the\n sound may inevitably change. For example, if the plugin increase the range, there\n is an automation playing at the max value and on top of that an LFO is applied.\n See the following curve:\n                                   .\n                                  . .\n          .....                  .   .\n before: .     .     and after: .     .\n\n Persisting parameter values:\n\n Plugins are responsible for persisting their parameter's values between\n sessions by implementing the state extension. Otherwise parameter value will\n not be recalled when reloading a project. Hosts should _not_ try to save and\n restore parameter values for plugins that don't implement the state\n extension.\n\n Advice for the host:\n\n - store plain values in the document (automation)\n - store modulation amount in plain value delta, not in percentage\n - when you apply a CC mapping, remember the min/max plain values so you can adjust\n - do not implement a parameter saving fall back for plugins that don't\n   implement the state extension\n\n Advice for the plugin:\n\n - think carefully about your parameter range when designing your DSP\n - avoid shrinking parameter ranges, they are very likely to change the sound\n - consider changing the parameter range as a tradeoff: what you improve vs what you break\n - make sure to implement saving and loading the parameter values using the\n   state extension\n - if you plan to use adapters for other plugin formats, then you need to pay extra\n   attention to the adapter requirements"]
pub const CLAP_EXT_PARAMS: &CStr = c"clap.params";

pub const CLAP_PARAM_IS_STEPPED: clap_param_info_flags = 1;
pub const CLAP_PARAM_IS_PERIODIC: clap_param_info_flags = 2;
pub const CLAP_PARAM_IS_HIDDEN: clap_param_info_flags = 4;
pub const CLAP_PARAM_IS_READONLY: clap_param_info_flags = 8;
pub const CLAP_PARAM_IS_BYPASS: clap_param_info_flags = 16;
pub const CLAP_PARAM_IS_AUTOMATABLE: clap_param_info_flags = 32;
pub const CLAP_PARAM_IS_AUTOMATABLE_PER_NOTE_ID: clap_param_info_flags = 64;
pub const CLAP_PARAM_IS_AUTOMATABLE_PER_KEY: clap_param_info_flags = 128;
pub const CLAP_PARAM_IS_AUTOMATABLE_PER_CHANNEL: clap_param_info_flags = 256;
pub const CLAP_PARAM_IS_AUTOMATABLE_PER_PORT: clap_param_info_flags = 512;
pub const CLAP_PARAM_IS_MODULATABLE: clap_param_info_flags = 1024;
pub const CLAP_PARAM_IS_MODULATABLE_PER_NOTE_ID: clap_param_info_flags = 2048;
pub const CLAP_PARAM_IS_MODULATABLE_PER_KEY: clap_param_info_flags = 4096;
pub const CLAP_PARAM_IS_MODULATABLE_PER_CHANNEL: clap_param_info_flags = 8192;
pub const CLAP_PARAM_IS_MODULATABLE_PER_PORT: clap_param_info_flags = 16384;
pub const CLAP_PARAM_REQUIRES_PROCESS: clap_param_info_flags = 32768;
pub const CLAP_PARAM_IS_ENUM: clap_param_info_flags = 65536;
pub type clap_param_info_flags = u32;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_param_info {
    pub id: clap_id,
    pub flags: clap_param_info_flags,
    pub cookie: *mut c_void,
    pub name: [c_char; 256usize],
    pub module: [c_char; 1024usize],
    pub min_value: f64,
    pub max_value: f64,
    pub default_value: f64,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_plugin_params {
    pub count: Option<unsafe extern "C" fn(plugin: *const clap_plugin) -> u32>,
    pub get_info: Option<
        unsafe extern "C" fn(
            plugin: *const clap_plugin,
            param_index: u32,
            param_info: *mut clap_param_info,
        ) -> bool,
    >,
    pub get_value: Option<
        unsafe extern "C" fn(
            plugin: *const clap_plugin,
            param_id: clap_id,
            out_value: *mut f64,
        ) -> bool,
    >,
    pub value_to_text: Option<
        unsafe extern "C" fn(
            plugin: *const clap_plugin,
            param_id: clap_id,
            value: f64,
            out_buffer: *mut c_char,
            out_buffer_capacity: u32,
        ) -> bool,
    >,
    pub text_to_value: Option<
        unsafe extern "C" fn(
            plugin: *const clap_plugin,
            param_id: clap_id,
            param_value_text: *const c_char,
            out_value: *mut f64,
        ) -> bool,
    >,
    pub flush: Option<
        unsafe extern "C" fn(
            plugin: *const clap_plugin,
            in_: *const clap_input_events,
            out: *const clap_output_events,
        ),
    >,
}

pub const CLAP_PARAM_RESCAN_VALUES: clap_param_rescan_flags = 1;
pub const CLAP_PARAM_RESCAN_TEXT: clap_param_rescan_flags = 2;
pub const CLAP_PARAM_RESCAN_INFO: clap_param_rescan_flags = 4;
pub const CLAP_PARAM_RESCAN_ALL: clap_param_rescan_flags = 8;
pub type clap_param_rescan_flags = u32;

pub const CLAP_PARAM_CLEAR_ALL: clap_param_clear_flags = 1;
pub const CLAP_PARAM_CLEAR_AUTOMATIONS: clap_param_clear_flags = 2;
pub const CLAP_PARAM_CLEAR_MODULATIONS: clap_param_clear_flags = 4;
pub type clap_param_clear_flags = u32;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_host_params {
    pub rescan:
        Option<unsafe extern "C" fn(host: *const clap_host, flags: clap_param_rescan_flags)>,
    pub clear: Option<
        unsafe extern "C" fn(
            host: *const clap_host,
            param_id: clap_id,
            flags: clap_param_clear_flags,
        ),
    >,
    pub request_flush: Option<unsafe extern "C" fn(host: *const clap_host)>,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_color {
    pub alpha: u8,
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

pub const CLAP_COLOR_TRANSPARENT: clap_color = clap_color {
    alpha: 0,
    red: 0,
    green: 0,
    blue: 0,
};

pub const CLAP_EXT_PARAM_INDICATION: &CStr = c"clap.param-indication/4";
pub const CLAP_EXT_PARAM_INDICATION_COMPAT: &CStr = c"clap.param-indication.draft/4";

pub const CLAP_PARAM_INDICATION_AUTOMATION_NONE: c_uint = 0;
pub const CLAP_PARAM_INDICATION_AUTOMATION_PRESENT: c_uint = 1;
pub const CLAP_PARAM_INDICATION_AUTOMATION_PLAYING: c_uint = 2;
pub const CLAP_PARAM_INDICATION_AUTOMATION_RECORDING: c_uint = 3;
pub const CLAP_PARAM_INDICATION_AUTOMATION_OVERRIDING: c_uint = 4;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_plugin_param_indication {
    pub set_mapping: Option<
        unsafe extern "C" fn(
            plugin: *const clap_plugin,
            param_id: clap_id,
            has_mapping: bool,
            color: *const clap_color,
            label: *const c_char,
            description: *const c_char,
        ),
    >,
    pub set_automation: Option<
        unsafe extern "C" fn(
            plugin: *const clap_plugin,
            param_id: clap_id,
            automation_state: u32,
            color: *const clap_color,
        ),
    >,
}

pub const CLAP_EXT_POSIX_FD_SUPPORT: &CStr = c"clap.posix-fd-support";

pub const CLAP_POSIX_FD_READ: clap_posix_fd_flags = 1;
pub const CLAP_POSIX_FD_WRITE: clap_posix_fd_flags = 2;
pub const CLAP_POSIX_FD_ERROR: clap_posix_fd_flags = 4;
pub type clap_posix_fd_flags = u32;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_plugin_posix_fd_support {
    pub on_fd: Option<
        unsafe extern "C" fn(plugin: *const clap_plugin, fd: c_int, flags: clap_posix_fd_flags),
    >,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_host_posix_fd_support {
    pub register_fd: Option<
        unsafe extern "C" fn(host: *const clap_host, fd: c_int, flags: clap_posix_fd_flags) -> bool,
    >,
    pub modify_fd: Option<
        unsafe extern "C" fn(host: *const clap_host, fd: c_int, flags: clap_posix_fd_flags) -> bool,
    >,
    pub unregister_fd: Option<unsafe extern "C" fn(host: *const clap_host, fd: c_int) -> bool>,
}

pub const CLAP_EXT_PRESET_LOAD: &CStr = c"clap.preset-load/2";
pub const CLAP_EXT_PRESET_LOAD_COMPAT: &CStr = c"clap.preset-load.draft/2";

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_plugin_preset_load {
    pub from_location: Option<
        unsafe extern "C" fn(
            plugin: *const clap_plugin,
            location_kind: u32,
            location: *const c_char,
            load_key: *const c_char,
        ) -> bool,
    >,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_host_preset_load {
    pub on_error: Option<
        unsafe extern "C" fn(
            host: *const clap_host,
            location_kind: u32,
            location: *const c_char,
            load_key: *const c_char,
            os_error: i32,
            msg: *const c_char,
        ),
    >,
    pub loaded: Option<
        unsafe extern "C" fn(
            host: *const clap_host,
            location_kind: u32,
            location: *const c_char,
            load_key: *const c_char,
        ),
    >,
}

pub const CLAP_EXT_REMOTE_CONTROLS: &CStr = c"clap.remote-controls/2";
pub const CLAP_EXT_REMOTE_CONTROLS_COMPAT: &CStr = c"clap.remote-controls.draft/2";

pub const CLAP_REMOTE_CONTROLS_COUNT: c_uint = 8;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_remote_controls_page {
    pub section_name: [c_char; 256usize],
    pub page_id: clap_id,
    pub page_name: [c_char; 256usize],
    pub param_ids: [clap_id; 8usize],
    pub is_for_preset: bool,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_plugin_remote_controls {
    pub count: Option<unsafe extern "C" fn(plugin: *const clap_plugin) -> u32>,
    pub get: Option<
        unsafe extern "C" fn(
            plugin: *const clap_plugin,
            page_index: u32,
            page: *mut clap_remote_controls_page,
        ) -> bool,
    >,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_host_remote_controls {
    pub changed: Option<unsafe extern "C" fn(host: *const clap_host)>,
    pub suggest_page: Option<unsafe extern "C" fn(host: *const clap_host, page_id: clap_id)>,
}

pub const CLAP_EXT_RENDER: &CStr = c"clap.render";
pub const CLAP_RENDER_REALTIME: clap_plugin_render_mode = 0;
pub const CLAP_RENDER_OFFLINE: clap_plugin_render_mode = 1;
pub type clap_plugin_render_mode = i32;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_plugin_render {
    pub has_hard_realtime_requirement:
        Option<unsafe extern "C" fn(plugin: *const clap_plugin) -> bool>,
    pub set: Option<
        unsafe extern "C" fn(plugin: *const clap_plugin, mode: clap_plugin_render_mode) -> bool,
    >,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_istream {
    pub ctx: *mut c_void,
    pub read: Option<
        unsafe extern "C" fn(stream: *const clap_istream, buffer: *mut c_void, size: u64) -> i64,
    >,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_ostream {
    pub ctx: *mut c_void,
    pub write: Option<
        unsafe extern "C" fn(stream: *const clap_ostream, buffer: *const c_void, size: u64) -> i64,
    >,
}

pub const CLAP_EXT_STATE_CONTEXT: &CStr = c"clap.state-context/2";

pub const clap_plugin_state_context_type_CLAP_STATE_CONTEXT_FOR_PRESET:
    clap_plugin_state_context_type = 1;
pub const clap_plugin_state_context_type_CLAP_STATE_CONTEXT_FOR_DUPLICATE:
    clap_plugin_state_context_type = 2;
pub const clap_plugin_state_context_type_CLAP_STATE_CONTEXT_FOR_PROJECT:
    clap_plugin_state_context_type = 3;
pub type clap_plugin_state_context_type = c_uint;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_plugin_state_context {
    pub save: Option<
        unsafe extern "C" fn(
            plugin: *const clap_plugin,
            stream: *const clap_ostream,
            context_type: u32,
        ) -> bool,
    >,
    pub load: Option<
        unsafe extern "C" fn(
            plugin: *const clap_plugin,
            stream: *const clap_istream,
            context_type: u32,
        ) -> bool,
    >,
}

#[doc = " @page State\n @brief state management\n\n Plugins can implement this extension to save and restore both parameter\n values and non-parameter state. This is used to persist a plugin's state\n between project reloads, when duplicating and copying plugin instances, and\n for host-side preset management.\n\n If you need to know if the save/load operation is meant for duplicating a plugin\n instance, for saving/loading a plugin preset or while saving/loading the project\n then consider implementing CLAP_EXT_STATE_CONTEXT in addition to CLAP_EXT_STATE."]
pub const CLAP_EXT_STATE: &CStr = c"clap.state";

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_plugin_state {
    pub save: Option<
        unsafe extern "C" fn(plugin: *const clap_plugin, stream: *const clap_ostream) -> bool,
    >,
    pub load: Option<
        unsafe extern "C" fn(plugin: *const clap_plugin, stream: *const clap_istream) -> bool,
    >,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_host_state {
    pub mark_dirty: Option<unsafe extern "C" fn(host: *const clap_host)>,
}

pub const CLAP_EXT_SURROUND: &CStr = c"clap.surround/4";
pub const CLAP_EXT_SURROUND_COMPAT: &CStr = c"clap.surround.draft/4";
pub const CLAP_PORT_SURROUND: &CStr = c"surround";

pub const CLAP_SURROUND_FL: c_uint = 0;
pub const CLAP_SURROUND_FR: c_uint = 1;
pub const CLAP_SURROUND_FC: c_uint = 2;
pub const CLAP_SURROUND_LFE: c_uint = 3;
pub const CLAP_SURROUND_BL: c_uint = 4;
pub const CLAP_SURROUND_BR: c_uint = 5;
pub const CLAP_SURROUND_FLC: c_uint = 6;
pub const CLAP_SURROUND_FRC: c_uint = 7;
pub const CLAP_SURROUND_BC: c_uint = 8;
pub const CLAP_SURROUND_SL: c_uint = 9;
pub const CLAP_SURROUND_SR: c_uint = 10;
pub const CLAP_SURROUND_TC: c_uint = 11;
pub const CLAP_SURROUND_TFL: c_uint = 12;
pub const CLAP_SURROUND_TFC: c_uint = 13;
pub const CLAP_SURROUND_TFR: c_uint = 14;
pub const CLAP_SURROUND_TBL: c_uint = 15;
pub const CLAP_SURROUND_TBC: c_uint = 16;
pub const CLAP_SURROUND_TBR: c_uint = 17;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_plugin_surround {
    pub is_channel_mask_supported:
        Option<unsafe extern "C" fn(plugin: *const clap_plugin, channel_mask: u64) -> bool>,
    pub get_channel_map: Option<
        unsafe extern "C" fn(
            plugin: *const clap_plugin,
            is_input: bool,
            port_index: u32,
            channel_map: *mut u8,
            channel_map_capacity: u32,
        ) -> u32,
    >,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_host_surround {
    pub changed: Option<unsafe extern "C" fn(host: *const clap_host)>,
}

pub const CLAP_EXT_TAIL: &CStr = c"clap.tail";

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_pluginail {
    pub get: Option<unsafe extern "C" fn(plugin: *const clap_plugin) -> u32>,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_hostail {
    pub changed: Option<unsafe extern "C" fn(host: *const clap_host)>,
}

pub const CLAP_EXT_THREAD_CHECK: &CStr = c"clap.thread-check";

#[doc = " @page thread-check\n\n CLAP defines two symbolic threads:\n\n main-thread:\n    This is the thread in which most of the interaction between the plugin and host happens.\n    This will be the same OS thread throughout the lifetime of the plug-in.\n    On macOS and Windows, this must be the thread on which gui and timer events are received\n    (i.e., the main thread of the program).\n    It isn't a realtime thread, yet this thread needs to respond fast enough to allow responsive\n    user interaction, so it is strongly recommended plugins run long,and expensive or blocking\n    tasks such as preset indexing or asset loading in dedicated background threads started by the\n    plugin.\n\n audio-thread:\n    This thread can be used for realtime audio processing. Its execution should be as\n    deterministic as possible to meet the audio interface's deadline (can be <1ms). There are a\n    known set of operations that should be avoided: malloc() and free(), contended locks and\n    mutexes, I/O, waiting, and so forth.\n\n    The audio-thread is symbolic, there isn't one OS thread that remains the\n    audio-thread for the plugin lifetime. A host is may opt to have a\n    thread pool and the plugin.process() call may be scheduled on different OS threads over time.\n    However, the host must guarantee that single plugin instance will not be two audio-threads\n    at the same time.\n\n    Functions marked with [audio-thread] **ARE NOT CONCURRENT**. The host may mark any OS thread,\n    including the main-thread as the audio-thread, as long as it can guarantee that only one OS\n    thread is the audio-thread at a time in a plugin instance. The audio-thread can be seen as a\n    concurrency guard for all functions marked with [audio-thread].\n\n    The real-time constraint on the [audio-thread] interacts closely with the render extension.\n    If a plugin doesn't implement render, then that plugin must have all [audio-thread] functions\n    meet the real time standard. If the plugin does implement render, and returns true when\n    render mode is set to real-time or if the plugin advertises a hard realtime requirement, it\n    must implement realtime constraints. Hosts also provide functions marked [audio-thread].\n    These can be safely called by a plugin in the audio thread. Therefore hosts must either (1)\n    implement those functions meeting the real-time constraints or (2) not process plugins which\n    advertise a hard realtime constraint or don't implement the render extension. Hosts which\n    provide [audio-thread] functions outside these conditions may experience inconsistent or\n    inaccurate rendering.\n\n  Clap also tags some functions as [thread-safe]. Functions tagged as [thread-safe] can be called\n  from any thread unless explicitly counter-indicated (for instance [thread-safe, !audio-thread])\n  and may be called concurrently. Since a [thread-safe] function may be called from the\n  [audio-thread] unless explicitly counter-indicated, it must also meet the realtime constraints\n  as describes above."]
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_hosthread_check {
    pub is_main_thread: Option<unsafe extern "C" fn(host: *const clap_host) -> bool>,
    pub is_audio_thread: Option<unsafe extern "C" fn(host: *const clap_host) -> bool>,
}

#[doc = " @page\n\n This extension lets the plugin use the host's thread pool.\n\n The plugin must provide @ref clap_pluginhread_pool, and the host may provide @ref\n clap_hosthread_pool. If it doesn't, the plugin should process its data by its own means. In\n the worst case, a single threaded for-loop.\n\n Simple example with N voices to process\n\n @code\n void myplug_thread_pool_exec(const clap_plugin *plugin, uint32_t voice_index)\n {\n    compute_voice(plugin, voice_index);\n }\n\n void myplug_process(const clap_plugin *plugin, const clap_process *process)\n {\n    ...\n    bool didComputeVoices = false;\n    if (host_thread_pool && host_thread_pool.exec)\n       didComputeVoices = host_thread_pool.request_exec(host, plugin, N);\n\n    if (!didComputeVoices)\n       for (uint32_t i = 0; i < N; ++i)\n          myplug_thread_pool_exec(plugin, i);\n    ...\n }\n @endcode\n\n Be aware that using a thread pool may break hard real-time rules due to the thread\n synchronization involved.\n\n If the host knows that it is running under hard real-time pressure it may decide to not\n provide this interface."]
pub const CLAP_EXT_THREAD_POOL: &CStr = c"clap.thread-pool";

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_pluginhread_pool {
    pub exec: Option<unsafe extern "C" fn(plugin: *const clap_plugin, task_index: u32)>,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_hosthread_pool {
    pub request_exec: Option<unsafe extern "C" fn(host: *const clap_host, num_tasks: u32) -> bool>,
}

pub const CLAP_EXT_TIMER_SUPPORT: &CStr = c"clap.timer-support";

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_pluginimer_support {
    pub on_timer: Option<unsafe extern "C" fn(plugin: *const clap_plugin, timer_id: clap_id)>,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_hostimer_support {
    pub register_timer: Option<
        unsafe extern "C" fn(
            host: *const clap_host,
            period_ms: u32,
            timer_id: *mut clap_id,
        ) -> bool,
    >,
    pub unregister_timer:
        Option<unsafe extern "C" fn(host: *const clap_host, timer_id: clap_id) -> bool>,
}

pub const CLAP_EXT_TRACK_INFO: &CStr = c"clap.track-info/1";
pub const CLAP_EXT_TRACK_INFO_COMPAT: &CStr = c"clap.track-info.draft/1";

pub const CLAP_TRACK_INFO_HAS_TRACK_NAME: c_uint = 1;
pub const CLAP_TRACK_INFO_HAS_TRACK_COLOR: c_uint = 2;
pub const CLAP_TRACK_INFO_HAS_AUDIO_CHANNEL: c_uint = 4;
pub const CLAP_TRACK_INFO_IS_FOR_RETURN_TRACK: c_uint = 8;
pub const CLAP_TRACK_INFO_IS_FOR_BUS: c_uint = 16;
pub const CLAP_TRACK_INFO_IS_FOR_MASTER: c_uint = 32;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_track_info {
    pub flags: u64,
    pub name: [c_char; 256usize],
    pub color: clap_color,
    pub audio_channel_count: i32,
    pub audio_port_type: *const c_char,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_pluginrack_info {
    pub changed: Option<unsafe extern "C" fn(plugin: *const clap_plugin)>,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_hostrack_info {
    pub get:
        Option<unsafe extern "C" fn(host: *const clap_host, info: *mut clap_track_info) -> bool>,
}

pub const CLAP_EXT_VOICE_INFO: &CStr = c"clap.voice-info";

pub const CLAP_VOICE_INFO_SUPPORTS_OVERLAPPING_NOTES: c_uint = 1;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_voice_info {
    pub voice_count: u32,
    pub voice_capacity: u32,
    pub flags: u64,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_plugin_voice_info {
    pub get: Option<
        unsafe extern "C" fn(plugin: *const clap_plugin, info: *mut clap_voice_info) -> bool,
    >,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_host_voice_info {
    pub changed: Option<unsafe extern "C" fn(host: *const clap_host)>,
}
