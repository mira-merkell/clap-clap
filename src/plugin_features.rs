//! Standard plugin features.
//!
//! For practical reasons we avoid spaces and use `-` instead to facilitate
//! scripts that generate the feature array.
//!
//! Non-standard features should be formatted as follows: "$namespace:$feature"
//!
//! See also: [`Plugin::features`] how to define plugin features as an arbitrary
//! list of keywords.
//!
//! [`Plugin::features`]: crate::plugin::Plugin::features

pub const CLAP_PLUGIN_FEATURE_INSTRUMENT: &str = "instrument";
pub const CLAP_PLUGIN_FEATURE_AUDIO_EFFECT: &str = "audio-effect";
pub const CLAP_PLUGIN_FEATURE_NOTE_EFFECT: &str = "note-effect";
pub const CLAP_PLUGIN_FEATURE_NOTE_DETECTOR: &str = "note-detector";
pub const CLAP_PLUGIN_FEATURE_ANALYZER: &str = "analyzer";

pub const CLAP_PLUGIN_FEATURE_SYNTHESIZER: &str = "synthesizer";
pub const CLAP_PLUGIN_FEATURE_SAMPLER: &str = "sampler";
pub const CLAP_PLUGIN_FEATURE_DRUM: &str = "drum";
pub const CLAP_PLUGIN_FEATURE_DRUM_MACHINE: &str = "drum-machine";

pub const CLAP_PLUGIN_FEATURE_FILTER: &str = "filter";
pub const CLAP_PLUGIN_FEATURE_PHASER: &str = "phaser";
pub const CLAP_PLUGIN_FEATURE_EQUALIZER: &str = "equalizer";
pub const CLAP_PLUGIN_FEATURE_DEESSER: &str = "de-esser";
pub const CLAP_PLUGIN_FEATURE_PHASE_VOCODER: &str = "phase-vocoder";
pub const CLAP_PLUGIN_FEATURE_GRANULAR: &str = "granular";
pub const CLAP_PLUGIN_FEATURE_FREQUENCY_SHIFTER: &str = "frequency-shifter";
pub const CLAP_PLUGIN_FEATURE_PITCH_SHIFTER: &str = "pitch-shifter";

pub const CLAP_PLUGIN_FEATURE_DISTORTION: &str = "distortion";
pub const CLAP_PLUGIN_FEATURE_TRANSIENT_SHAPER: &str = "transient-shaper";
pub const CLAP_PLUGIN_FEATURE_COMPRESSOR: &str = "compressor";
pub const CLAP_PLUGIN_FEATURE_EXPANDER: &str = "expander";
pub const CLAP_PLUGIN_FEATURE_GATE: &str = "gate";
pub const CLAP_PLUGIN_FEATURE_LIMITER: &str = "limiter";

pub const CLAP_PLUGIN_FEATURE_FLANGER: &str = "flanger";
pub const CLAP_PLUGIN_FEATURE_CHORUS: &str = "chorus";
pub const CLAP_PLUGIN_FEATURE_DELAY: &str = "delay";
pub const CLAP_PLUGIN_FEATURE_REVERB: &str = "reverb";

pub const CLAP_PLUGIN_FEATURE_TREMOLO: &str = "tremolo";
pub const CLAP_PLUGIN_FEATURE_GLITCH: &str = "glitch";

pub const CLAP_PLUGIN_FEATURE_UTILITY: &str = "utility";
pub const CLAP_PLUGIN_FEATURE_PITCH_CORRECTION: &str = "pitch-correction";
pub const CLAP_PLUGIN_FEATURE_RESTORATION: &str = "restoration";

pub const CLAP_PLUGIN_FEATURE_MULTI_EFFECTS: &str = "multi-effects";

pub const CLAP_PLUGIN_FEATURE_MIXING: &str = "mixing";
pub const CLAP_PLUGIN_FEATURE_MASTERING: &str = "mastering";

pub const CLAP_PLUGIN_FEATURE_MONO: &str = "mono";
pub const CLAP_PLUGIN_FEATURE_STEREO: &str = "stereo";
pub const CLAP_PLUGIN_FEATURE_SURROUND: &str = "surround";
pub const CLAP_PLUGIN_FEATURE_AMBISONIC: &str = "ambisonic";
