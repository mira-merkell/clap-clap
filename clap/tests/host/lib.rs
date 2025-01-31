mod bad;

use std::{
    ffi::{CStr, c_char, c_void},
    ptr::{null, null_mut},
};

use clap::{
    Error,
    factory::{Factory, FactoryPluginDescriptor},
    plugin::Plugin,
};

const NAME: &CStr = c"test.plugin";

#[derive(Default)]
struct TestPlug;

impl Plugin for TestPlug {
    const ID: &'static str = "test.plugin";
    type AudioThread = ();
    type Extensions = ();

    fn activate(&mut self, _: f64, _: usize, _: usize) -> Result<(), Error> {
        Ok(())
    }
}

struct DummyHost(clap_sys::clap_host);

impl DummyHost {
    const fn new() -> Self {
        extern "C" fn get_extensions(
            _: *const clap_sys::clap_host,
            _: *const c_char,
        ) -> *const c_void {
            null()
        }
        extern "C" fn request_restart(_: *const clap_sys::clap_host) {}
        extern "C" fn request_process(_: *const clap_sys::clap_host) {}
        extern "C" fn request_callback(_: *const clap_sys::clap_host) {}

        Self(clap_sys::clap_host {
            clap_version: clap_sys::CLAP_VERSION,
            host_data: null_mut(),
            name: c"dummy".as_ptr(),
            vendor: c"⧉⧉⧉".as_ptr(),
            url: c"".as_ptr(),
            version: c"1".as_ptr(),
            get_extension: Some(get_extensions),
            request_restart: Some(request_restart),
            request_process: Some(request_process),
            request_callback: Some(request_callback),
        })
    }

    fn as_ptr(&self) -> *const clap_sys::clap_host {
        &self.0
    }
}

fn create_factory() -> Factory {
    Factory::new(vec![Box::new(
        FactoryPluginDescriptor::<TestPlug>::allocate(),
    )])
}
