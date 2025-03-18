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
