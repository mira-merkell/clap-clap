mod audio_ports;
mod log;
mod params;

use std::{ffi::CStr, marker::PhantomData, mem::MaybeUninit, ptr::null};

use clap_clap::{
    ext::{Extensions, audio_ports::AudioPortInfo},
    factory::{Factory, FactoryHost, FactoryPluginPrototype},
    ffi::{
        CLAP_EXT_AUDIO_PORTS, CLAP_EXT_PARAMS, clap_audio_port_info, clap_plugin,
        clap_plugin_audio_ports, clap_plugin_params,
    },
    id::ClapId,
    plugin::Plugin,
    prelude::AudioPorts,
};

use crate::shims::host::SHIM_CLAP_HOST;

trait Test<P: Plugin> {
    fn test(self, bed: &mut TestBed<P>);
}

#[derive(Debug, Default, Copy, Clone)]
struct TestConfig<P> {
    _marker: PhantomData<P>,
}

impl<P: Plugin + Copy + 'static> TestConfig<P> {
    fn test(self, case: impl Test<P>) -> Self {
        TestBed::new(self).test(case);
        self
    }
}

#[derive(Debug)]
pub struct TestBed<P> {
    clap_plugin: *const clap_plugin,
    pub ext_audio_ports: Option<ExtAudioPorts>,
    pub ext_params: Option<ExtParams>,
    _config: TestConfig<P>,
}

impl<P: Plugin + 'static> TestBed<P> {
    fn new(config: TestConfig<P>) -> Self {
        let factory = Factory::new(vec![Box::new(
            FactoryPluginPrototype::<P>::build().unwrap(),
        )]);

        assert_eq!(factory.plugins_count(), 1);
        let plugin_desc = factory.descriptor(0).unwrap();
        assert!(!plugin_desc.is_null());
        let plugin_id = unsafe { (*plugin_desc).id };
        assert!(!plugin_id.is_null());

        let host = unsafe { FactoryHost::new_unchecked(SHIM_CLAP_HOST.as_ref()) };
        let clap_plugin = factory
            .create_plugin(unsafe { CStr::from_ptr(plugin_id) }, host)
            .unwrap();
        assert!(!clap_plugin.is_null());

        unsafe {
            Self {
                clap_plugin,
                ext_audio_ports: ExtAudioPorts::try_new_unchecked(clap_plugin),
                ext_params: ExtParams::try_new_unchecked(clap_plugin),
                _config: config,
            }
        }
    }

    fn test(&mut self, case: impl Test<P>) -> &mut Self {
        case.test(self);
        self
    }
}

impl<P> Drop for TestBed<P> {
    fn drop(&mut self) {
        assert!(!self.clap_plugin.is_null());
        let clap_plugin = unsafe { &*self.clap_plugin };
        unsafe { clap_plugin.destroy.unwrap()(clap_plugin) };

        self.clap_plugin = null();
    }
}

#[derive(Debug)]
pub struct ExtAudioPorts {
    clap_plugin: *const clap_plugin,
    clap_plugin_audio_ports: *const clap_plugin_audio_ports,
}

impl ExtAudioPorts {
    /// # Safety
    ///
    /// clap_plugin must be non-null.
    pub unsafe fn try_new_unchecked(clap_plugin: *const clap_plugin) -> Option<Self> {
        assert!(!clap_plugin.is_null());
        let extension = unsafe {
            (*clap_plugin).get_extension.unwrap()(clap_plugin, CLAP_EXT_AUDIO_PORTS.as_ptr())
        };

        unsafe { extension.as_ref() }.map(|ext| Self {
            clap_plugin,
            clap_plugin_audio_ports: (&raw const *ext).cast(),
        })
    }

    pub fn count(&self, is_input: bool) -> u32 {
        let audio_ports = unsafe { self.clap_plugin_audio_ports.as_ref() }.unwrap();
        unsafe { audio_ports.count.unwrap()(self.clap_plugin, is_input) }
    }

    pub fn get(&self, index: u32, is_input: bool) -> Option<AudioPortInfo> {
        let audio_ports = unsafe { self.clap_plugin_audio_ports.as_ref() }.unwrap();
        let mut info = MaybeUninit::<clap_audio_port_info>::uninit();

        if unsafe { audio_ports.get.unwrap()(self.clap_plugin, index, is_input, info.as_mut_ptr()) }
        {
            let info = unsafe { info.assume_init() };

            let name = unsafe { CStr::from_ptr(info.name.as_ptr()) };
            let port_type = (!info.port_type.is_null())
                .then(|| unsafe { CStr::from_ptr(info.port_type) })
                .and_then(|s| s.to_str().ok())?;

            Some(AudioPortInfo {
                id: ClapId::try_from(info.id).unwrap_or(ClapId::invalid_id()),
                name: name.to_str().ok().map(|s| s.to_owned()),
                flags: info.flags,
                channel_count: info.channel_count,
                port_type: port_type.try_into().ok(),
                in_place_pair: ClapId::try_from(info.in_place_pair).ok(),
            })
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct ExtParams {
    clap_plugin: *const clap_plugin,
    clap_plugin_params: *const clap_plugin_params,
}

impl ExtParams {
    /// # Safety
    ///
    /// clap_plugin must be non-null.
    pub unsafe fn try_new_unchecked(clap_plugin: *const clap_plugin) -> Option<Self> {
        assert!(!clap_plugin.is_null());
        let extension =
            unsafe { (*clap_plugin).get_extension.unwrap()(clap_plugin, CLAP_EXT_PARAMS.as_ptr()) };

        unsafe { extension.as_ref() }.map(|ext| Self {
            clap_plugin,
            clap_plugin_params: (&raw const *ext).cast(),
        })
    }

    pub fn count(&self) -> u32 {
        let params = unsafe { self.clap_plugin_params.as_ref() }.unwrap();
        unsafe { params.count.unwrap()(self.clap_plugin) }
    }
}
