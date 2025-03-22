use crate::plugin::Plugin;

pub trait Latency<P>
where
    P: Plugin,
{
    /// Return the plugin latency in samples.
    fn get(plugin: &P) -> u32;
}

impl<P: Plugin> Latency<P> for () {
    fn get(_: &P) -> u32 {
        0
    }
}

pub(crate) use ffi::PluginLatency;
mod ffi {
    use std::marker::PhantomData;

    use crate::{
        ext::latency::Latency,
        ffi::{clap_plugin, clap_plugin_latency},
        plugin::{ClapPlugin, Plugin},
    };

    extern "C-unwind" fn get<E, P>(plugin: *const clap_plugin) -> u32
    where
        E: Latency<P>,
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

    pub struct PluginLatency<P> {
        #[allow(unused)]
        clap_plugin_latency: clap_plugin_latency,
        _marker: PhantomData<P>,
    }

    impl<P: Plugin> PluginLatency<P> {
        pub fn new<E: Latency<P>>(_: E) -> Self {
            Self {
                clap_plugin_latency: clap_plugin_latency {
                    get: Some(get::<E, P>),
                },
                _marker: PhantomData,
            }
        }
    }
}
