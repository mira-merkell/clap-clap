#![feature(box_vec_non_null)]

trait Plugin: Default {
    const ID: &'static str;
    const NAME: &'static str;
    const VENDOR: &'static str;

    fn init(&mut self) -> Result<(), plugin::Error>;
    fn activate(
        &mut self,
        sample_rate: f64,
        min_frames: u32,
        max_frames: u32,
    ) -> Result<(), plugin::Error>;
    fn deactivate(&mut self);
    fn start_processing(&mut self) -> Result<(), plugin::Error>;
    fn stop_processing(&mut self);
    fn reset(&mut self);
}

mod plugin {
    use crate::Plugin;
    use clap::{clap_host, clap_plugin};

    use crate::plugin::desc::Descriptor;
    use std::marker::PhantomData;
    use std::ptr::{NonNull, null};

    pub enum Error {}

    pub(crate) mod desc {
        use crate::Plugin;
        use clap::{CLAP_VERSION, clap_plugin_descriptor};
        use std::ffi::{CStr, CString};
        use std::marker::PhantomData;
        use std::ptr::null;
        use std::str::FromStr;

        pub(crate) struct Descriptor<P> {
            id: CString,
            name: CString,
            vendor: CString,
            pub(crate) raw: clap_plugin_descriptor,
            _marker: PhantomData<P>,
        }

        impl<P: Plugin> Descriptor<P> {
            pub fn new(plugin: &P) -> Self {
                let id = CString::from_str(P::ID).unwrap();
                let name = CString::from_str(P::NAME).unwrap();
                let vendor = CString::from_str(P::VENDOR).unwrap();

                let raw = clap_plugin_descriptor {
                    clap_version: CLAP_VERSION,
                    id: id.as_c_str().as_ptr(),
                    name: name.as_c_str().as_ptr(),
                    vendor: vendor.as_c_str().as_ptr(),
                    url: null(),
                    manual_url: null(),
                    support_url: null(),
                    version: null(),
                    description: null(),
                    features: null(),
                };

                todo!();

                Self {
                    id,
                    name,
                    vendor,
                    raw,
                    _marker: PhantomData,
                }
            }
        }
    }

    struct ClapPluginData<P> {
        pub(crate) desc: Descriptor<P>,
        plugin: P,
        clap_host: Option<*const clap_host>,
    }

    impl<P: Plugin> ClapPluginData<P> {
        fn new(plugin: P, clap_host: Option<*const clap_host>) -> Self {
            Self {
                desc: Descriptor::new(&plugin),
                plugin,
                clap_host,
            }
        }
    }

    struct Wrap<P> {
        clap_plugin: NonNull<clap_plugin>,
        _marker: PhantomData<P>,
    }

    impl<P: Plugin> Wrap<P> {
        const fn new(clap_plugin: NonNull<clap_plugin>) -> Self {
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
                    desc: &raw const unsafe { &*data }.desc.raw,
                    plugin_data: data as *mut _,
                    init: Some(ffi::init::<P>),
                    destroy: Some(ffi::destroy::<P>),
                    activate: Some(ffi::activate::<P>),
                    deactivate: Some(ffi::deactivate::<P>),
                    start_processing: Some(ffi::start_processing::<P>),
                    stop_processing: Some(ffi::stop_processing::<P>),
                    reset: Some(ffi::reset::<P>),
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

    mod ffi {
        use crate::Plugin;
        use crate::plugin::Wrap;
        use clap::clap_plugin;
        use std::ptr::NonNull;

        const fn wrap_clap_ptr<P: Plugin>(plugin: *const clap_plugin) -> Wrap<P> {
            let plugin = plugin as *mut _;
            Wrap::<P>::new(NonNull::new(plugin).expect("plugin should be non-null"))
        }

        #[allow(warnings, unused)]
        pub(crate) extern "C" fn init<P: Plugin>(plugin: *const clap_plugin) -> bool {
            wrap_clap_ptr::<P>(plugin).plugin_mut().init().is_ok()
        }

        #[allow(warnings, unused)]
        pub(crate) extern "C" fn destroy<P: Plugin>(plugin: *const clap_plugin) {
            wrap_clap_ptr::<P>(plugin).unwrap_data();
        }

        #[allow(warnings, unused)]
        pub(crate) extern "C" fn activate<P: Plugin>(
            plugin: *const clap_plugin,
            sample_rate: f64,
            min_frames_count: u32,
            max_frames_count: u32,
        ) -> bool {
            wrap_clap_ptr::<P>(plugin)
                .plugin_mut()
                .activate(sample_rate, min_frames_count, max_frames_count)
                .is_ok()
        }

        #[allow(warnings, unused)]
        pub(crate) extern "C" fn deactivate<P: Plugin>(plugin: *const clap_plugin) {
            wrap_clap_ptr::<P>(plugin).plugin_mut().deactivate()
        }

        #[allow(warnings, unused)]
        pub(crate) extern "C" fn start_processing<P: Plugin>(plugin: *const clap_plugin) -> bool {
            wrap_clap_ptr::<P>(plugin)
                .plugin_mut()
                .start_processing()
                .is_ok()
        }

        #[allow(warnings, unused)]
        pub(crate) extern "C" fn stop_processing<P: Plugin>(plugin: *const clap_plugin) {
            wrap_clap_ptr::<P>(plugin).plugin_mut().stop_processing()
        }

        #[allow(warnings, unused)]
        pub(crate) extern "C" fn reset<P: Plugin>(plugin: *const clap_plugin) {
            wrap_clap_ptr::<P>(plugin).plugin_mut().reset()
        }
    }
}

mod factory {

    struct Factory;

    pub(crate) mod ffi {
        use clap::clap_plugin_factory;

        pub(crate) static PLUGIN_FACTORY: clap_plugin_factory = clap_plugin_factory {
            get_plugin_count: None,
            get_plugin_descriptor: None,
            create_plugin: None,
        };
    }
}

mod entry {
    mod ffi {
        use crate::factory::ffi::PLUGIN_FACTORY;
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

            &raw const PLUGIN_FACTORY as *const _
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
