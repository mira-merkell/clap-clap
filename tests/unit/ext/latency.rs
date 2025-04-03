mod plugin_latency {
    use std::marker::PhantomData;

    use clap_clap::{
        Error,
        ext::{Extensions, latency::Latency},
        plugin::Plugin,
    };

    use crate::{
        ext::{Test, TestBed, TestConfig, TestPlugin},
        shims::plugin::ShimPlugin,
    };

    #[derive(Debug, Default)]
    struct CheckNoExt<P> {
        _marker: PhantomData<P>,
    }

    impl<P: TestPlugin + 'static> Test<P> for CheckNoExt<P> {
        fn test(self, bed: &mut TestBed<P>) {
            if P::latency().is_some() {
                assert!(bed.ext_latency.is_some());
            } else {
                assert!(bed.ext_latency.is_none());
            }
        }
    }

    #[test]
    fn no_ports_shim() {
        TestConfig::default().test::<ShimPlugin>(CheckNoExt::default());
    }

    #[derive(Default, Copy, Clone)]
    struct Plug {
        latency: u32,
    }

    impl Plugin for Plug {
        type AudioThread = ();
        const ID: &'static str = "";
        const NAME: &'static str = "";

        fn activate(&mut self, _: f64, _: u32, _: u32) -> Result<Self::AudioThread, Error> {
            Ok(())
        }
    }

    impl TestPlugin for Plug {
        fn initialize(&mut self, cfg: &TestConfig) {
            self.latency = cfg.latency;
        }
    }

    impl Extensions<Self> for Plug {
        fn latency() -> Option<impl Latency<Self>> {
            Some(PlugLatency)
        }
    }

    #[derive(Debug, Copy, Clone)]
    struct PlugLatency;

    impl Latency<Plug> for PlugLatency {
        fn get(plugin: &Plug) -> u32 {
            plugin.latency
        }
    }

    #[test]
    fn no_ports_ports() {
        TestConfig::default().test::<Plug>(CheckNoExt::default());
    }

    struct CheckLatency;

    impl Test<Plug> for CheckLatency {
        fn test(self, bed: &mut TestBed<Plug>) {
            let latency = bed.ext_latency.as_ref().unwrap().get();

            let mut handle = bed.plugin();
            assert_eq!(latency, unsafe { handle.plugin() }.latency);
        }
    }

    #[test]
    fn plugin_latency_0() {
        TestConfig::default().test::<Plug>(CheckLatency);
    }

    #[test]
    fn plugin_latency_1() {
        TestConfig {
            latency: 0,
            ..Default::default()
        }
        .test::<Plug>(CheckLatency);

        TestConfig {
            latency: 1,
            ..Default::default()
        }
        .test::<Plug>(CheckLatency);

        TestConfig {
            latency: 10,
            ..Default::default()
        }
        .test::<Plug>(CheckLatency);

        TestConfig {
            latency: 999,
            ..Default::default()
        }
        .test::<Plug>(CheckLatency);
    }
}

mod host_latency {
    use std::{error::Error, pin::Pin};

    use clap_clap::{
        host,
        host::Error::{Callback, ExtensionNotFound},
    };

    use crate::host::{ExtLatencyConfig, Test, TestBed, TestConfig};

    struct CheckLatencyNotImpl<E: Error> {
        error: E,
    }

    impl Test for CheckLatencyNotImpl<host::Error> {
        fn test(self, bed: Pin<&mut TestBed>) {
            let host = unsafe { bed.host_mut() };
            let err = host.get_extension().latency().unwrap_err();
            assert_eq!(err, self.error);
        }
    }

    #[test]
    fn latency_not_impl() {
        TestConfig::default().test(CheckLatencyNotImpl {
            error: ExtensionNotFound("latency"),
        });
    }

    #[test]
    fn latency_no_method_changed() {
        TestConfig {
            ext_latency: Some(ExtLatencyConfig {
                null_callback: true,
            }),
            ..Default::default()
        }
        .test(CheckLatencyNotImpl {
            error: Callback("changed"),
        });
    }

    struct CheckCallChanged;

    impl Test for CheckCallChanged {
        fn test(self, mut bed: Pin<&mut TestBed>) {
            let host = unsafe { bed.as_mut().host_mut() };
            let latency = host.get_extension().latency().unwrap();
            latency.changed();

            assert!(bed.ext_latency.as_ref().unwrap().call_changed);
        }
    }

    #[test]
    fn latency_call_changed() {
        TestConfig {
            ext_latency: Some(ExtLatencyConfig {
                null_callback: false,
            }),
            ..Default::default()
        }
        .test(CheckCallChanged);
    }
}
