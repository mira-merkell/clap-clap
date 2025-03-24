mod plugin_latency {
    use std::marker::PhantomData;

    use clap_clap::{
        Error,
        ext::{Extensions, latency::Latency},
        plugin::Plugin,
    };

    use crate::{
        ext::{Test, TestBed, TestConfig},
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
        TestConfig::<ShimPlugin>::default().test(CheckNoExt::default());
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
        TestConfig::<TestPlug>::default().test(CheckNoExt::default());
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
        TestConfig::<TestPlug>::default().test(CheckLatency);
    }

    #[test]
    fn plugin_latency_1() {
        TestConfig::<TestPlug>::with_op(|p| p.latency = 0).test(CheckLatency);
        TestConfig::<TestPlug>::with_op(|p| p.latency = 1).test(CheckLatency);
        TestConfig::<TestPlug>::with_op(|p| p.latency = 10).test(CheckLatency);
        TestConfig::<TestPlug>::with_op(|p| p.latency = 999).test(CheckLatency);
    }
}
