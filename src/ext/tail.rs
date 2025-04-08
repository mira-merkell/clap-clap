use crate::plugin::Plugin;

pub trait Tail<P: Plugin> {
    /// Any value greater or equal to [`i32::MAX`] implies infinite tail.
    ///
    /// [`i32::MAX`]: i32::MAX
    fn get(plugin: &P) -> u32;
}

pub(crate) use ffi::PluginTail;

use crate::{ffi::clap_host_tail, host::Host};

mod ffi {
    use std::marker::PhantomData;

    use crate::{
        ext::tail::Tail,
        ffi::{clap_plugin, clap_plugin_tail},
        plugin::{ClapPlugin, Plugin},
    };

    extern "C-unwind" fn get<E, P>(plugin: *const clap_plugin) -> u32
    where
        E: Tail<P>,
        P: Plugin,
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

        E::get(plugin)
    }

    pub(crate) struct PluginTail<P> {
        #[allow(unused)]
        clap_plugin_tail: clap_plugin_tail,
        _marker: PhantomData<P>,
    }

    impl<P: Plugin> PluginTail<P> {
        pub(crate) fn new<E: Tail<P>>(_: E) -> Self {
            Self {
                clap_plugin_tail: clap_plugin_tail {
                    get: Some(get::<E, P>),
                },
                _marker: PhantomData,
            }
        }
    }
}

impl<P: Plugin> Tail<P> for () {
    fn get(_: &P) -> u32 {
        0
    }
}

#[derive(Debug)]
pub struct HostTail<'a> {
    host: &'a Host,
    clap_host_tail: &'a clap_host_tail,
}

impl<'a> HostTail<'a> {
    /// # Safety
    ///
    /// All extension interface function pointers must be non-null (Some), and
    /// the functions must be thread-safe.
    pub(crate) const unsafe fn new_unchecked(
        host: &'a Host,
        clap_host_tail: &'a clap_host_tail,
    ) -> Self {
        Self {
            host,
            clap_host_tail,
        }
    }

    pub fn changed(&self) {
        // SAFETY: By construction, the callback must be a valid function pointer,
        // and the call is thread-safe.
        let callback = self.clap_host_tail.changed.unwrap();
        unsafe { callback(self.host.clap_host()) }
    }
}
