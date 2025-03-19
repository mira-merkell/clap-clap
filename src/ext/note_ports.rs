use std::fmt::{Display, Formatter};

use crate::{
    ffi::{
        CLAP_NOTE_DIALECT_CLAP, CLAP_NOTE_DIALECT_MIDI, CLAP_NOTE_DIALECT_MIDI_MPE,
        CLAP_NOTE_DIALECT_MIDI2, clap_note_port_info,
    },
    id::ClapId,
    impl_flags_u32,
    plugin::Plugin,
};

pub trait NotePorts<P>
where
    P: Plugin,
{
    fn count(plugin: &P, is_input: bool) -> u32;
    fn get(plugin: &P, index: u32, is_input: bool) -> Option<NotePortInfo>;
}

impl<P: Plugin> NotePorts<P> for () {
    fn count(_: &P, _: bool) -> u32 {
        0
    }

    fn get(_: &P, _: u32, _: bool) -> Option<NotePortInfo> {
        None
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u32)]
pub enum NoteDialect {
    /// Uses clap_event_note and clap_event_note_expression.
    Clap = CLAP_NOTE_DIALECT_CLAP,
    /// Uses clap_event_midi, no polyphonic expression
    Midi = CLAP_NOTE_DIALECT_MIDI,
    /// Uses clap_event_midi, with polyphonic expression (MPE)
    MidiMPE = CLAP_NOTE_DIALECT_MIDI_MPE,
    /// Uses clap_event_midi2
    Midi2 = CLAP_NOTE_DIALECT_MIDI2,
}

impl_flags_u32!(NoteDialect);

#[derive(Debug, Default, Clone, PartialEq)]
pub struct NotePortInfo {
    pub id: ClapId,
    pub supported_dialects: u32,
    pub preferred_dialect: u32,
    pub name: String,
}

impl NotePortInfo {
    pub(super) fn fill_clap_note_port_info(&self, info: &mut clap_note_port_info) {
        info.id = self.id.into();

        // info.name.len > 1 so no underflow
        let n = self.name.len().min(info.name.len() - 1);
        unsafe {
            std::ptr::copy_nonoverlapping(self.name.as_ptr(), info.name.as_mut_ptr() as *mut _, n)
        }
        // n is within bounds
        info.name[n] = b'\0' as _;

        info.supported_dialects = self.supported_dialects;
        info.preferred_dialect = self.preferred_dialect;
    }
}

pub(crate) use ffi::PluginNotePorts;

mod ffi {
    use std::marker::PhantomData;

    use crate::{
        ext::note_ports::NotePorts,
        ffi::{clap_note_port_info, clap_plugin, clap_plugin_note_ports},
        plugin::{ClapPlugin, Plugin},
    };

    extern "C-unwind" fn count<E, P>(plugin: *const clap_plugin, is_input: bool) -> u32
    where
        P: Plugin,
        E: NotePorts<P>,
    {
        if plugin.is_null() {
            return 0;
        }
        // SAFETY: We just checked that the pointer is non-null and the plugin
        // has been obtained from host and is tied to type P.
        let mut clap_plugin = unsafe { ClapPlugin::<P>::new_unchecked(plugin) };

        // SAFETY: This function is called on the main thread.
        // It is guaranteed that we are the only function accessing the plugin now.
        // So the mutable reference to plugin for the duration of this call is
        // safe.
        let plugin = unsafe { clap_plugin.plugin() };

        E::count(plugin, is_input)
    }

    extern "C-unwind" fn get<E, P>(
        plugin: *const clap_plugin,
        index: u32,
        is_input: bool,
        info: *mut clap_note_port_info,
    ) -> bool
    where
        P: Plugin,
        E: NotePorts<P>,
    {
        if plugin.is_null() {
            return false;
        }
        // SAFETY: We just checked that the pointer is non-null and the plugin
        // has been obtained from host and is tied to type P.
        let mut clap_plugin = unsafe { ClapPlugin::<P>::new_unchecked(plugin) };

        // SAFETY: This function is called on the main thread.
        // It is guaranteed that we are the only function accessing the plugin now.
        // So the mutable reference to plugin for the duration of this call is
        // safe.
        let plugin = unsafe { clap_plugin.plugin() };

        // SAFETY: The host guarantees we are the only function that can access info
        // for the duration of the function call.  So obtaining a mutable reference
        // is safe.
        let info = unsafe { &mut *info };

        E::get(plugin, index, is_input)
            .map(|x| x.fill_clap_note_port_info(info))
            .is_some()
    }

    pub struct PluginNotePorts<P> {
        #[allow(unused)]
        clap_plugin_note_ports: clap_plugin_note_ports,
        _marker: PhantomData<P>,
    }

    impl<P: Plugin> PluginNotePorts<P> {
        pub fn new<E: NotePorts<P>>(_: E) -> Self {
            Self {
                clap_plugin_note_ports: clap_plugin_note_ports {
                    count: Some(count::<E, P>),
                    get: Some(get::<E, P>),
                },
                _marker: PhantomData,
            }
        }
    }
}

#[derive(Debug)]
pub enum Error {}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "")
    }
}

impl std::error::Error for Error {}

impl From<Error> for crate::Error {
    fn from(value: Error) -> Self {
        crate::ext::Error::NotePorts(value).into()
    }
}
