use clap::{CLAP_VERSION, clap_plugin_descriptor};

enum Error {}

mod myplug {
    use crate::Plugin;

    #[derive(Default)]
    pub struct MyPlug {}

    impl Plugin for MyPlug {
        const ID: &'static str = "$$$ Pluggg $$$";
        const NAME: &'static str = "";
    }
}

trait Plugin: Default {
    const ID: &'static str;
    const NAME: &'static str;
}

trait PluginLike {}
impl<P: Plugin> PluginLike for P {}

mod plugin {
    use crate::{Plugin, PluginLike};
    use clap::{CLAP_VERSION, clap_plugin_descriptor};
    use std::ffi::CString;
    use std::ptr::null;
    use std::str::FromStr;

    pub struct Descriptor {
        pub(crate) raw: clap_plugin_descriptor,
        pub(crate) id: CString,
        name: CString,
        pub(crate) create: Box<dyn FnOnce() -> Box<dyn PluginLike>>
    }

    impl Descriptor {
        pub fn new<P: Plugin + 'static>() -> Self {
            let id = CString::from_str(P::ID).unwrap();
            let name = CString::from_str(P::NAME).unwrap();

            Self {
                raw: clap_plugin_descriptor {
                    clap_version: CLAP_VERSION,
                    id: id.as_c_str().as_ptr(),
                    name: name.as_c_str().as_ptr(),
                    vendor: c"".as_ptr(),
                    url: c"".as_ptr(),
                    manual_url: c"".as_ptr(),
                    support_url: c"".as_ptr(),
                    version: c"".as_ptr(),
                    description: c"".as_ptr(),
                    features: [c"".as_ptr(), null()].as_ptr(),
                },
                id,
                name,
                create: Box::new(|| Box::new(P::default()) )
            }
        }
    }
}

mod factory {
    use std::ffi::CStr;
    use crate::{Plugin, PluginLike};
    use crate::plugin::Descriptor;
    
   

    struct Factory<const N: usize> {
        desc: [Descriptor; N],
    }

    impl Factory<0> {
        fn create_plugin(&self, plugin_id: &str) -> Box<dyn PluginLike> {
            for desc in &self.desc {
                if desc.id.as_c_str().to_str().unwrap() == plugin_id {
                    
                }
            }
            todo!()
        }
    }

    static PLUGIN_FACTORY: Factory<0> = Factory { desc: [] };

    pub(crate) mod ffi {
        use crate::factory::PLUGIN_FACTORY;
        use clap::{clap_host, clap_plugin, clap_plugin_descriptor, clap_plugin_factory};
        use std::ffi::c_char;

        extern "C" fn get_plugin_count(_: *const clap_plugin_factory) -> u32 {
            PLUGIN_FACTORY.desc.len() as u32
        }

        extern "C" fn get_plugin_descriptor(
            _: *const clap_plugin_factory,
            id: u32,
        ) -> *const clap_plugin_descriptor {
            &raw const PLUGIN_FACTORY.desc[id as usize].raw
        }

        extern "C" fn create_plugin(
            _: *const clap_plugin_factory,
            host: *const clap_host,
            plugin_id: *const c_char,
        ) -> *const clap_plugin {
            todo!()
        }

        #[allow(non_upper_case_globals)]
        pub(crate) static plugin_factory: clap_plugin_factory = clap_plugin_factory {
            get_plugin_count: Some(get_plugin_count),
            get_plugin_descriptor: Some(get_plugin_descriptor),
            create_plugin: Some(create_plugin),
        };
    }
}

mod entry {
    mod ffi {
        use crate::factory::ffi::plugin_factory;
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

            &raw const plugin_factory as *const _
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
