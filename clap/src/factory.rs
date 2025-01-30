use crate::host;
use crate::host::Host;
use crate::plugin::{Plugin, PluginDescriptor, Runtime};
use clap_sys::{clap_host, clap_plugin, clap_plugin_descriptor};
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::fmt::{Display, Formatter};
use std::ptr::NonNull;
use std::sync::Arc;

/// This type exists only be visible from within `clap::entry!` macro.
pub struct FactoryHost {
    host: NonNull<clap_host>,
}

impl FactoryHost {
    pub const fn new(host: NonNull<clap_host>) -> Self {
        Self { host }
    }

    pub(crate) fn into_inner(self) -> NonNull<clap_host> {
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
    fn clap_plugin_descriptor(&self) -> &clap_plugin_descriptor;
    fn boxed_clap_plugin(&self, host: FactoryHost) -> Result<Box<clap_plugin>, Error>;
}

impl<P: Plugin> FactoryPlugin for FactoryPluginDescriptor<P> {
    fn plugin_id(&self) -> &CStr {
        &self.0.id
    }

    fn clap_plugin_descriptor(&self) -> &clap_plugin_descriptor {
        &self.0.raw_descriptor
    }

    fn boxed_clap_plugin(&self, host: FactoryHost) -> Result<Box<clap_plugin>, Error> {
        // Safety:
        // The pointer unwrapped from FactoryHost is a valid pointer
        // to a CLAP host, obtained as the argument passed to plugin
        // factory's create_plugin().
        let host = unsafe { Host::try_from_factory(host) }.map_err(Error::CreateHost)?;
        Ok(Runtime::<P>::initialize(Arc::new(host)).boxed_clap_plugin())
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
    ) -> Result<Box<clap_plugin>, Error> {
        let i = self.id_map.get(plugin_id).ok_or(Error::PluginIdNotFound)?;
        self.plugins[*i].boxed_clap_plugin(host)
    }
}

unsafe impl Send for Factory {}
unsafe impl Sync for Factory {}

#[derive(Debug, Clone)]
pub enum Error {
    PluginIdNotFound,
    CreateHost(host::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::PluginIdNotFound => write!(f, "factory plugin id not found"),
            Error::CreateHost(_) => write!(f, "create host handle"),
        }
    }
}

impl std::error::Error for Error {}

impl From<Error> for crate::Error {
    fn from(value: Error) -> Self {
        crate::Error::Factory(value)
    }
}
