use std::ffi::CStr;

mod ffi {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    #![allow(warnings, unused)]
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

// Export raw, null-terminated byte strings as CStr.
//
// Safety:
// The symbols exported must be static, null-terminated byte strings from ffi.
macro_rules! export_cstr_from_bytes {
    ($($name_id:ident),*) => { $(
        pub const $name_id: &CStr = unsafe { CStr::from_ptr(ffi::$name_id.as_ptr() as *const _) };

        #[allow(non_snake_case)]
        #[cfg(test)]
        mod $name_id {
            use std::ffi::CStr;
            use super::{ffi, CLAP_NAME_SIZE, $name_id};

            #[test]
            fn export_cstr_from_bytes () {
                let bytes: Vec<u8> = ffi::$name_id.iter().map(|&b| b as u8).collect();
                let _ = CStr::from_bytes_with_nul(&bytes).expect("should be a valid CStr");
            }

            #[test]
            fn is_static() {
                const fn borrow_static() -> &'static [u8] {
                    ffi::$name_id
                }

                let _ = borrow_static();
            }

            #[test]
            fn is_valid_rust_string() {
                let _ = $name_id.to_str().expect("should be valid Rust string");
            }

            #[test]
            fn len_is_less_than_clap_name_size() {
                assert!($name_id.to_str().unwrap().len() < CLAP_NAME_SIZE);
            }
        }
    )* }
}


macro_rules! cast_const_as_usize {
    ($($name:ident),*) => {$(
        pub const $name: usize = ffi::$name as usize;

        #[allow(non_snake_case)]
        #[cfg(test)]
        mod $name {
            use super::ffi;

            #[test]
            fn cast_as_usize() {
                usize::try_from(ffi::$name).expect("should fit into usize");
            }
        }
    )*};
}


cast_const_as_usize!(CLAP_NAME_SIZE, CLAP_PATH_SIZE);

// CLAP id
pub use crate::ffi::{CLAP_INVALID_ID, clap_id};

// CLAP version
pub use crate::ffi::{CLAP_VERSION_MAJOR, CLAP_VERSION_MINOR, CLAP_VERSION_REVISION, clap_version};

pub const CLAP_VERSION: clap_version = clap_version {
    major: CLAP_VERSION_MAJOR,
    minor: CLAP_VERSION_MINOR,
    revision: CLAP_VERSION_REVISION,
};

// CLAP entry
pub use crate::ffi::{
    clap_host, clap_plugin, clap_plugin_descriptor, clap_plugin_entry, clap_plugin_factory,
};

// CLAP process
pub use crate::ffi::{clap_audio_buffer, clap_process, clap_process_status};

// Export CLAP_PROCESS_* enum as clap_process_status
macro_rules! clap_process_status_const {
    ($($name:ident),*) => {
        $(
            pub const $name: clap_process_status =
                    ffi::$name as clap_process_status;


            #[allow(non_snake_case)]
            #[cfg(test)]
            mod $name {
                use super::{$name, clap_process_status};

                #[test]
                fn cast_as_clap_process_status() {
                    let _ : clap_process_status = $name.try_into()
                        .expect("should fit into clap_process_status");
                }
            }
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

// CLAP plugin factory
export_cstr_from_bytes!(CLAP_PLUGIN_FACTORY_ID);

// CLAP plugin extension: audio_ports
pub use ffi::{clap_audio_port_info, clap_plugin_audio_ports};

macro_rules! cast_flags_as_u32 {
    ($($flag:ident),*) => {
        $(
            pub const $flag: u32 = ffi::$flag as u32;

            #[allow(non_snake_case)]
            #[cfg(test)]
            mod $flag {
                use super::ffi;

                #[test]
                fn cast_as_u32() {
                    u32::try_from(ffi::$flag).expect("should fit into u32");
                }
            }
        )*
    };
}

cast_flags_as_u32!(
    CLAP_AUDIO_PORT_IS_MAIN,
    CLAP_AUDIO_PORT_PREFERS_64BITS,
    CLAP_AUDIO_PORT_REQUIRES_COMMON_SAMPLE_SIZE,
    CLAP_AUDIO_PORT_SUPPORTS_64BITS
);

export_cstr_from_bytes!(
    CLAP_EXT_AUDIO_PORTS,
    CLAP_PORT_MONO,
    CLAP_PORT_STEREO,
    CLAP_PORT_SURROUND,
    CLAP_PORT_AMBISONIC
);

// CLAP_PLUGIN_FEATURE_*
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

// clap_log

pub use ffi::clap_host_log;
pub use ffi::clap_log_severity;

macro_rules! cast_const_as_clap_log_severity {
    ($($name:ident),*) => {
        $(
            pub const $name: clap_log_severity = ffi::$name as clap_log_severity;

            #[allow(non_snake_case)]
            #[cfg(test)]
            mod $name {
                use super::*;

                #[test]
                fn cast_as_clap_log_severity() {
                    clap_log_severity::try_from(ffi::$name).expect("value should fit into clap_log_severity");
                }
            }
        )*
    };
}

cast_const_as_clap_log_severity!(
    CLAP_LOG_FATAL,
    CLAP_LOG_ERROR,
    CLAP_LOG_WARNING,
    CLAP_LOG_INFO,
    CLAP_LOG_DEBUG,
    CLAP_LOG_HOST_MISBEHAVING,
    CLAP_LOG_PLUGIN_MISBEHAVING
);
export_cstr_from_bytes!(CLAP_EXT_LOG);
