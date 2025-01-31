pub use clap_sys::{
    CLAP_PLUGIN_FACTORY_ID, CLAP_VERSION, clap_host, clap_plugin, clap_plugin_descriptor,
    clap_plugin_entry, clap_plugin_factory,
};

#[macro_export]
macro_rules! entry {
    ($($plug:ty),*) => {
        mod _clap_entry {
            use $crate::entry::*;
            use $crate::factory::{Factory, FactoryHost, FactoryPluginDescriptor};
            use $crate::plugin::Plugin;

            pub fn plugin_prototype<P: Plugin>() -> Box<FactoryPluginDescriptor<P>> {
                Box::new(FactoryPluginDescriptor::allocate())
            }

            use super::*; // Access the types supplied as macro arguments.

            static FACTORY: std::sync::OnceLock<Factory> = std::sync::OnceLock::new();

            fn factory_init_once<'a>() -> &'a Factory {
                FACTORY.get_or_init(|| Factory::new(vec![$(plugin_prototype::<$plug>(),)*]))
            }

            /// Safety:
            ///
            /// CLAP requires this method to be thread safe.
            /// The function factory_init_once() is thread-safe and
            /// plugins_count() takes a shared reference to Factory.
            /// Together, they are thread-safe.
            extern "C" fn get_plugin_count(_: *const clap_plugin_factory) -> u32 {
                factory_init_once().plugins_count()
            }

            /// Safety:
            ///
            /// CLAP requires this method to be thread safe.
            /// The function factory_init_once() is thread-safe and
            /// descriptor() takes a shared reference to Factory.
            /// Together, they are thread-safe.
            extern "C" fn get_plugin_descriptor(
                _: *const clap_plugin_factory,
                index: u32,
            ) -> *const clap_plugin_descriptor {
                factory_init_once().descriptor(index)
                    .unwrap_or(std::ptr::null())
            }

            /// Safety:
            ///
            /// CLAP requires this method to be thread safe.
            /// The function factory_init_once() is thread-safe and
            /// boxed_clap_plugin() takes a shared reference to Factory.
            /// Together, they are thread-safe.
            extern "C" fn create_plugin(
                _: *const clap_plugin_factory,
                host: *const clap_host,
                plugin_id: *const std::ffi::c_char,
            ) -> *const clap_plugin {
                if plugin_id.is_null() || host.is_null() {
                    return std::ptr::null();
                }

                // Safety: We just checked that host is non-null.
                let host = FactoryHost::new(unsafe{ std::ptr::NonNull::new_unchecked(host as *mut _)});
                // Safety: We checked if plug_id is non-null.
                // The host guarantees that this is a valid C string now.
                let plugin_id = unsafe { std::ffi::CStr::from_ptr(plugin_id) };
                factory_init_once()
                        .boxed_clap_plugin(plugin_id, host)
                        .map(Box::into_raw).unwrap_or(std::ptr::null_mut())
            }

            static CLAP_PLUGIN_FACTORY: clap_plugin_factory = clap_plugin_factory {
                get_plugin_count: Some(get_plugin_count),
                get_plugin_descriptor: Some(get_plugin_descriptor),
                create_plugin: Some(create_plugin),
            };

            extern "C" fn init(plugin_path: *const std::ffi::c_char) -> bool {
                !plugin_path.is_null()
            }

            extern "C" fn deinit() {}

            /// Safety:
            ///
            /// CLAP requires this method to be thread safe.
            extern "C" fn get_factory(factory_id: *const std::ffi::c_char) -> *const std::ffi::c_void {
                if factory_id.is_null() {
                    return std::ptr::null();
                }
                // Safety: we cheched if factory_id is non-null.
                // The host guarantees that this is a valid C string.
                let id = unsafe { std::ffi::CStr::from_ptr(factory_id) };
                if id == CLAP_PLUGIN_FACTORY_ID {
                    &raw const CLAP_PLUGIN_FACTORY as *const _
                } else { std::ptr::null() }
            }

            #[allow(non_upper_case_globals)]
            #[allow(warnings, unused)]
            #[unsafe(no_mangle)]
            // Make this symbor pub(crate), so that tests cat access it.
            pub(crate) static clap_entry: clap_plugin_entry = clap_plugin_entry {
                clap_version: CLAP_VERSION,
                init: Some(init),
                deinit: Some(deinit),
                get_factory: Some(get_factory),
            };
        }
    };
}
