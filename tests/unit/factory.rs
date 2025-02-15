use std::ffi::CStr;

use clap_clap::factory::{
    Error::{IndexOutOfBounds, PluginIdNotFound},
    Factory, FactoryHost, FactoryPluginPrototype,
};

use crate::{
    plugin::TestPlugin,
    shims::{host::SHIM_CLAP_HOST, plugin::ShimPlugin},
};

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
            .map(|_| Box::new(FactoryPluginPrototype::<TestPlugin>::build().unwrap()) as _)
            .collect(),
    )
}

#[test]
fn dummy_desc() {
    let factory = dummy(1);

    assert_eq!(factory.descriptor(2).unwrap_err(), IndexOutOfBounds(2));
    assert_eq!(factory.descriptor(1).unwrap_err(), IndexOutOfBounds(1));
    let desc = factory.descriptor(0).unwrap();

    assert_eq!(unsafe { CStr::from_ptr((*desc).id) }, c"clap.plugin.test");
}

#[test]
fn dummy_create() {
    let factory = dummy(1);

    let plugin = factory
        .create_plugin(c"clap.plugin.test", unsafe {
            FactoryHost::new_unchecked(SHIM_CLAP_HOST.as_ref())
        })
        .unwrap();

    let id = unsafe { CStr::from_ptr((*(*plugin).desc).id) };
    assert_eq!(id, c"clap.plugin.test");

    unsafe { (*plugin).destroy.unwrap()(plugin) }
}

fn two_dummies() -> Factory {
    Factory::new(vec![
        Box::new(FactoryPluginPrototype::<TestPlugin>::build().unwrap()),
        Box::new(FactoryPluginPrototype::<ShimPlugin>::build().unwrap()),
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
    assert_eq!(id, c"clap.plugin.test");
}

#[test]
fn two_dummies_desc1() {
    let factory = two_dummies();

    let _ = factory.descriptor(2).unwrap_err();
    let desc = factory.descriptor(1).unwrap();
    let id = unsafe { *desc }.id;
    let id = unsafe { CStr::from_ptr(id) };
    assert_eq!(id, c"");
}

#[test]
fn two_dummies_create0() {
    let factory = two_dummies();

    let plugin = factory
        .create_plugin(c"", unsafe {
            FactoryHost::new_unchecked(SHIM_CLAP_HOST.as_ref())
        })
        .unwrap();

    let id = unsafe { CStr::from_ptr((*(*plugin).desc).id) };
    assert_eq!(id, c"");

    unsafe { (*plugin).destroy.unwrap()(plugin) }
}

#[test]
fn two_dummies_create1() {
    let factory = two_dummies();

    let plugin = factory
        .create_plugin(c"", unsafe {
            FactoryHost::new_unchecked(SHIM_CLAP_HOST.as_ref())
        })
        .unwrap();

    let id = unsafe { CStr::from_ptr((*(*plugin).desc).id) };
    assert_eq!(id, c"");

    unsafe { (*plugin).destroy.unwrap()(plugin) }
}

#[test]
fn two_dummies_create_badid() {
    let factory = two_dummies();

    let err = factory
        .create_plugin(c"noname", unsafe {
            FactoryHost::new_unchecked(SHIM_CLAP_HOST.as_ref())
        })
        .unwrap_err();

    assert_eq!(err, PluginIdNotFound);
}
