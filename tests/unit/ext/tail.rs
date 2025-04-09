mod plugin_tail {
    use clap_clap::{
        Error,
        ext::{Extensions, tail::Tail},
        plugin::Plugin,
    };

    use crate::{
        ext::{Test, TestBed, TestConfig, TestPlugin},
        shims::plugin::ShimPlugin,
    };

    #[derive(Debug, Default)]
    struct CheckExtImpl {
        should_implement: bool,
    }

    impl<P: TestPlugin + 'static> Test<P> for CheckExtImpl {
        fn test(self, bed: &mut TestBed<P>) {
            if P::tail().is_some() && self.should_implement {
                assert!(bed.ext_tail.is_some());
            } else if P::tail().is_none() && !self.should_implement {
                assert!(bed.ext_tail.is_none());
            } else {
                panic!("wrong implementation")
            }
        }
    }

    #[test]
    fn ext_impl_shim() {
        TestConfig::default().test::<ShimPlugin>(CheckExtImpl {
            should_implement: false,
        });
    }

    #[derive(Default, Clone)]
    struct Plug {
        tail: u32,
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
        fn initialize(&mut self, config: &TestConfig) {
            self.tail = config.tail
        }
    }

    impl Extensions<Plug> for Plug {
        fn tail() -> Option<impl Tail<Plug>> {
            Some(PlugTail)
        }
    }

    struct PlugTail;

    impl Tail<Plug> for PlugTail {
        fn get(plugin: &Plug) -> u32 {
            plugin.tail
        }
    }

    #[test]
    fn ext_impl_tail() {
        TestConfig::default().test::<Plug>(CheckExtImpl {
            should_implement: true,
        });
    }

    struct CheckTail;

    impl Test<Plug> for CheckTail {
        fn test(self, bed: &mut TestBed<Plug>) {
            let tail = bed.ext_tail.as_mut().unwrap().get();

            let mut plugin = bed.plugin();
            let exp_tail = unsafe { plugin.plugin().tail };

            assert_eq!(tail, exp_tail);
        }
    }

    #[test]
    fn get_tail_01() {
        TestConfig {
            tail: 0,
            ..Default::default()
        }
        .test(CheckTail);
    }

    #[test]
    fn get_tail_02() {
        TestConfig {
            tail: 17,
            ..Default::default()
        }
        .test(CheckTail);
    }

    #[test]
    fn get_tail_03() {
        TestConfig {
            tail: i32::MAX as u32 + 1,
            ..Default::default()
        }
        .test(CheckTail);
    }
}

mod host_tail {
    use std::{error::Error, pin::Pin};

    use clap_clap::{
        host,
        host::Error::{Callback, ExtensionNotFound},
    };

    use crate::host::{ExtTailConfig, Test, TestBed, TestConfig};

    struct CheckTailNotImpl<E: Error> {
        error: E,
    }

    impl Test for CheckTailNotImpl<host::Error> {
        fn test(self, bed: Pin<&mut TestBed>) {
            let host = unsafe { bed.host_mut() };
            let err = host.get_extension().tail().unwrap_err();
            assert_eq!(err, self.error);
        }
    }

    #[test]
    fn tail_not_impl() {
        TestConfig::default().test(CheckTailNotImpl {
            error: ExtensionNotFound("tail"),
        });
    }

    #[test]
    fn tail_no_method_changed() {
        TestConfig {
            ext_tail: Some(ExtTailConfig {
                null_callback: true,
            }),
            ..Default::default()
        }
        .test(CheckTailNotImpl {
            error: Callback("changed"),
        });
    }

    struct CheckCallChanged;

    impl Test for CheckCallChanged {
        fn test(self, mut bed: Pin<&mut TestBed>) {
            let host = unsafe { bed.as_mut().host_mut() };
            let latency = host.get_extension().tail().unwrap();
            latency.changed();

            assert!(bed.ext_tail.as_ref().unwrap().call_changed);
        }
    }

    #[test]
    fn tail_call_make_dirty() {
        TestConfig {
            ext_tail: Some(ExtTailConfig {
                null_callback: false,
            }),
            ..Default::default()
        }
        .test(CheckCallChanged);
    }
}
