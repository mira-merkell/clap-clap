use std::{
    ffi::{CStr, CString, c_char},
    ptr::null,
};

use crate::{
    ffi::{CLAP_VERSION, clap_plugin_descriptor},
    plugin::{Error, Plugin},
};

#[allow(dead_code)]
pub struct PluginDescriptor {
    clap_plugin_descriptor: clap_plugin_descriptor,
    clap_features: Box<[*const c_char]>,

    id: CString,
    name: CString,
    vendor: CString,
    url: CString,
    manual_url: CString,
    support_url: CString,
    version: CString,
    description: CString,
    features: Box<[CString]>,
}

impl PluginDescriptor {
    pub fn new<P: Plugin>() -> Result<Self, Error> {
        let id = CString::new(P::ID)?;
        let name = CString::new(P::NAME)?;
        let vendor = CString::new(P::VENDOR)?;
        let url = CString::new(P::URL)?;
        let manual_url = CString::new(P::MANUAL_URL)?;
        let support_url = CString::new(P::SUPPORT_URL)?;
        let version = CString::new(P::VERSION)?;
        let description = CString::new(P::DESCRIPTION)?;

        let features: Box<[CString]> = P::FEATURES
            .split_whitespace()
            .map(CString::new)
            .collect::<Result<_, _>>()?;
        let mut clap_features: Vec<*const c_char> =
            features.iter().map(|s| s.as_c_str().as_ptr()).collect();
        clap_features.push(null());
        let clap_features = clap_features.into_boxed_slice();

        Ok(Self {
            clap_plugin_descriptor: clap_plugin_descriptor {
                clap_version: CLAP_VERSION,
                id: id.as_c_str().as_ptr(),
                name: name.as_c_str().as_ptr(),
                vendor: vendor.as_c_str().as_ptr(),
                url: url.as_c_str().as_ptr(),
                manual_url: manual_url.as_c_str().as_ptr(),
                support_url: support_url.as_c_str().as_ptr(),
                version: version.as_c_str().as_ptr(),
                description: description.as_c_str().as_ptr(),
                features: clap_features.as_ptr(),
            },
            clap_features,
            id,
            name,
            vendor,
            url,
            manual_url,
            support_url,
            version,
            description,
            features,
        })
    }

    pub fn plugin_id(&self) -> &CStr {
        self.id.as_ref()
    }

    pub fn clap_plugin_descriptor(&self) -> &clap_plugin_descriptor {
        &self.clap_plugin_descriptor
    }
}
