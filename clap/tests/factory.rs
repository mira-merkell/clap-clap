use std::{ffi::CStr, ptr::null};

use clap::factory::{
    Error::{CreateHost, IndexOutOfBounds, PluginIdNotFound},
    Factory, FactoryHost, FactoryPluginDescriptor,
};

use crate::helpers::{Dummy, DummyHost};

mod helpers;

#[test]
fn factory_empty() {
    let factory = Factory::new(vec![]);
    assert_eq!(factory.plugins_count(), 0);

    assert_eq!(factory.descriptor(0).unwrap_err(), IndexOutOfBounds(0));
    assert_eq!(factory.descriptor(1).unwrap_err(), IndexOutOfBounds(1));
}

fn dummy(n: usize) -> Factory {
    Factory::new(
        (0..n)
            .map(|_| Box::new(FactoryPluginDescriptor::<Dummy>::allocate()) as _)
            .collect(),
    )
}

#[test]
fn factory_dummy_desc() {
    let factory = dummy(1);

    assert_eq!(factory.descriptor(2).unwrap_err(), IndexOutOfBounds(2));
    assert_eq!(factory.descriptor(1).unwrap_err(), IndexOutOfBounds(1));
    let desc = factory.descriptor(0).unwrap();

    assert_eq!(unsafe { CStr::from_ptr((*desc).id) }, c"dummy");
}

#[test]
fn factory_dummy_create_bad() {
    let factory = dummy(1);

    assert_eq!(
        factory
            .clap_plugin(c"", unsafe { FactoryHost::new(null()) })
            .unwrap_err(),
        PluginIdNotFound
    );
    matches!(
        factory
            .clap_plugin(c"dummy", unsafe { FactoryHost::new(null()) })
            .unwrap_err(),
        CreateHost(_)
    );
}

#[test]
fn factory_dummy_create() {
    let factory = dummy(1);

    let plugin = factory
        .clap_plugin(c"dummy", unsafe {
            FactoryHost::new(DummyHost::new().as_clap_host())
        })
        .unwrap();

    let id = unsafe { CStr::from_ptr((*(*plugin).desc).id) };
    assert_eq!(id, c"dummy");

    unsafe { (*plugin).destroy.unwrap()(plugin) }
}
