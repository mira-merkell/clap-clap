#![feature(box_vec_non_null)]

trait Plugin: Default {
    const ID: &'static str;
    const NAME: &'static str;
}

mod plugin {

    mod plugin_fn {
        use crate::Plugin;
        use crate::plugin::Wrap;
        use clap::clap_plugin;
        use std::ptr::NonNull;

        pub(crate) extern "C" fn destroy<P: Plugin>(plugin: *const clap_plugin) {
            let plugin = plugin as *mut _;
            Wrap::<P>::new(NonNull::new(plugin).expect("plugin should be non-null")).unwrap_data();
        }
    }

    use crate::Plugin;
    use clap::{clap_host, clap_plugin};
    use std::marker::PhantomData;
    use std::ptr::{NonNull, null};

    struct ClapPluginData<P> {
        clap_host: Option<*const clap_host>,
        plugin: P,
    }

    struct Wrap<P> {
        clap_plugin: NonNull<clap_plugin>,
        _marker: PhantomData<P>,
    }

    impl<P: Plugin> Wrap<P> {
        fn new(clap_plugin: NonNull<clap_plugin>) -> Self {
            Self {
                clap_plugin,
                _marker: PhantomData,
            }
        }

        fn into_inner(self) -> *const clap_plugin {
            self.clap_plugin.as_ptr()
        }

        fn plugin_data(&self) -> &ClapPluginData<P> {
            let data = unsafe { self.clap_plugin.as_ref() }.plugin_data;
            unsafe { &*(data as *const _) }
        }

        fn plugin(&self) -> &P {
            &self.plugin_data().plugin
        }

        fn plugin_data_mut(&mut self) -> &mut ClapPluginData<P> {
            let data = unsafe { self.clap_plugin.as_ref() }.plugin_data;
            unsafe { &mut *(data as *mut _) }
        }

        fn plugin_mut(&mut self) -> &mut P {
            &mut self.plugin_data_mut().plugin
        }

        fn wrap_data(data: ClapPluginData<P>) -> Self {
            let data = Box::into_raw(Box::new(data));

            Self {
                clap_plugin: Box::into_non_null(Box::new(clap_plugin {
                    desc: null(),
                    plugin_data: data as *mut _,
                    init: None,
                    destroy: Some(plugin_fn::destroy::<P>),
                    activate: None,
                    deactivate: None,
                    start_processing: None,
                    stop_processing: None,
                    reset: None,
                    process: None,
                    get_extension: None,
                    on_main_thread: None,
                })),
                _marker: PhantomData,
            }
        }

        fn unwrap_data(self) -> ClapPluginData<P> {
            let data = unsafe { self.clap_plugin.as_ref() }.plugin_data;
            let data: *mut ClapPluginData<P> = data as *mut _;
            let data = unsafe { Box::from_raw(data) };
            *data
        }
    }
}

mod entry {
    mod ffi {

        use clap::{CLAP_PLUGIN_FACTORY_ID, CLAP_VERSION, clap_plugin_entry};
        use std::{
            ffi::{CStr, c_char, c_void},
            ptr::null,
        };

        extern "C" fn init(_plugin_path: *const c_char) -> bool {
            true
        }

        extern "C" fn deinit() {}

        extern "C" fn get_factory(_factory_id: *const c_char) -> *const c_void {
            if _factory_id.is_null()
                || unsafe { CStr::from_ptr(_factory_id) } == CLAP_PLUGIN_FACTORY_ID
            {
                return null();
            }

            todo!()
        }

        #[allow(non_upper_case_globals)]
        #[allow(warnings, unused)]
        #[unsafe(no_mangle)]
        static clap_entry: clap_plugin_entry = clap_plugin_entry {
            clap_version: CLAP_VERSION,
            init: Some(init),
            deinit: Some(deinit),
            get_factory: Some(get_factory),
        };
    }
}
