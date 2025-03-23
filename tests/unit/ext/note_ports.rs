mod plugin_note_ports {
    use std::marker::PhantomData;

    use clap_clap::{
        Error,
        ext::{
            Extensions,
            note_ports::{NoteDialect, NotePortInfo, NotePorts},
        },
        id::ClapId,
        plugin::Plugin,
    };

    use crate::{
        ext::{Test, TestBed, TestConfig},
        shims::plugin::ShimPlugin,
    };

    #[derive(Debug, Default)]
    struct CheckNoPorts<P> {
        _marker: PhantomData<P>,
    }

    impl<P: Plugin + 'static> Test<P> for CheckNoPorts<P> {
        fn test(self, bed: &mut TestBed<P>) {
            if P::note_ports().is_some() {
                assert!(bed.ext_note_ports.is_some());
            } else {
                assert!(bed.ext_note_ports.is_none());
            }
        }
    }

    #[test]
    fn no_ports_shim() {
        TestConfig::<ShimPlugin>::default().test(CheckNoPorts::default());
    }

    #[derive(Default, Copy, Clone)]
    struct Ports;

    impl Plugin for Ports {
        type AudioThread = ();
        const ID: &'static str = "";
        const NAME: &'static str = "";

        fn activate(&mut self, _: f64, _: u32, _: u32) -> Result<Self::AudioThread, Error> {
            Ok(())
        }
    }

    impl Extensions<Self> for Ports {
        fn note_ports() -> Option<impl NotePorts<Self>> {
            Some(Self {})
        }
    }

    impl NotePorts<Self> for Ports {
        fn count(_: &Self, _: bool) -> u32 {
            1
        }

        fn get(_: &Self, index: u32, is_input: bool) -> Option<NotePortInfo> {
            if is_input && index == 0 {
                Some(NotePortInfo {
                    name: "input 1".to_owned(),
                    id: ClapId::from(0),
                    supported_dialects: NoteDialect::Clap as u32 | NoteDialect::Midi2 as u32,
                    preferred_dialect: NoteDialect::Clap as u32,
                })
            } else if !is_input && index == 0 {
                Some(NotePortInfo {
                    name: "output 1".to_owned(),
                    id: ClapId::from(1),
                    supported_dialects: NoteDialect::Midi as u32 | NoteDialect::Midi2 as u32,
                    preferred_dialect: NoteDialect::MidiMPE as u32,
                })
            } else {
                None
            }
        }
    }

    #[test]
    fn no_ports_ports() {
        TestConfig::<Ports>::default().test(CheckNoPorts::default());
    }

    #[test]
    fn ports_input_output_count() {
        let bed = &mut TestBed::new(TestConfig::<Ports>::default());

        let note_ports = bed.ext_note_ports.as_ref().unwrap();

        assert_eq!(note_ports.count(false), 1);
        assert_eq!(note_ports.count(true), 1);
        assert_eq!(note_ports.get(1, false), None);
        assert_eq!(note_ports.get(1, true), None);
    }

    #[test]
    fn ports_input_info() {
        let bed = &mut TestBed::new(TestConfig::<Ports>::default());

        let note_ports = bed.ext_note_ports.as_ref().unwrap();

        let plug = Ports {};
        let port_info = Ports::get(&plug, 0, true);
        assert_eq!(note_ports.get(0, true), port_info);
    }

    #[test]
    fn ports_output_info() {
        let bed = &mut TestBed::new(TestConfig::<Ports>::default());

        let note_ports = bed.ext_note_ports.as_ref().unwrap();

        let plug = Ports {};
        let port_info = Ports::get(&plug, 0, false);
        assert_eq!(note_ports.get(0, false), port_info);
    }
}

mod host_note_ports {
    use std::{error::Error, pin::Pin};

    use clap_clap::{
        ext::note_ports::NoteDialect,
        host,
        host::Error::{Callback, ExtensionNotFound},
    };

    use crate::host::{ExtNotePortsConfig, Test, TestBed, TestConfig};

    struct CheckNotePortNotImpl<E: Error> {
        error: E,
    }

