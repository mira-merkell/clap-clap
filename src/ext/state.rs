use std::fmt::{Display, Formatter};

use crate::{
    plugin::Plugin,
    stream::{IStream, OStream},
};

pub trait State<P: Plugin> {
    /// Saves the plugin state into stream.
    ///
    /// # Return
    ///
    /// Returns `Ok` if the state was correctly saved.
    fn save(plugin: &P, stream: &mut OStream) -> Result<(), crate::Error>;

    /// Loads the plugin state from stream.
    ///
    /// # Return
    ///
    /// Returns `Ok` if the state was correctly restored.
    fn load(plugin: &P, stream: &mut IStream) -> Result<(), crate::Error>;
}

pub(crate) use ffi::PluginState;

mod ffi {
    use std::marker::PhantomData;

    use crate::{
        ext::state::State,
        ffi::{clap_istream, clap_ostream, clap_plugin, clap_plugin_state},
        plugin::{ClapPlugin, Plugin},
        stream::{IStream, OStream},
    };

    extern "C-unwind" fn save<E, P>(plugin: *const clap_plugin, stream: *const clap_ostream) -> bool
    where
        E: State<P>,
        P: Plugin,
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

        if unsafe { stream.as_ref().and_then(|s| s.write) }.is_none() {
            return false;
        }
        // SAFETY: We just checked if both stream and stream.write are non-null.
        let mut stream = unsafe { OStream::new_unchecked(stream) };

        E::save(plugin, &mut stream).is_ok()
    }

    extern "C-unwind" fn load<E, P>(plugin: *const clap_plugin, stream: *const clap_istream) -> bool
    where
        E: State<P>,
        P: Plugin,
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

        if unsafe { stream.as_ref().and_then(|s| s.read) }.is_none() {
            return false;
        }
        // SAFETY: We just checked if both stream and stream.read are non-null.
        let mut stream = unsafe { IStream::new_unchecked(stream) };

        E::load(plugin, &mut stream).is_ok()
    }

    pub(crate) struct PluginState<P> {
        #[allow(unused)]
        clap_plugin_state: clap_plugin_state,
        _marker: PhantomData<P>,
    }

    impl<P: Plugin> PluginState<P> {
        pub(crate) fn new<E: State<P>>(_: E) -> Self {
            Self {
                clap_plugin_state: clap_plugin_state {
                    save: Some(save::<E, P>),
                    load: Some(load::<E, P>),
                },
                _marker: PhantomData,
            }
        }
    }
}

impl<P: Plugin> State<P> for () {
    fn save(_: &P, _: &mut OStream) -> Result<(), crate::Error> {
        Ok(())
    }

    fn load(_: &P, _: &mut IStream) -> Result<(), crate::Error> {
        Ok(())
    }
}

#[derive(Debug)]
pub enum Error {
    Read,
    Write,
    Eof,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use Error::*;
        match self {
            Read => write!(f, "read error"),
            Write => write!(f, "write error"),
            Eof => write!(f, "end of file"),
        }
    }
}

impl std::error::Error for Error {}

impl From<Error> for crate::Error {
    fn from(value: Error) -> Self {
        crate::ext::Error::State(value).into()
    }
}
