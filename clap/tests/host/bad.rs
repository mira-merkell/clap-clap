use std::ptr::null;

use clap::factory::FactoryHost;

use crate::{DummyHost, NAME, create_factory};

#[test]
#[should_panic(expected = "called `Result::unwrap()` on an `Err` value: PluginIdNotFound")]
fn wrong_plugin_id() {
    let host = DummyHost::new();
    create_factory()
        .boxed_clap_plugin(c"testxxxn", unsafe { FactoryHost::new(host.as_ptr()) })
        .unwrap();
}

#[test]
fn dummy_host() {
    let host = DummyHost::new();
    let plugin = create_factory()
        .boxed_clap_plugin(NAME, unsafe { FactoryHost::new(host.as_ptr()) })
        .unwrap();

    let destroy_plugin = plugin.destroy.unwrap();
    unsafe { destroy_plugin(plugin.as_ref()) };
}

macro_rules! test_host_null_desc {
    ($test_name:ident,$erase_string:ident) => {
        #[test]
        #[should_panic(
            expected = "called `Result::unwrap()` on an `Err` value: CreateHost(NullPtr)"
        )]
        fn $test_name() {
            let mut host = DummyHost::new();

            host.0.$erase_string = null();
            create_factory()
                .boxed_clap_plugin(NAME, unsafe { FactoryHost::new(host.as_ptr()) })
                .unwrap();
        }
    };
}

test_host_null_desc!(bad_host_null_name, name);
test_host_null_desc!(bad_host_null_vendor, vendor);
test_host_null_desc!(bad_host_null_url, url);
test_host_null_desc!(bad_host_null_version, version);

macro_rules! test_host_null_method {
    ($test_name:ident, $method:ident) => {
        #[test]
        #[should_panic(
            expected = "called `Result::unwrap()` on an `Err` value: CreateHost(MethodNotFound("
        )]
        fn $test_name() {
            let mut host = DummyHost::new();

            host.0.$method = None;

            create_factory()
                .boxed_clap_plugin(NAME, unsafe { FactoryHost::new(host.as_ptr()) })
                .unwrap();
        }
    };
}

test_host_null_method!(bad_host_null_get_extension, get_extension);
test_host_null_method!(bad_host_null_request_process, request_process);
test_host_null_method!(bad_host_null_request_restart, request_restart);
test_host_null_method!(bad_host_null_request_callback, request_callback);
