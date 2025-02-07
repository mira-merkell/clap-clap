use std::{
    collections::HashMap,
    ffi::{CStr, CString, c_char},
    marker::PhantomData,
    ptr::null,
};

use crate::{
    clap_sys::{CLAP_VERSION, clap_plugin_descriptor},
    plugin::{Error, Plugin, desc::PluginDescriptorKey::*},
};

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
enum PluginDescriptorKey {
    Id,
    Name,
    Vendor,
    Url,
    ManualUrl,
    SupportUrl,
    Version,
    Description,
}

#[derive(Default)]
struct PluginDescriptorBuilder {
    map: HashMap<PluginDescriptorKey, CString>,
    features: Vec<CString>,
}

impl PluginDescriptorBuilder {
    fn add_feature(&mut self, feature: &str) -> Result<&mut Self, Error> {
        self.features
            .push(CString::new(feature).map_err(Error::NulError)?);
        Ok(self)
    }

    fn build<P: Plugin>(&self) -> Result<PluginDescriptor<P>, Error> {
        // Check for mandatory fields
        if !self.map.contains_key(&Id) || !self.map.contains_key(&Name) {
            return Err(Error::MissingFields);
        }
        Ok(PluginDescriptor::new(
            self.map.clone(),
            self.features.clone().into(),
        ))
    }
}

macro_rules! impl_builder_methods {
    ($(($method:tt, $key:ident)),*) => {
        impl PluginDescriptorBuilder {
            $(
                fn $method(&mut self, value: &str) -> Result<&mut Self, Error> {
                    use PluginDescriptorKey::*;
                    self.map
                        .insert($key, CString::new(value).map_err(Error::NulError)?);
                    Ok(self)
                }
            )*
        }
    };
}

impl_builder_methods!(
    (id, Id),
    (name, Name),
    (vendor, Vendor),
    (url, Url),
    (manual_url, ManualUrl),
    (support_url, SupportUrl),
    (version, Version),
    (description, Description)
);

#[allow(dead_code)]
pub(crate) struct PluginDescriptor<P> {
    pub(crate) clap_plugin_descriptor: clap_plugin_descriptor,
    clap_features: Box<[*const c_char]>,

    map: HashMap<PluginDescriptorKey, CString>,
    features: Box<[CString]>,

    _marker: PhantomData<P>,
}

impl<P: Plugin> PluginDescriptor<P> {
    fn new(map: HashMap<PluginDescriptorKey, CString>, features: Box<[CString]>) -> Self {
        let mut clap_features: Vec<_> = features.iter().map(|f| f.as_c_str().as_ptr()).collect();
        clap_features.push(null());
        let clap_features = clap_features.into_boxed_slice();

        let get_ptr_to_key = |key: PluginDescriptorKey| -> *const c_char {
            map.get(&key)
                .map(|s| s.as_c_str().as_ptr())
                .unwrap_or(null())
        };

        Self {
            clap_plugin_descriptor: clap_plugin_descriptor {
                clap_version: CLAP_VERSION,
                id: get_ptr_to_key(Id),
                name: get_ptr_to_key(Name),
                vendor: get_ptr_to_key(Vendor),
                url: get_ptr_to_key(Url),
                manual_url: get_ptr_to_key(ManualUrl),
                support_url: get_ptr_to_key(SupportUrl),
                version: get_ptr_to_key(Version),
                description: get_ptr_to_key(Description),
                features: clap_features.as_ptr(),
            },
            clap_features,
            map,
            features,
            _marker: PhantomData,
        }
    }

    pub fn plugin_id(&self) -> &CStr {
        // The builder guarantees that Id is set, so this won't panic.
        self.map.get(&Id).unwrap().as_ref()
    }
}

pub fn build_plugin_descriptor<P: Plugin>() -> Result<PluginDescriptor<P>, Error> {
    let mut builder = PluginDescriptorBuilder::default();
    builder
        .id(P::ID)?
        .name(P::NAME)?
        .vendor(P::VENDOR)?
        .url(P::URL)?
        .manual_url(P::MANUAL_URL)?
        .support_url(P::SUPPORT_URL)?
        .version(P::VERSION)?
        .description(P::DESCRIPTION)?;

    P::FEATURES
        .split_whitespace()
        .try_for_each(|feat| builder.add_feature(feat).map(|_| ()))?;

    builder.build()
}
