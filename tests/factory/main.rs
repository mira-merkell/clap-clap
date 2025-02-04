use std::ffi::CStr;

use clap_clap::{
    Error,
    factory::{
        Error::{IndexOutOfBounds, PluginIdNotFound},
        Factory, FactoryHost, FactoryPluginDescriptor,
    },
    plugin::Plugin,
};

use crate::{dummy_host::DummyHost, dummy_plugin::Dummy};

#[path = "../plugin/dummy.rs"]
mod dummy_plugin;

#[path = "../host/dummy.rs"]
mod dummy_host;

#[test]
pub fn empty() {
    let factory = Factory::new(vec![]);
    assert_eq!(factory.plugins_count(), 0);

    assert_eq!(factory.descriptor(0).unwrap_err(), IndexOutOfBounds(0));
    assert_eq!(factory.descriptor(1).unwrap_err(), IndexOutOfBounds(1));
}

fn dummy(n: usize) -> Factory {
    Factory::new(
        (0..n)
            .map(|_| {
                Box::new(FactoryPluginDescriptor::<Dummy>::build_plugin_descriptor().unwrap()) as _
            })
            .collect(),
    )
}

#[test]
fn dummy_desc() {
    let factory = dummy(1);

    assert_eq!(factory.descriptor(2).unwrap_err(), IndexOutOfBounds(2));
    assert_eq!(factory.descriptor(1).unwrap_err(), IndexOutOfBounds(1));
    let desc = factory.descriptor(0).unwrap();

    assert_eq!(unsafe { CStr::from_ptr((*desc).id) }, c"dummy");
}

static DUMMY_HOST: DummyHost = DummyHost::new();

#[test]
fn dummy_create() {
    let factory = dummy(1);

    let plugin = factory
        .create_plugin(c"dummy", unsafe {
            FactoryHost::new(DUMMY_HOST.as_clap_host())
        })
        .unwrap();

    let id = unsafe { CStr::from_ptr((*(*plugin).desc).id) };
    assert_eq!(id, c"dummy");

    unsafe { (*plugin).destroy.unwrap()(plugin) }
}

#[derive(Default)]
struct DummyToo(Dummy);

impl Plugin for DummyToo {
    type AudioThread = ();
    type Extensions = ();
    const ID: &'static str = "also dummy";
    const NAME: &'static str = "Dummy II";

    fn activate(
        &mut self,
        sample_rate: f64,
        min_frames: u32,
        max_frames: u32,
    ) -> Result<Self::AudioThread, Error> {
        self.0.activate(sample_rate, min_frames, max_frames)
    }
}

fn two_dummies() -> Factory {
    Factory::new(vec![
        Box::new(FactoryPluginDescriptor::<Dummy>::build_plugin_descriptor().unwrap()),
        Box::new(FactoryPluginDescriptor::<DummyToo>::build_plugin_descriptor().unwrap()),
    ])
}

#[test]
fn two_dummies_count() {
    let factory = two_dummies();

    assert_eq!(factory.plugins_count(), 2);
}

#[test]
fn two_dummies_desc0() {
    let factory = two_dummies();

    let _ = factory.descriptor(2).unwrap_err();
    let desc = factory.descriptor(0).unwrap();
    let id = unsafe { *desc }.id;
    let id = unsafe { CStr::from_ptr(id) };
    assert_eq!(id, c"dummy");
}

#[test]
fn two_dummies_desc1() {
    let factory = two_dummies();

    let _ = factory.descriptor(2).unwrap_err();
    let desc = factory.descriptor(1).unwrap();
    let id = unsafe { *desc }.id;
    let id = unsafe { CStr::from_ptr(id) };
    assert_eq!(id, c"also dummy");
}

#[test]
fn two_dummies_create0() {
    let factory = two_dummies();

    let plugin = factory
        .create_plugin(c"dummy", unsafe {
            FactoryHost::new(DUMMY_HOST.as_clap_host())
        })
        .unwrap();

    let id = unsafe { CStr::from_ptr((*(*plugin).desc).id) };
    assert_eq!(id, c"dummy");

    unsafe { (*plugin).destroy.unwrap()(plugin) }
}

#[test]
fn two_dummies_create1() {
    let factory = two_dummies();

    let plugin = factory
        .create_plugin(c"also dummy", unsafe {
            FactoryHost::new(DUMMY_HOST.as_clap_host())
        })
        .unwrap();

    let id = unsafe { CStr::from_ptr((*(*plugin).desc).id) };
    assert_eq!(id, c"also dummy");

    unsafe { (*plugin).destroy.unwrap()(plugin) }
}

#[test]
fn two_dummies_create_badid() {
    let factory = two_dummies();

    let err = factory
        .create_plugin(c"noname", unsafe {
            FactoryHost::new(DUMMY_HOST.as_clap_host())
        })
        .unwrap_err();

    assert_eq!(err, PluginIdNotFound);
}
