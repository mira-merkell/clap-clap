use std::ffi::CStr;

mod ffi {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    #![allow(warnings, unused)]
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

pub use crate::ffi::{
    CLAP_AUDIO_PORT_IS_MAIN, CLAP_AUDIO_PORT_PREFERS_64BITS,
    CLAP_AUDIO_PORT_REQUIRES_COMMON_SAMPLE_SIZE, CLAP_AUDIO_PORT_SUPPORTS_64BITS,
    CLAP_VERSION_MAJOR, CLAP_VERSION_MINOR, CLAP_VERSION_REVISION, clap_audio_buffer,
    clap_audio_port_info, clap_host, clap_plugin, clap_plugin_audio_ports, clap_plugin_descriptor,
    clap_plugin_entry, clap_plugin_factory, clap_process, clap_process_status, clap_version, CLAP_INVALID_ID,
};

pub const CLAP_VERSION: clap_version = clap_version {
    major: CLAP_VERSION_MAJOR,
    minor: CLAP_VERSION_MINOR,
    revision: CLAP_VERSION_REVISION,
};

macro_rules! clap_process_status_const {
    ($($name:ident),*) => {
        $(
            pub const $name: clap_process_status =
                    ffi::$name as clap_process_status;
        )*
    };
}

clap_process_status_const!(
    CLAP_PROCESS_ERROR,
    CLAP_PROCESS_CONTINUE,
    CLAP_PROCESS_CONTINUE_IF_NOT_QUIET,
    CLAP_PROCESS_TAIL,
    CLAP_PROCESS_SLEEP
);

macro_rules! export_cstr_from_bytes {
    ($($name_id:ident),*) => {$(
        pub const $name_id: &CStr = unsafe { CStr::from_bytes_with_nul_unchecked(ffi::$name_id) };
    )*}
}

export_cstr_from_bytes!(CLAP_PLUGIN_FACTORY_ID);

export_cstr_from_bytes!(
    CLAP_EXT_AUDIO_PORTS,
    CLAP_PORT_MONO,
    CLAP_PORT_STEREO,
    CLAP_PORT_SURROUND,
    CLAP_PORT_AMBISONIC
);

export_cstr_from_bytes!(
    CLAP_PLUGIN_FEATURE_INSTRUMENT,
    CLAP_PLUGIN_FEATURE_AUDIO_EFFECT,
    CLAP_PLUGIN_FEATURE_NOTE_EFFECT,
    CLAP_PLUGIN_FEATURE_NOTE_DETECTOR,
    CLAP_PLUGIN_FEATURE_ANALYZER,
    CLAP_PLUGIN_FEATURE_SYNTHESIZER,
    CLAP_PLUGIN_FEATURE_SAMPLER,
    CLAP_PLUGIN_FEATURE_DRUM,
    CLAP_PLUGIN_FEATURE_DRUM_MACHINE,
    CLAP_PLUGIN_FEATURE_FILTER,
    CLAP_PLUGIN_FEATURE_PHASER,
    CLAP_PLUGIN_FEATURE_EQUALIZER,
    CLAP_PLUGIN_FEATURE_DEESSER,
    CLAP_PLUGIN_FEATURE_PHASE_VOCODER,
    CLAP_PLUGIN_FEATURE_GRANULAR,
    CLAP_PLUGIN_FEATURE_FREQUENCY_SHIFTER,
    CLAP_PLUGIN_FEATURE_PITCH_SHIFTER,
    CLAP_PLUGIN_FEATURE_DISTORTION,
    CLAP_PLUGIN_FEATURE_TRANSIENT_SHAPER,
    CLAP_PLUGIN_FEATURE_COMPRESSOR,
    CLAP_PLUGIN_FEATURE_EXPANDER,
    CLAP_PLUGIN_FEATURE_GATE,
    CLAP_PLUGIN_FEATURE_LIMITER,
    CLAP_PLUGIN_FEATURE_FLANGER,
    CLAP_PLUGIN_FEATURE_CHORUS,
    CLAP_PLUGIN_FEATURE_DELAY,
    CLAP_PLUGIN_FEATURE_REVERB,
    CLAP_PLUGIN_FEATURE_TREMOLO,
    CLAP_PLUGIN_FEATURE_GLITCH,
    CLAP_PLUGIN_FEATURE_UTILITY,
    CLAP_PLUGIN_FEATURE_PITCH_CORRECTION,
    CLAP_PLUGIN_FEATURE_RESTORATION,
    CLAP_PLUGIN_FEATURE_MULTI_EFFECTS,
    CLAP_PLUGIN_FEATURE_MIXING,
    CLAP_PLUGIN_FEATURE_MASTERING,
    CLAP_PLUGIN_FEATURE_MONO,
    CLAP_PLUGIN_FEATURE_STEREO,
    CLAP_PLUGIN_FEATURE_SURROUND,
    CLAP_PLUGIN_FEATURE_AMBISONIC
);
