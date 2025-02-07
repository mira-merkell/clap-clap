macro_rules! plugin_feature_from_cstr {
    ($($feature:tt),*) => {
        $(
            #[allow(non_snake_case)]
            #[test]
            fn $feature() {
                use clap_clap::plugin_features::$feature;

                let feat = clap_clap::ffi::$feature.to_str().unwrap();
                assert_eq!($feature, feat);
            }
        )*
    };
}

plugin_feature_from_cstr!(
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
