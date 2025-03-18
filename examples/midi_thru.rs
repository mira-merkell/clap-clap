use std::sync::Arc;

use clap_clap::prelude as clap;

#[derive(Default)]
struct MidiThru;

impl clap::Extensions<Self> for MidiThru {
    fn note_ports() -> Option<impl clap::NotePorts<Self>> {
        Some(Self::default())
    }
}

impl clap::NotePorts<Self> for MidiThru {
    fn count(_: &Self, _: bool) -> u32 {
        1
    }

    fn get(_: &Self, index: u32, is_input: bool) -> Option<clap::NotePortInfo> {
        if index == 0 && is_input {
            Some(clap::NotePortInfo {
                id: clap::ClapId::from(0),
                supported_dialects: clap::NoteDialect::Midi as u32,
                preferred_dialect: clap::NoteDialect::Midi as u32,
                name: "In 1".to_string(),
            })
        } else if index == 0 {
            Some(clap::NotePortInfo {
                id: clap::ClapId::from(0),
                supported_dialects: clap::NoteDialect::Midi as u32,
                preferred_dialect: clap::NoteDialect::Midi as u32,
                name: "Out 1".to_string(),
            })
        } else {
            None
        }
    }
}

impl clap::Plugin for MidiThru {
    type AudioThread = Self;

    const ID: &'static str = "com.your-company.YourPlugin";
    const NAME: &'static str = "Plugin Name";
    const VENDOR: &'static str = "Vendor";
    const URL: &'static str = "https://your-domain.com/your-plugin";
    const MANUAL_URL: &'static str = "https://your-domain.com/your-plugin/manual";
    const SUPPORT_URL: &'static str = "https://your-domain.com/support";
    const VERSION: &'static str = "1.4.2";
    const DESCRIPTION: &'static str = "The plugin description.";

    fn features() -> impl Iterator<Item = &'static str> {
        "fx midi instrument".split_whitespace()
    }

    fn init(&mut self, _: Arc<clap::Host>) -> Result<(), clap::Error> {
        Ok(())
    }

    /// Start the audio thread.
    fn activate(&mut self, _: f64, _: u32, _: u32) -> Result<Self, clap::Error> {
        Ok(Self {})
    }
}

impl clap::AudioThread<Self> for MidiThru {
    fn process(&mut self, process: &mut clap::Process) -> Result<clap::Status, clap::Error> {
        let nframes = process.frames_count();
        let nev = process.in_events().size();
        let mut ev_index = 0;
        let mut next_ev_frame = if nev > 0 { 0 } else { nframes };

        let mut i = 0;
        while i < nframes {
            while ev_index < nev && next_ev_frame == i {
                {
                    let in_events = process.in_events();
                    let header = in_events.get(ev_index);
                    if header.time() != i {
                        next_ev_frame = header.time();
                        break;
                    }

                    if let Ok(midi) = header.midi() {
                        let _ = process.out_events().try_push(midi);
                    }
                }

                ev_index += 1;

                if ev_index == nev {
                    next_ev_frame = nframes;
                    break;
                }
            }

            i += 1;
        }
        Ok(clap::Status::Continue)
    }
}

// Export clap_entry symbols and build a plugin factory.
clap::entry!(MidiThru);
