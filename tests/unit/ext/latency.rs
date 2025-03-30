mod plugin_latency {
    use std::marker::PhantomData;

    use clap_clap::{
        Error,
        ext::{Extensions, latency::Latency},
        plugin::Plugin,
    };

    use crate::{
        ext::{Test, TestBed},
        shims::plugin::ShimPlugin,
    };

    #[derive(Debug, Default)]
    struct CheckNoExt<P> {
        _marker: PhantomData<P>,
    }

    impl<P: Plugin + 'static> Test<P> for CheckNoExt<P> {
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
        TestBed::<ShimPlugin>::default().test(CheckNoExt::default());
    }

    #[derive(Default, Copy, Clone)]
    struct TestPlug {
        latency: u32,
    }

    impl Plugin for TestPlug {
        type AudioThread = ();
        const ID: &'static str = "";
        const NAME: &'static str = "";

        fn activate(&mut self, _: f64, _: u32, _: u32) -> Result<Self::AudioThread, Error> {
            Ok(())
        }
    }

    impl Extensions<Self> for TestPlug {
        fn latency() -> Option<impl Latency<Self>> {
            Some(TestLatency)
        }
    }

    #[derive(Debug, Copy, Clone)]
    struct TestLatency;

    impl Latency<TestPlug> for TestLatency {
        fn get(plugin: &TestPlug) -> u32 {
            plugin.latency
        }
    }

    #[test]
    fn no_ports_ports() {
        TestBed::<TestPlug>::default().test(CheckNoExt::default());
    }

    struct CheckLatency;

    impl Test<TestPlug> for CheckLatency {
        fn test(self, bed: &mut TestBed<TestPlug>) {
            let latency = bed.ext_latency.as_ref().unwrap().get();

            let mut handle = bed.plugin();
            assert_eq!(latency, unsafe { handle.plugin() }.latency);
        }
    }

    #[test]
    fn plugin_latency_0() {
        TestBed::<TestPlug>::default().test(CheckLatency);
    }

    #[test]
    fn plugin_latency_1() {
        TestBed::<TestPlug>::with_op(|p| p.latency = 0).test(CheckLatency);
        TestBed::<TestPlug>::with_op(|p| p.latency = 1).test(CheckLatency);
        TestBed::<TestPlug>::with_op(|p| p.latency = 10).test(CheckLatency);
        TestBed::<TestPlug>::with_op(|p| p.latency = 999).test(CheckLatency);
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