    impl Test for CheckNotePortNotImpl<host::Error> {
        fn test(self, bed: Pin<&mut TestBed>) {
            let host = unsafe { bed.host_mut() };
            let err = host.get_extension().note_ports().unwrap_err();
            assert_eq!(err, self.error);
        }
    }

    #[test]
    fn note_port_not_impl() {
        TestConfig::default().test(CheckNotePortNotImpl {
            error: ExtensionNotFound("note_ports"),
        });
    }

    #[test]
    fn note_port_no_method_supported_dialects() {
        TestConfig {
            ext_note_ports: Some(ExtNotePortsConfig {
                null_supported_dialects: true,
                ..Default::default()
            }),
            ..Default::default()
        }
        .test(CheckNotePortNotImpl {
            error: Callback("supported_dialects"),
        });
    }

    #[test]
    fn note_port_no_method_rescan() {
        TestConfig {
            ext_note_ports: Some(ExtNotePortsConfig {
                null_rescan: true,
                ..Default::default()
            }),
            ..Default::default()
        }
        .test(CheckNotePortNotImpl {
            error: Callback("rescan"),
        });
    }

    struct CheckSupportedDialect {
        supported: NoteDialect,
        not_supported: Option<NoteDialect>,
    }

    impl Test for CheckSupportedDialect {
        fn test(self, mut bed: Pin<&mut TestBed>) {
            let host = unsafe { bed.as_mut().host_mut() };
            let note_ports = host.get_extension().note_ports().unwrap();

            assert!(self.supported.is_set(note_ports.supported_dialects()));

            if let Some(flag) = self.not_supported {
                assert!(!flag.is_set(note_ports.supported_dialects()));
            }
        }
    }

    #[test]
    fn note_port_supported_flag_01() {
        TestConfig {
            ext_note_ports: Some(ExtNotePortsConfig {
                supported_dialects: !0, // all flags supported
                ..Default::default()
            }),
            ..Default::default()
        }
        .test(CheckSupportedDialect {
            supported: NoteDialect::Clap,
            not_supported: None,
        })
        .test(CheckSupportedDialect {
            supported: NoteDialect::MidiMPE,
            not_supported: None,
        });
    }

    #[test]
    fn note_port_supported_flag_02() {
        TestConfig {
            ext_note_ports: Some(ExtNotePortsConfig {
                supported_dialects: !(NoteDialect::Clap as u32),
                ..Default::default()
            }),
            ..Default::default()
        }
        .test(CheckSupportedDialect {
            supported: NoteDialect::MidiMPE,
            not_supported: Some(NoteDialect::Clap),
        })
        .test(CheckSupportedDialect {
            supported: NoteDialect::Midi2,
            not_supported: Some(NoteDialect::Clap),
        });
    }

    #[test]
    fn note_port_supported_flag_03() {
        TestConfig {
            ext_note_ports: Some(ExtNotePortsConfig {
                supported_dialects: NoteDialect::Clap as u32,
                ..Default::default()
            }),
            ..Default::default()
        }
        .test(CheckSupportedDialect {
            supported: NoteDialect::Clap,
            not_supported: Some(NoteDialect::Midi),
        })
        .test(CheckSupportedDialect {
            supported: NoteDialect::Clap,
            not_supported: Some(NoteDialect::Midi2),
        });
    }

    struct CheckRescanFlags {
        flags: u32,
    }

    impl Test for CheckRescanFlags {
        fn test(self, mut bed: Pin<&mut TestBed>) {
            let host = unsafe { bed.as_mut().host_mut() };
            let note_ports = host.get_extension().note_ports().unwrap();

            note_ports.rescan(self.flags);

            assert_eq!(
                bed.ext_note_ports.as_ref().unwrap().call_rescan_flags,
                self.flags
            );
        }
    }

    #[test]
    fn note_port_impl_flag_channel_count() {
        TestConfig {
            ext_note_ports: Some(ExtNotePortsConfig::default()),
            ..Default::default()
        }
        .test(CheckRescanFlags { flags: 0 })
        .test(CheckRescanFlags { flags: 1 })
        .test(CheckRescanFlags { flags: 127 })
        .test(CheckRescanFlags { flags: 128 });
    }
}
