macro_rules! plugin_feature_from_cstr {
    ($(($feature:tt, $clap_feature:tt)),* $(,)?) => {
        $(
            #[allow(non_snake_case)]
            #[test]
            fn $feature() {
                use clap_clap::plugin_features::$feature;

                let feat = clap_clap::ffi::$clap_feature.to_str().unwrap();
                assert_eq!($feature, feat);
            }
        )*
    };
}

plugin_feature_from_cstr!(
    (INSTRUMENT, CLAP_PLUGIN_FEATURE_INSTRUMENT),
    (AUDIO_EFFECT, CLAP_PLUGIN_FEATURE_AUDIO_EFFECT),
    (NOTE_EFFECT, CLAP_PLUGIN_FEATURE_NOTE_EFFECT),
    (NOTE_DETECTOR, CLAP_PLUGIN_FEATURE_NOTE_DETECTOR),
    (ANALYZER, CLAP_PLUGIN_FEATURE_ANALYZER),
    (SYNTHESIZER, CLAP_PLUGIN_FEATURE_SYNTHESIZER),
    (SAMPLER, CLAP_PLUGIN_FEATURE_SAMPLER),
    (DRUM, CLAP_PLUGIN_FEATURE_DRUM),
    (DRUM_MACHINE, CLAP_PLUGIN_FEATURE_DRUM_MACHINE),
    (FILTER, CLAP_PLUGIN_FEATURE_FILTER),
    (PHASER, CLAP_PLUGIN_FEATURE_PHASER),
    (EQUALIZER, CLAP_PLUGIN_FEATURE_EQUALIZER),
    (DEESSER, CLAP_PLUGIN_FEATURE_DEESSER),
    (PHASE_VOCODER, CLAP_PLUGIN_FEATURE_PHASE_VOCODER),
    (GRANULAR, CLAP_PLUGIN_FEATURE_GRANULAR),
    (FREQUENCY_SHIFTER, CLAP_PLUGIN_FEATURE_FREQUENCY_SHIFTER),
    (PITCH_SHIFTER, CLAP_PLUGIN_FEATURE_PITCH_SHIFTER),
    (DISTORTION, CLAP_PLUGIN_FEATURE_DISTORTION),
    (TRANSIENT_SHAPER, CLAP_PLUGIN_FEATURE_TRANSIENT_SHAPER),
    (COMPRESSOR, CLAP_PLUGIN_FEATURE_COMPRESSOR),
    (EXPANDER, CLAP_PLUGIN_FEATURE_EXPANDER),
    (GATE, CLAP_PLUGIN_FEATURE_GATE),
    (LIMITER, CLAP_PLUGIN_FEATURE_LIMITER),
    (FLANGER, CLAP_PLUGIN_FEATURE_FLANGER),
    (CHORUS, CLAP_PLUGIN_FEATURE_CHORUS),
    (DELAY, CLAP_PLUGIN_FEATURE_DELAY),
    (REVERB, CLAP_PLUGIN_FEATURE_REVERB),
    (TREMOLO, CLAP_PLUGIN_FEATURE_TREMOLO),
    (GLITCH, CLAP_PLUGIN_FEATURE_GLITCH),
    (UTILITY, CLAP_PLUGIN_FEATURE_UTILITY),
    (PITCH_CORRECTION, CLAP_PLUGIN_FEATURE_PITCH_CORRECTION),
    (RESTORATION, CLAP_PLUGIN_FEATURE_RESTORATION),
    (MULTI_EFFECTS, CLAP_PLUGIN_FEATURE_MULTI_EFFECTS),
    (MIXING, CLAP_PLUGIN_FEATURE_MIXING),
    (MASTERING, CLAP_PLUGIN_FEATURE_MASTERING),
    (MONO, CLAP_PLUGIN_FEATURE_MONO),
    (STEREO, CLAP_PLUGIN_FEATURE_STEREO),
    (SURROUND, CLAP_PLUGIN_FEATURE_SURROUND),
    (AMBISONIC, CLAP_PLUGIN_FEATURE_AMBISONIC),
);
