use crate::plugin::{ClapPluginData, Plugin, PluginDescriptor};
use clap_sys::{clap_host, clap_plugin, clap_plugin_descriptor};
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::ptr::NonNull;

/// This type exists only be visible from within `clap::entry!` macro.
pub struct FactoryHost {
    _host: NonNull<clap_host>,
}

impl FactoryHost {
    pub const fn new(host: NonNull<clap_host>) -> Self {
        Self { _host: host }
    }
}
/// This type exists only be visible from within `clap::entry!` macro.
pub struct FactoryPluginDescriptor<P>(PluginDescriptor<P>);

impl<P: Plugin> FactoryPluginDescriptor<P> {
    pub fn allocate() -> Self {
        Self(PluginDescriptor::allocate())
    }
}

pub trait FactoryPlugin {
    fn plugin_id(&self) -> &CStr;
    fn clap_plugin_descriptor(&self) -> &clap_plugin_descriptor;
    fn boxed_clap_plugin(&self, host: FactoryHost) -> Box<clap_plugin>;
}

impl<P: Plugin> FactoryPlugin for FactoryPluginDescriptor<P> {
    fn plugin_id(&self) -> &CStr {
        &self.0.id
    }

    fn clap_plugin_descriptor(&self) -> &clap_plugin_descriptor {
        &self.0.raw_descriptor
    }

    fn boxed_clap_plugin(&self, host: FactoryHost) -> Box<clap_plugin> {
        ClapPluginData::generate(P::default(), host).boxed_clap_plugin()
    }
}

pub struct Factory {
    id_map: HashMap<CString, usize>,
    plugins: Vec<Box<dyn FactoryPlugin>>,
}

impl Factory {
    pub fn new(plugins: Vec<Box<dyn FactoryPlugin>>) -> Self {
        Self {
            id_map: plugins
                .iter()
                .enumerate()
                .map(|(i, p)| (CString::from(p.plugin_id()), i))
                .collect(),
            plugins,
        }
    }

    pub fn plugins_count(&self) -> u32 {
        self.plugins
            .len()
            .try_into()
            .expect("plugins_count should fit into u32")
    }

    pub fn descriptor(&self, index: u32) -> &clap_plugin_descriptor {
        self.plugins[usize::try_from(index).expect("index should fit into usize")]
            .clap_plugin_descriptor()
    }

    pub fn boxed_clap_plugin(
        &self,
        plugin_id: &CStr,
        host: FactoryHost,
    ) -> Option<Box<clap_plugin>> {
        self.id_map
            .get(plugin_id)
            .map(|&i| self.plugins[i].boxed_clap_plugin(host))
    }
}

unsafe impl Send for Factory {}
unsafe impl Sync for Factory {}
