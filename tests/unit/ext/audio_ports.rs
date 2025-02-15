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
    use clap_clap::{ext::audio_ports::RescanFlags, host::Host};

    use crate::host::TestHostConfig;

    #[test]
    fn audio_port_not_impl() {
        let test_host = TestHostConfig::default().build();
        let host = unsafe { Host::new(test_host.clap_host()) };

        let _ = host.get_extension().audio_ports().unwrap_err();
    }

    #[test]
    fn audio_port_no_method_is_rescan() {
        let mut test_host = TestHostConfig {
            impl_ext_audio_ports: true,
            ..Default::default()
        }
        .build();

        unsafe { test_host.as_mut().get_unchecked_mut() }
            .clap_host_audio_ports
            .is_rescan_flag_supported = None;

        let host = unsafe { Host::new(test_host.clap_host()) };
        let _ = host.get_extension().audio_ports().unwrap_err();
    }

    #[test]
    fn audio_port_no_method_rescan() {
        let mut test_host = TestHostConfig {
            impl_ext_audio_ports: true,
            ..Default::default()
        }
        .build();

        unsafe { test_host.as_mut().get_unchecked_mut() }
            .clap_host_audio_ports
            .rescan = None;

        let host = unsafe { Host::new(test_host.clap_host()) };
        let _ = host.get_extension().audio_ports().unwrap_err();
    }

    #[test]
    fn audio_port_impl() {
        let test_host = TestHostConfig {
            impl_ext_audio_ports: true,
            ..Default::default()
        }
        .build();
        let host = unsafe { Host::new(test_host.clap_host()) };

        let audio_ports = host.get_extension().audio_ports().unwrap();
        assert!(audio_ports.is_rescan_flag_supported(RescanFlags::ChannelCount));
        audio_ports.rescan(123);
        assert_eq!(test_host.ext_audio_port_call_rescan_flags, 123);
    }
}
