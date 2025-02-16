mod static_ports {
    use clap_clap::ext::audio_ports::{
        AudioPortFlags, AudioPortType, AudioPorts, MonoPorts, StereoPorts,
    };

    use crate::shims::plugin::ShimPlugin;

    macro_rules! check_mono_ports {
        ($name:tt, $In:literal, $Out: literal) => {
            #[test]
            fn $name() {
                let plug = ShimPlugin {};

                assert_eq!(MonoPorts::<$In, $Out>::count(&plug, true), $In);
                assert_eq!(MonoPorts::<$In, $Out>::count(&plug, false), $Out);

                let main_in = MonoPorts::<$In, $Out>::get(&plug, 0, true).unwrap();
                assert!(AudioPortFlags::IsMain.is_set(main_in.flags));

                let main_out = MonoPorts::<$In, $Out>::get(&plug, 0, false).unwrap();
                assert!(AudioPortFlags::IsMain.is_set(main_out.flags));

                for n in 0..$In {
                    let port = MonoPorts::<$In, $Out>::get(&plug, n, true).unwrap();
                    assert_eq!(port.port_type.unwrap(), AudioPortType::Mono);
                }
                assert!(MonoPorts::<$In, $Out>::get(&plug, $In, true).is_none());

                for n in 0..$Out {
                    let port = MonoPorts::<$In, $Out>::get(&plug, n, false).unwrap();
                    assert_eq!(port.port_type.unwrap(), AudioPortType::Mono);
                }
                assert!(MonoPorts::<$In, $Out>::get(&plug, $Out, false).is_none());
            }
        };
    }

    check_mono_ports!(mono_1_1, 1, 1);
    check_mono_ports!(mono_1_2, 1, 2);
    check_mono_ports!(mono_2_1, 2, 1);
    check_mono_ports!(mono_2_2, 2, 2);
    check_mono_ports!(mono_13_17, 13, 17);

    macro_rules! check_stereo_ports {
        ($name:tt, $In:literal, $Out: literal) => {
            #[test]
            fn $name() {
                let plug = ShimPlugin {};

                assert_eq!(StereoPorts::<$In, $Out>::count(&plug, true), $In);
                assert_eq!(StereoPorts::<$In, $Out>::count(&plug, false), $Out);

                let main_in = StereoPorts::<$In, $Out>::get(&plug, 0, true).unwrap();
                assert!(AudioPortFlags::IsMain.is_set(main_in.flags));

                let main_out = StereoPorts::<$In, $Out>::get(&plug, 0, false).unwrap();
                assert!(AudioPortFlags::IsMain.is_set(main_out.flags));

                for n in 0..$In {
                    let port = StereoPorts::<$In, $Out>::get(&plug, n, true).unwrap();
                    assert_eq!(port.port_type.unwrap(), AudioPortType::Stereo);
                }
                assert!(StereoPorts::<$In, $Out>::get(&plug, $In, true).is_none());

                for n in 0..$Out {
                    let port = StereoPorts::<$In, $Out>::get(&plug, n, false).unwrap();
                    assert_eq!(port.port_type.unwrap(), AudioPortType::Stereo);
                }
                assert!(StereoPorts::<$In, $Out>::get(&plug, $Out, false).is_none());
            }
        };
    }

    check_stereo_ports!(stereo_1_1, 1, 1);
    check_stereo_ports!(stereo_1_2, 1, 2);
    check_stereo_ports!(stereo_2_1, 2, 1);
    check_stereo_ports!(stereo_2_2, 2, 2);
    check_stereo_ports!(stereo_13_17, 13, 17);
}

mod host_audio_ports {
    use std::{error::Error, pin::Pin};

    use clap_clap::{
        ext::audio_ports::RescanFlags,
        host,
        host::Error::{Callback, ExtensionNotFound},
    };

    use crate::host::{ExtAudioPortsConfig, Test, TestBed, TestConfig};

