use std::{
    ffi::{CString, c_char},
    marker::PhantomData,
    ptr::null,
    str::FromStr,
};

use clap_sys::{CLAP_VERSION, clap_plugin_descriptor};

use crate::plugin::Plugin;

#[allow(dead_code)]
pub(crate) struct PluginDescriptor<P> {
    pub(crate) id: CString,
    name: CString,
    vendor: CString,
    url: CString,
    manual_url: CString,
    support_url: CString,
    version: CString,
    description: CString,
    features: Box<[CString]>,

    raw_features: Box<[*const c_char]>,
    pub(crate) raw_descriptor: clap_plugin_descriptor,
    _marker: PhantomData<P>,
}

impl<P: Plugin> PluginDescriptor<P> {
    pub fn allocate() -> Self {
        let id = CString::from_str(P::ID).unwrap();
        let name = CString::from_str(P::NAME).unwrap();
        let vendor = CString::from_str(P::VENDOR).unwrap();
        let url = CString::from_str(P::URL).unwrap();
        let manual_url = CString::from_str(P::MANUAL_URL).unwrap();
        let support_url = CString::from_str(P::SUPPORT_URL).unwrap();
        let version = CString::from_str(P::VERSION).unwrap();
        let description = CString::from_str(P::DESCRIPTION).unwrap();

        let features: Vec<_> = String::from_str(P::FEATURES)
            .unwrap()
            .split_whitespace()
            .map(|s| CString::from_str(s).unwrap())
            .collect();
        let mut features_raw: Vec<_> = features.iter().map(|f| f.as_c_str().as_ptr()).collect();
        features_raw.push(null());
        let features_raw = features_raw.into_boxed_slice();

        let raw = clap_plugin_descriptor {
            clap_version: CLAP_VERSION,
            id: id.as_c_str().as_ptr(),
            name: name.as_c_str().as_ptr(),
            vendor: vendor.as_c_str().as_ptr(),
            url: url.as_c_str().as_ptr(),
            manual_url: manual_url.as_c_str().as_ptr(),
            support_url: support_url.as_c_str().as_ptr(),
            version: version.as_c_str().as_ptr(),
            description: description.as_c_str().as_ptr(),
            features: features_raw.as_ptr(),
        };

        Self {
            id,
            name,
            vendor,
            url,
            manual_url,
            support_url,
            version,
            description,
            features: features.into(),
            raw_features: features_raw,
            raw_descriptor: raw,
            _marker: PhantomData,
        }
    }
}
