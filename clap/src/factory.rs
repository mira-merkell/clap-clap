use std::{
    collections::HashMap,
    ffi::{CStr, CString},
    fmt::{Display, Formatter},
    sync::Arc,
};

use clap_sys::{clap_host, clap_plugin, clap_plugin_descriptor};

use crate::{
    host,
    host::Host,
    plugin::{Plugin, PluginDescriptor, Runtime},
};

/// This type exists only be visible from within `clap::entry!` macro.
pub struct FactoryHost {
    host: *const clap_host,
}

impl FactoryHost {
    /// # Safety
    ///
    /// host must be non-null.
    pub const unsafe fn new(host: *const clap_host) -> Self {
        Self { host }
    }

    pub(crate) fn into_inner(self) -> *const clap_host {
        self.host
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
    fn clap_plugin_descriptor(&self) -> *const clap_plugin_descriptor;
    fn clap_plugin(&self, host: FactoryHost) -> Result<*const clap_plugin, Error>;
}

impl<P: Plugin> FactoryPlugin for FactoryPluginDescriptor<P> {
    fn plugin_id(&self) -> &CStr {
        &self.0.id
    }

    fn clap_plugin_descriptor(&self) -> *const clap_plugin_descriptor {
        &raw const self.0.raw_descriptor
    }

    fn clap_plugin(&self, host: FactoryHost) -> Result<*const clap_plugin, Error> {
        // Safety:
        // The pointer unwrapped from FactoryHost is a valid pointer
        // to a CLAP host, obtained as the argument passed to plugin
        // factory's create_plugin().
        let host =
            unsafe { Host::try_from_factory(host.into_inner()) }.map_err(Error::CreateHost)?;
        Ok(Runtime::<P>::initialize(Arc::new(host))
            .into_clap_plugin()
            .into_inner())
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

    pub fn descriptor(&self, index: u32) -> Result<*const clap_plugin_descriptor, Error> {
        let index = usize::try_from(index).map_err(|_| Error::IndexOutOfBounds)?;
        (index < self.plugins.len())
            // This needs to be lazy to avoid evaluating on invalid index.
            .then(|| self.plugins[index].clap_plugin_descriptor())
            .ok_or(Error::IndexOutOfBounds)
    }

    pub fn clap_plugin(
        &self,
        plugin_id: &CStr,
        host: FactoryHost,
    ) -> Result<*const clap_plugin, Error> {
        let i = self.id_map.get(plugin_id).ok_or(Error::PluginIdNotFound)?;
        self.plugins[*i].clap_plugin(host)
    }
}

unsafe impl Send for Factory {}
unsafe impl Sync for Factory {}

#[derive(Debug, Clone)]
pub enum Error {
    PluginIdNotFound,
    CreateHost(host::Error),
    IndexOutOfBounds,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::PluginIdNotFound => write!(f, "factory plugin id not found"),
            Error::CreateHost(_) => write!(f, "create host handle"),
            Error::IndexOutOfBounds => write!(f, "index out ouf bounds"),
        }
    }
}

impl std::error::Error for Error {}

impl From<Error> for crate::Error {
    fn from(value: Error) -> Self {
        crate::Error::Factory(value)
    }
}
