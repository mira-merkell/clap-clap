/// Export `clap_entry` symbols and build a plugin factory.
///
/// Use this macro to build a CLAP plugin bundle from types that implement
/// the [`Plugin`] trait.
///
/// [`Plugin`]: crate::plugin::Plugin
///
/// # Example
///
/// ```no_compile
/// #[derive(Default)]
/// struct MyPlugin;
///
/// impl Plugin for MyPlugin {
///     ...
/// }
///
/// #[derive(Default)]
/// struct MyPluginToo;
///
/// impl Plugin for MyPluginToo {
///     ...
/// }
///
/// clap_clap::entry!(MyPlugin, MyPluginToo);
/// ```
///
/// The crate that invokes the macro should be a dynamic library with C ABI.
/// Specify the ABI in your crate's `Cargo.toml`:
///
/// ```toml
/// [lib]
/// crate-type = ["cdylib"]
/// ```
#[macro_export]
macro_rules! entry {
    ($($plug:ty),*) => {
        mod _clap_entry {
            use $crate::ffi::{
                CLAP_PLUGIN_FACTORY_ID, CLAP_VERSION,
                clap_plugin, clap_plugin_descriptor, clap_plugin_entry,
                clap_plugin_factory, clap_host
            };
            use $crate::factory::{Factory, FactoryHost, FactoryPluginDescriptor};
            use $crate::plugin::Plugin;

            use super::*; // Access the types supplied as macro arguments.

            fn plugin_prototype<P: Plugin>() -> Box<FactoryPluginDescriptor<P>> {
                Box::new(FactoryPluginDescriptor::build()
                            .expect("cannot build factory plugin descriptor"))
            }

            static FACTORY: std::sync::LazyLock<Factory> =
                std::sync::LazyLock::new(||
                    Factory::new(vec![$(plugin_prototype::<$plug>(),)*])
                );

            /// SAFETY: CLAP requires this method to be thread-safe.
            /// The LazyLock guarding FACTORY is thread-safe and
            /// plugins_count() takes a shared reference to Factory.
            extern "C" fn get_plugin_count(_: *const clap_plugin_factory) -> u32 {
                FACTORY.plugins_count()
            }

            /// SAFETY: CLAP requires this method to be thread-safe.
            /// The LazyLock guarding FACTORY is thread-safe and
            /// descriptor() takes a shared reference to Factory.
            extern "C" fn get_plugin_descriptor(
                _: *const clap_plugin_factory,
                index: u32,
            ) -> *const clap_plugin_descriptor {
                FACTORY.descriptor(index).unwrap_or(std::ptr::null())
            }

            /// SAFETY: CLAP requires this method to be thread-safe.
            /// The LazyLock guarding FACTORY is thread-safe and
            /// boxed_clap_plugin() takes a shared reference to Factory.
            extern "C" fn create_plugin(
                _: *const clap_plugin_factory,
                host: *const clap_host,
                plugin_id: *const std::ffi::c_char,
            ) -> *const clap_plugin {
                if plugin_id.is_null() || host.is_null() {
                    return std::ptr::null();
                }
                // SAFETY: We just checked that host is non-null.
                let host = unsafe { FactoryHost::new(host) };
                // SAFETY: We checked if plug_id is non-null.
                // The host guarantees that this is a valid C string now.
                let plugin_id = unsafe { std::ffi::CStr::from_ptr(plugin_id) };
                FACTORY
                    .create_plugin(plugin_id, host)
                    .unwrap_or(std::ptr::null_mut())
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

            /// SAFETY: CLAP requires this method to be thread-safe.
            extern "C" fn get_factory(factory_id: *const std::ffi::c_char) -> *const std::ffi::c_void {
                if factory_id.is_null() {
                    return std::ptr::null();
                }
                // SAFETY: we cheched if factory_id is non-null.
                // The host guarantees that this is a valid C string.
                let id = unsafe { std::ffi::CStr::from_ptr(factory_id) };
                if id == CLAP_PLUGIN_FACTORY_ID {
                    &raw const CLAP_PLUGIN_FACTORY as *const _
                } else { std::ptr::null() }
            }

            #[allow(non_upper_case_globals)]
            #[unsafe(no_mangle)]
            #[used]
            // Make this symbor public, so that plugin's own tests can access it.
            pub static clap_entry: clap_plugin_entry = clap_plugin_entry {
                clap_version: CLAP_VERSION,
                init: Some(init),
                deinit: Some(deinit),
                get_factory: Some(get_factory),
            };
        }
    };
}
