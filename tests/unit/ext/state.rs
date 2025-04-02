mod plugin_state {
    use std::{
        io::{Read, Write},
        sync::{Arc, Mutex},
    };

    use clap_clap::{
        Error,
        ext::{Extensions, state::State},
        plugin::Plugin,
        stream::{IStream, OStream},
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
            if P::state().is_some() && self.should_implement {
                assert!(bed.ext_state.is_some());
            } else if P::state().is_none() && !self.should_implement {
                assert!(bed.ext_state.is_none());
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
        state: Arc<Mutex<[u8; 5]>>,
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
            let mut state = self.state.lock().unwrap();
            *state = cfg.state;
        }
    }

    impl Extensions<Self> for Plug {
        fn state() -> Option<impl State<Self>> {
            Some(PlugState)
        }
    }

    #[derive(Debug)]
    struct PlugState;

    impl State<Plug> for PlugState {
        fn save(plugin: &Plug, stream: &mut OStream) -> Result<(), Error> {
            let state = plugin.state.lock().unwrap();
            let n = state.len();

            let mut i = 0;
            while i < n {
                let written = stream
                    .write(&state[i..n])
                    .map_err(|_| clap_clap::ext::state::Error::Write)?;
                if written == 0 {
                    return Err(clap_clap::ext::state::Error::Eof.into());
                }

                i += written;
            }

            Ok(())
        }

        fn load(plugin: &Plug, stream: &mut IStream) -> Result<(), Error> {
            let mut state = plugin.state.lock().unwrap();
            let n = state.len();

            let mut i = 0;
            while i < n {
                let written = stream
                    .read(&mut state[i..n])
                    .map_err(|_| clap_clap::ext::state::Error::Read)?;
                if written == 0 {
                    return Err(clap_clap::ext::state::Error::Eof.into());
                }

                i += written;
            }
            Ok(())
        }
    }

    #[test]
    fn ext_impl_state() {
        TestConfig::default().test::<Plug>(CheckExtImpl {
            should_implement: true,
        });
    }

    struct CheckSaveState {
        buf: Option<Vec<u8>>,
        should_fail: bool,
        max_per_round: usize,
    }

    impl Test<Plug> for CheckSaveState {
        fn test(mut self, bed: &mut TestBed<Plug>) {
            assert_ne!(
                bed.ext_state
                    .as_mut()
                    .unwrap()
                    .save(self.buf.as_mut(), self.max_per_round),
                self.should_fail
            );

            if !self.should_fail {
                let mut wrapper = bed.plugin();
                let plugin = unsafe { wrapper.plugin() };
                let state = plugin.state.lock().unwrap();

                let Some(buf) = &self.buf else {
                    panic!("no buffer to store the state")
                };
                assert_eq!(buf[0..state.len()], state[..]);
            }
        }
    }

    #[test]
    fn save_state_00() {
        TestConfig {
            state: [0, 1, 2, 3, 4],
            ..Default::default()
        }
        .test(CheckSaveState {
            buf: None,
            should_fail: true,
            max_per_round: 1,
        })
        .test(CheckSaveState {
            buf: None,
            should_fail: true,
            max_per_round: 2,
        })
        .test(CheckSaveState {
            buf: None,
            should_fail: true,
            max_per_round: 99,
        });
    }

    #[test]
    fn save_state_01() {
        TestConfig {
            state: [0, 1, 2, 3, 4],
            ..Default::default()
        }
        .test(CheckSaveState {
            buf: Some(vec![]),
            should_fail: true,
            max_per_round: 1,
        })
        .test(CheckSaveState {
            buf: Some(vec![]),
            should_fail: true,
            max_per_round: 2,
        })
        .test(CheckSaveState {
            buf: Some(vec![]),
            should_fail: true,
            max_per_round: 99,
        });
    }

    #[test]
    fn save_state_02() {
        TestConfig {
            state: [0, 1, 2, 3, 4],
            ..Default::default()
        }
        .test(CheckSaveState {
            buf: Some(vec![0; 4]),
            should_fail: true,
            max_per_round: 1,
        })
        .test(CheckSaveState {
            buf: Some(vec![0; 4]),
            should_fail: true,
            max_per_round: 3,
        })
        .test(CheckSaveState {
            buf: Some(vec![0; 4]),
            should_fail: true,
            max_per_round: 99,
        });
    }

    #[test]
    fn save_state_03() {
        TestConfig {
            state: [0, 1, 2, 3, 4],
            ..Default::default()
        }
        .test(CheckSaveState {
            buf: Some(vec![0; 5]),
            should_fail: false,
            max_per_round: 1,
        })
        .test(CheckSaveState {
            buf: Some(vec![0; 5]),
            should_fail: false,
            max_per_round: 2,
        })
        .test(CheckSaveState {
            buf: Some(vec![0; 5]),
            should_fail: false,
            max_per_round: 3,
        })
        .test(CheckSaveState {
            buf: Some(vec![0; 5]),
            should_fail: false,
            max_per_round: 4,
        })
        .test(CheckSaveState {
            buf: Some(vec![0; 5]),
            should_fail: false,
            max_per_round: 5,
        })
        .test(CheckSaveState {
            buf: Some(vec![0; 5]),
            should_fail: false,
            max_per_round: 6,
        });
    }

    #[test]
    fn save_state_04() {
        TestConfig {
            state: [0, 1, 2, 3, 4],
            ..Default::default()
        }
        .test(CheckSaveState {
            buf: Some(vec![0; 6]),
            should_fail: false,
            max_per_round: 1,
        })
        .test(CheckSaveState {
            buf: Some(vec![0; 6]),
            should_fail: false,
            max_per_round: 2,
        })
        .test(CheckSaveState {
            buf: Some(vec![0; 6]),
            should_fail: false,
            max_per_round: 3,
        })
        .test(CheckSaveState {
            buf: Some(vec![0; 6]),
            should_fail: false,
            max_per_round: 4,
        })
        .test(CheckSaveState {
            buf: Some(vec![0; 6]),
            should_fail: false,
            max_per_round: 5,
        })
        .test(CheckSaveState {
            buf: Some(vec![0; 6]),
            should_fail: false,
            max_per_round: 99,
        });
    }

    struct CheckLoadState {
        buf: Option<Vec<u8>>,
        should_fail: bool,
        max_per_round: usize,
    }

    impl Test<Plug> for CheckLoadState {
        fn test(mut self, bed: &mut TestBed<Plug>) {
            assert_ne!(
                bed.ext_state
                    .as_mut()
                    .unwrap()
                    .load(self.buf.as_mut(), self.max_per_round),
                self.should_fail
            );

            if !self.should_fail {
                let mut wrapper = bed.plugin();
                let plugin = unsafe { wrapper.plugin() };
                let state = plugin.state.lock().unwrap();

                let Some(buf) = &self.buf else {
                    panic!("no buffer to store the state")
                };
                assert_eq!(buf[0..state.len()], state[..]);
            }
        }
    }

    #[test]
    fn load_state_00() {
        TestConfig {
            state: [0; 5],
            ..Default::default()
        }
        .test(CheckLoadState {
            buf: None,
            should_fail: true,
            max_per_round: 1,
        })
        .test(CheckLoadState {
            buf: None,
            should_fail: true,
            max_per_round: 2,
        })
        .test(CheckLoadState {
            buf: None,
            should_fail: true,
            max_per_round: 99,
        });
    }

    #[test]
    fn load_state_01() {
        TestConfig {
            state: [0; 5],
            ..Default::default()
        }
        .test(CheckLoadState {
            buf: Some(vec![]),
            should_fail: true,
            max_per_round: 1,
        })
        .test(CheckLoadState {
            buf: Some(vec![]),
            should_fail: true,
            max_per_round: 2,
        })
        .test(CheckLoadState {
            buf: Some(vec![]),
            should_fail: true,
            max_per_round: 99,
        });
    }

    #[test]
    fn load_state_02() {
        TestConfig {
            state: [0; 5],
            ..Default::default()
        }
        .test(CheckLoadState {
            buf: Some(vec![0; 4]),
            should_fail: true,
            max_per_round: 1,
        })
        .test(CheckLoadState {
            buf: Some(vec![0; 4]),
            should_fail: true,
            max_per_round: 3,
        })
        .test(CheckLoadState {
            buf: Some(vec![0; 4]),
            should_fail: true,
            max_per_round: 99,
        });
    }

    #[test]
    fn load_state_03() {
        TestConfig {
            state: [0; 5],
            ..Default::default()
        }
        .test(CheckLoadState {
            buf: Some(vec![0, 1, 2, 3, 4]),
            should_fail: false,
            max_per_round: 1,
        })
        .test(CheckLoadState {
            buf: Some(vec![0, 1, 2, 3, 4]),
            should_fail: false,
            max_per_round: 2,
        })
        .test(CheckLoadState {
            buf: Some(vec![0, 1, 2, 3, 4]),
            should_fail: false,
            max_per_round: 3,
        })
        .test(CheckLoadState {
            buf: Some(vec![0, 1, 2, 3, 4]),
            should_fail: false,
            max_per_round: 4,
        })
        .test(CheckLoadState {
            buf: Some(vec![0, 1, 2, 3, 4]),
            should_fail: false,
            max_per_round: 5,
        })
        .test(CheckLoadState {
            buf: Some(vec![0, 1, 2, 3, 4]),
            should_fail: false,
            max_per_round: 99,
        });
    }

    #[test]
    fn load_state_04() {
        TestConfig {
            state: [0; 5],
            ..Default::default()
        }
        .test(CheckLoadState {
            buf: Some(vec![0, 1, 2, 3, 4, 5]),
            should_fail: false,
            max_per_round: 1,
        })
        .test(CheckLoadState {
            buf: Some(vec![0, 1, 2, 3, 4, 5]),
            should_fail: false,
            max_per_round: 3,
        })
        .test(CheckLoadState {
            buf: Some(vec![0, 1, 2, 3, 4, 5]),
            should_fail: false,
            max_per_round: 5,
        })
        .test(CheckLoadState {
            buf: Some(vec![0, 1, 2, 3, 4, 5]),
            should_fail: false,
            max_per_round: 99,
        });
    }
}