    struct CheckAudioPortNotImpl<E: Error> {
        error: E,
    }

    impl Test for CheckAudioPortNotImpl<host::Error> {
        fn test(self, bed: Pin<&mut TestBed>) {
            let host = unsafe { bed.host_mut() };
            let err = host.get_extension().audio_ports().unwrap_err();
            assert_eq!(err, self.error);
        }
    }

    #[test]
    fn audio_port_not_impl() {
        TestConfig::default().test(CheckAudioPortNotImpl {
            error: ExtensionNotFound("audio_ports"),
        });
    }

    #[test]
    fn audio_port_no_method_is_rescan() {
        TestConfig {
            ext_audio_ports: Some(ExtAudioPortsConfig {
                null_is_rescan_flag_supported: true,
                ..Default::default()
            }),
            ..Default::default()
        }
        .test(CheckAudioPortNotImpl {
            error: Callback("is_rescan_flag_supported"),
        });
    }

    #[test]
    fn audio_port_no_method_rescan() {
        TestConfig {
            ext_audio_ports: Some(ExtAudioPortsConfig {
                null_rescan: true,
                ..Default::default()
            }),
            ..Default::default()
        }
        .test(CheckAudioPortNotImpl {
            error: Callback("rescan"),
        });
    }

    struct CheckSupportedFlag {
        supported: RescanFlags,
        not_supported: Option<RescanFlags>,
    }

    impl Test for CheckSupportedFlag {
        fn test(self, mut bed: Pin<&mut TestBed>) {
            let host = unsafe { bed.as_mut().host_mut() };
            let audio_ports = host.get_extension().audio_ports().unwrap();

            assert!(audio_ports.is_rescan_flag_supported(self.supported));

            if let Some(flag) = self.not_supported {
                assert!(!audio_ports.is_rescan_flag_supported(flag));
            }
        }
    }

    #[test]
    fn audio_port_supported_flag_01() {
        TestConfig {
            ext_audio_ports: Some(ExtAudioPortsConfig {
                supported_flags: !0, // all flags supported
                ..Default::default()
            }),
            ..Default::default()
        }
        .test(CheckSupportedFlag {
            supported: RescanFlags::ChannelCount,
            not_supported: None,
        })
        .test(CheckSupportedFlag {
            supported: RescanFlags::PortType,
            not_supported: None,
        });
    }

    #[test]
    fn audio_port_supported_flag_02() {
        TestConfig {
            ext_audio_ports: Some(ExtAudioPortsConfig {
                supported_flags: !(RescanFlags::Names as u32),
                ..Default::default()
            }),
            ..Default::default()
        }
        .test(CheckSupportedFlag {
            supported: RescanFlags::ChannelCount,
            not_supported: Some(RescanFlags::Names),
        })
        .test(CheckSupportedFlag {
            supported: RescanFlags::PortType,
            not_supported: Some(RescanFlags::Names),
        });
    }

    struct CheckRescanFlags {
        flags: u32,
    }

    impl Test for CheckRescanFlags {
        fn test(self, mut bed: Pin<&mut TestBed>) {
            let host = unsafe { bed.as_mut().host_mut() };
            let audio_ports = host.get_extension().audio_ports().unwrap();

            audio_ports.rescan(self.flags);

            assert_eq!(
                bed.ext_audio_ports.as_ref().unwrap().call_rescan_flags,
                self.flags
            );
        }
    }

    #[test]
    fn audio_port_impl_flag_channel_count() {
        TestConfig {
            ext_audio_ports: Some(ExtAudioPortsConfig {
                supported_flags: RescanFlags::ChannelCount as u32,
                ..Default::default()
            }),
            ..Default::default()
        }
        .test(CheckRescanFlags { flags: 0 })
        .test(CheckRescanFlags { flags: 127 })
        .test(CheckRescanFlags { flags: 128 });
    }
}
