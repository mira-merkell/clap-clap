use std::{
    collections::HashMap,
    ffi::{CStr, CString},
    fmt::{Display, Formatter},
    marker::PhantomData,
    sync::Arc,
};

use crate::{
    ffi::{clap_host, clap_plugin, clap_plugin_descriptor},
    host::Host,
    plugin,
    plugin::{Plugin, PluginDescriptor, Runtime},
};

pub struct FactoryHost(*const clap_host);

impl FactoryHost {
    /// # Safety
    ///
    /// The pointer to `clap_host` and all it's methods must be non-null.
    /// It must point to a valid CLAP host that a plugin can be
    /// initialized with.
    pub const unsafe fn new_unchecked(host: *const clap_host) -> Self {
        debug_assert!(!host.is_null());
        Self(host)
    }
}

// SAFETY: !Send for raw pointers is not for safety, just as a lint.
unsafe impl Send for FactoryHost {}
// SAFETY: !Sync for raw pointers is not for safety, just as a lint.
unsafe impl Sync for FactoryHost {}

pub struct FactoryPluginPrototype<P> {
    descriptor: PluginDescriptor,
    _marker: PhantomData<P>,
}

impl<P: Plugin> FactoryPluginPrototype<P> {
    pub fn build() -> Result<Self, Error> {
        Ok(Self {
            descriptor: PluginDescriptor::new::<P>().map_err(Error::PluginDescriptor)?,
            _marker: PhantomData,
        })
    }
}

pub trait FactoryPlugin {
    fn plugin_id(&self) -> &CStr;

    /// # Safety
    ///
    /// The caller must assure that the pointer will remain valid for the
    /// intended use.
    unsafe fn clap_plugin_descriptor(&self) -> *const clap_plugin_descriptor;

    /// # Safety
    ///
    /// The caller must assure that the pointer will remain valid for the
    /// intended use.
    unsafe fn clap_plugin(&self, host: FactoryHost) -> Result<*const clap_plugin, Error>;
}

impl<P: Plugin> FactoryPlugin for FactoryPluginPrototype<P> {
    fn plugin_id(&self) -> &CStr {
        self.descriptor.plugin_id()
    }

    unsafe fn clap_plugin_descriptor(&self) -> *const clap_plugin_descriptor {
        &raw const *self.descriptor.clap_plugin_descriptor()
    }

    unsafe fn clap_plugin(&self, host: FactoryHost) -> Result<*const clap_plugin, Error> {
        // SAFETY: The pointer unwrapped from FactoryHost is a valid pointer
        // to a CLAP host, obtained as the argument passed to plugin
        // factory's create_plugin().
        let host = unsafe { Host::new(host.0) };
        Ok(Runtime::<P>::initialize(Arc::new(host))
            .map_err(Error::PluginDescriptor)?
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
        debug_assert!(u32::try_from(self.plugins.len()).is_ok());
        self.plugins.len() as u32
    }

    pub fn descriptor(&self, index: u32) -> Result<*const clap_plugin_descriptor, Error> {
        debug_assert!(usize::try_from(index).is_ok());
        let index = index as usize;
        (index < self.plugins.len())
            // This needs to be lazy to avoid evaluating on invalid index.
            .then(|| unsafe { self.plugins[index].clap_plugin_descriptor() })
            .ok_or(Error::IndexOutOfBounds(index as u32))
    }

    pub fn create_plugin(
        &self,
        plugin_id: &CStr,
        host: FactoryHost,
    ) -> Result<*const clap_plugin, Error> {
        let i = *self.id_map.get(plugin_id).ok_or(Error::PluginIdNotFound)?;
        unsafe { self.plugins[i].clap_plugin(host) }
    }
}

// SAFETY: !Send for raw pointers is not for safety, just as a lint.
unsafe impl Send for Factory {}
// SAFETY: !Sync for raw pointers is not for safety, just as a lint.
unsafe impl Sync for Factory {}

#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    PluginIdNotFound,
    PluginDescriptor(plugin::Error),
    IndexOutOfBounds(u32),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::PluginIdNotFound => write!(f, "factory plugin id not found"),
            Error::PluginDescriptor(e) => write!(f, "plugin descriptor: {e}"),
            Error::IndexOutOfBounds(n) => write!(f, "index out ouf bounds: {n}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<Error> for crate::Error {
    fn from(value: Error) -> Self {
        crate::Error::Factory(value)
    }
}
