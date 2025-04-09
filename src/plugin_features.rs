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

pub const INSTRUMENT: &str = "instrument";
pub const AUDIO_EFFECT: &str = "audio-effect";
pub const NOTE_EFFECT: &str = "note-effect";
pub const NOTE_DETECTOR: &str = "note-detector";
pub const ANALYZER: &str = "analyzer";

pub const SYNTHESIZER: &str = "synthesizer";
pub const SAMPLER: &str = "sampler";
pub const DRUM: &str = "drum";
pub const DRUM_MACHINE: &str = "drum-machine";

pub const FILTER: &str = "filter";
pub const PHASER: &str = "phaser";
pub const EQUALIZER: &str = "equalizer";
pub const DEESSER: &str = "de-esser";
pub const PHASE_VOCODER: &str = "phase-vocoder";
pub const GRANULAR: &str = "granular";
pub const FREQUENCY_SHIFTER: &str = "frequency-shifter";
pub const PITCH_SHIFTER: &str = "pitch-shifter";

pub const DISTORTION: &str = "distortion";
pub const TRANSIENT_SHAPER: &str = "transient-shaper";
pub const COMPRESSOR: &str = "compressor";
pub const EXPANDER: &str = "expander";
pub const GATE: &str = "gate";
pub const LIMITER: &str = "limiter";

pub const FLANGER: &str = "flanger";
pub const CHORUS: &str = "chorus";
pub const DELAY: &str = "delay";
pub const REVERB: &str = "reverb";

pub const TREMOLO: &str = "tremolo";
pub const GLITCH: &str = "glitch";

pub const UTILITY: &str = "utility";
pub const PITCH_CORRECTION: &str = "pitch-correction";
pub const RESTORATION: &str = "restoration";

pub const MULTI_EFFECTS: &str = "multi-effects";

pub const MIXING: &str = "mixing";
pub const MASTERING: &str = "mastering";

pub const MONO: &str = "mono";
pub const STEREO: &str = "stereo";
pub const SURROUND: &str = "surround";
pub const AMBISONIC: &str = "ambisonic";
