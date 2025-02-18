use std::ffi::CStr;

use clap_clap::plugin::{Plugin, PluginDescriptor};

#[derive(Default)]
struct Plug;

impl Plugin for Plug {
    type AudioThread<'a>
        = ()
    where
        Self: 'a;
    type Extensions = ();

    const ID: &'static str = "123.456";
    const NAME: &'static str = "Test Plug";
    const VENDOR: &'static str = "⧉⧉⧉";
    const URL: &'static str = "none";
    const MANUAL_URL: &'static str = "https://example.com";
    const SUPPORT_URL: &'static str = "ftp::/example.com";
    const VERSION: &'static str = "[34";
    const DESCRIPTION: &'static str = "none";
    const FEATURES: &'static str = "fx stereo distor..0[";

    fn activate(
        &mut self,
        _: f64,
        _: u32,
        _: u32,
    ) -> Result<Self::AudioThread<'_>, clap_clap::Error> {
        Ok(())
    }
}

macro_rules! check_desc_field {
    ($name:tt, $field:ident $(,)?) => {
        #[allow(non_snake_case)]
        #[test]
        fn $name() {
            let desc = PluginDescriptor::new::<Plug>().unwrap();
            let name = unsafe { CStr::from_ptr(desc.clap_plugin_descriptor().$field) }
                .to_str()
                .unwrap();

            assert_eq!(Plug::$name, name);
        }
    };
}

check_desc_field!(ID, id);
check_desc_field!(NAME, name);
check_desc_field!(VENDOR, vendor);
check_desc_field!(URL, url);
check_desc_field!(MANUAL_URL, manual_url);
check_desc_field!(SUPPORT_URL, support_url);
check_desc_field!(VERSION, version);
check_desc_field!(DESCRIPTION, description);

#[allow(non_snake_case)]
#[test]
fn FEATURES() {
    let desc = PluginDescriptor::new::<Plug>().unwrap();
    let mut features = Vec::new();
    let mut feat = desc.clap_plugin_descriptor().features;
    while !unsafe { *feat }.is_null() {
        features.push(
            unsafe { CStr::from_ptr(*feat) }
                .to_str()
                .unwrap()
                .to_owned(),
        );
        feat = unsafe { feat.add(1) };
    }

    let expected: Vec<String> = Plug::FEATURES
        .split_whitespace()
        .map(|s| s.to_owned())
        .collect();

    assert_eq!(features, expected);
}

#[test]
fn features_is_null_terminated() {
    let desc = PluginDescriptor::new::<Plug>().unwrap();
    let feat = desc.clap_plugin_descriptor().features;

    let expected: Vec<&str> = Plug::FEATURES.split_whitespace().collect();

    let feat_term = unsafe { feat.add(expected.len()) };
    assert!(unsafe { *feat_term }.is_null());
}

#[test]
fn valid_after_move() {
    let desc = PluginDescriptor::new::<Plug>().unwrap();
    let name = unsafe { CStr::from_ptr(desc.clap_plugin_descriptor().name) }
        .to_str()
        .unwrap();
    assert_eq!(Plug::NAME, name);

    let boxed = Box::new(desc);
    let name = unsafe { CStr::from_ptr(boxed.clap_plugin_descriptor().name) }
        .to_str()
        .unwrap();
    assert_eq!(Plug::NAME, name);

    let desc = *boxed;
    let name = unsafe { CStr::from_ptr(desc.clap_plugin_descriptor().name) }
        .to_str()
        .unwrap();
    assert_eq!(Plug::NAME, name);
}
