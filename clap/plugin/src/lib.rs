#![feature(box_vec_non_null)]

use clap::Plugin;

mod plugin {
    use crate::plugin::desc::Descriptor;
    use clap::Plugin;
    use clap_sys::{clap_host, clap_plugin};
    use std::{marker::PhantomData, ptr::NonNull};

    pub enum Error {}

    pub(crate) mod desc {

        use clap::Plugin;
        use clap_sys::{CLAP_VERSION, clap_plugin_descriptor};
        use std::{
            ffi::{CStr, CString},
            marker::PhantomData,
            ptr::null,
            str::FromStr,
        };

        pub(crate) struct Descriptor<P> {
            id: CString,
            name: CString,
            vendor: CString,
            pub(crate) raw: clap_plugin_descriptor,
            _marker: PhantomData<P>,
        }

        impl<P: Plugin> Descriptor<P> {
            pub fn new() -> Self {
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

    pub(crate) struct ClapPluginData<P> {
        pub(crate) desc: Descriptor<P>,
        plugin: P,
        clap_host: Option<*const clap_host>,
    }

    impl<P: Plugin> ClapPluginData<P> {
        pub(crate) fn new(plugin: P, clap_host: Option<*const clap_host>) -> Self {
            Self {
                desc: Descriptor::new(),
                plugin,
                clap_host,
            }
        }
    }

    pub(crate) struct Wrap<P> {
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

        pub(crate) fn into_inner(self) -> *const clap_plugin {
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

        pub(crate) fn wrap_data(data: ClapPluginData<P>) -> Self {
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
        use crate::{Plugin, plugin::Wrap};
        use clap_sys::clap_plugin;
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
    use crate::Plugin;
    use crate::plugin::desc::Descriptor;
    use crate::plugin::{ClapPluginData, Wrap};
    use clap_sys::{clap_host, clap_plugin, clap_plugin_descriptor};
    use std::sync::OnceLock;

    trait FactoryPluginDescriptor {
        fn descriptor(&self) -> *const clap_plugin_descriptor;
        fn create(&self, host: Option<*const clap_host>) -> *const clap_plugin;
    }

    impl<P: Plugin> FactoryPluginDescriptor for Descriptor<P> {
        fn descriptor(&self) -> *const clap_plugin_descriptor {
            &raw const self.raw
        }

        fn create(&self, host: Option<*const clap_host>) -> *const clap_plugin {
            let data = ClapPluginData::new(P::default(), host);
            Wrap::wrap_data(data).into_inner()
        }
    }

    fn plug_desc<P: Plugin>() -> Box<Descriptor<P>> {
        Box::new(Descriptor::new())
    }

    struct Factory {
        plugins: Vec<Box<dyn FactoryPluginDescriptor>>,
    }

    fn factory_init() -> Factory {
        Factory {
            plugins: vec![plug_desc::<myplug::MyPlug>()],
        }
    }

    unsafe impl Send for Factory {}
    unsafe impl Sync for Factory {}

    static FACTORY: OnceLock<Factory> = OnceLock::new();

    pub(crate) mod ffi {
        use crate::factory::{FACTORY, factory_init};
        use clap_sys::{clap_host, clap_plugin, clap_plugin_descriptor, clap_plugin_factory};
        use std::ffi::{CStr, c_char};
        use std::ptr::null;

        extern "C" fn get_plugin_count(_: *const clap_plugin_factory) -> u32 {
            FACTORY.get_or_init(factory_init).plugins.len() as u32
        }

        extern "C" fn get_plugin_descriptor(
            _: *const clap_plugin_factory,
            index: u32,
        ) -> *const clap_plugin_descriptor {
            FACTORY.get_or_init(factory_init).plugins[index as usize].descriptor()
        }

        extern "C" fn create_plugin(
            _: *const clap_plugin_factory,
            host: *const clap_host,
            plugin_id: *const c_char,
        ) -> *const clap_plugin {
            if !plugin_id.is_null() {
                for plugin in &FACTORY.get_or_init(factory_init).plugins {
                    let id = unsafe { CStr::from_ptr((*plugin.descriptor()).id) };
                    if unsafe { CStr::from_ptr(plugin_id) } == id {
                        let host = (!host.is_null()).then_some(host);
                        return plugin.create(host);
                    }
                }
            }

            null()
        }

        pub(crate) static PLUGIN_FACTORY: clap_plugin_factory = clap_plugin_factory {
            get_plugin_count: Some(get_plugin_count),
            get_plugin_descriptor: Some(get_plugin_descriptor),
            create_plugin: Some(create_plugin),
        };
    }
}

mod entry {
    mod ffi {
        use crate::factory::ffi::PLUGIN_FACTORY;
        use clap_sys::{CLAP_PLUGIN_FACTORY_ID, CLAP_VERSION, clap_plugin_entry};
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
