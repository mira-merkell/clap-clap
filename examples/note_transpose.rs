//! This example shows how to implement the NotePorts plugin extension.
//!
//! The plugin transposes incoming CLAP notes by a perfect fifth and passes all
//! MIDI or MIDI2 data unchanged.

use std::sync::Arc;

use clap_clap::prelude as clap;

#[derive(Default)]
struct Transpose;

impl clap::Extensions<Self> for Transpose {
    fn note_ports() -> Option<impl clap::NotePorts<Self>> {
        Some(Self {})
    }
}

impl clap::NotePorts<Self> for Transpose {
    fn count(_: &Self, _is_input: bool) -> u32 {
        2 // two in, two out
    }

    fn get(_: &Self, index: u32, is_input: bool) -> Option<clap::NotePortInfo> {
        if index < 2 {
            Some(clap::NotePortInfo {
                id: if is_input {
                    clap::ClapId::from(index as u16)
                } else {
                    clap::ClapId::from(index as u16 + 2)
                },
                supported_dialects: clap::NoteDialect::all(),
                preferred_dialect: clap::NoteDialect::Clap as u32,
                name: if is_input {
                    format!("In {index}")
                } else {
                    format!("Out {index}")
                },
            })
        } else {
            None
        }
    }
}

impl clap::Plugin for Transpose {
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
        "fx transpose midi thru".split_whitespace()
    }

    fn init(&mut self, _: Arc<clap::Host>) -> Result<(), clap::Error> {
        Ok(())
    }

    /// Start the audio thread.
    fn activate(&mut self, _: f64, _: u32, _: u32) -> Result<Self, clap::Error> {
        Ok(Self {})
    }
}

impl clap::AudioThread<Self> for Transpose {
    fn process(&mut self, process: &mut clap::Process) -> Result<clap::Status, clap::Error> {
        let in_events = process.in_events();
        let mut out_events = process.out_events();

        for i in 0..in_events.size() {
            let header = in_events.get(i);

            if let Ok(note) = header.note() {
                use clap::EventBuilder;
                let n = note.update().key(note.key() + 7); // Transpose notes by a perfect fifth.
                let _ = out_events.try_push(n.event());
            }

            if let Ok(note_expr) = header.note_expression() {
                let _ = out_events.try_push(note_expr);
            }

            if let Ok(midi) = header.midi() {
                let _ = out_events.try_push(midi);
            }

            if let Ok(midi2) = header.midi2() {
                let _ = out_events.try_push(midi2);
            }
        }

        Ok(clap::Continue)
    }
}

// Export clap_entry symbols and build a plugin factory.
clap::entry!(Transpose);
