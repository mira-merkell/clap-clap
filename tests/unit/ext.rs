mod audio_ports;
mod latency;
mod log;
mod note_ports;
mod params;

use std::{
    ffi::{CStr, CString},
    fmt::{Debug, Formatter},
    marker::PhantomData,
    mem::MaybeUninit,
    ptr::{null, null_mut},
};

use clap_clap::{
    ext::{
        audio_ports::AudioPortInfo,
        note_ports::NotePortInfo,
        params::{Error, ParamInfo},
    },
    factory::{Factory, FactoryHost, FactoryPluginPrototype},
    ffi::{
        CLAP_EXT_AUDIO_PORTS, CLAP_EXT_NOTE_PORTS, CLAP_EXT_PARAMS, clap_audio_port_info,
        clap_event_header, clap_input_events, clap_note_port_info, clap_output_events, clap_plugin,
        clap_plugin_audio_ports, clap_plugin_note_ports, clap_plugin_params,
    },
    id::ClapId,
    plugin::{ClapPlugin, Plugin},
};

use crate::shims::host::SHIM_CLAP_HOST;

trait Test<P: Plugin> {
    fn test(self, bed: &mut TestBed<P>);
}

#[derive(Default)]
struct TestConfig<P> {
    set_plug: Option<Box<dyn Fn(&mut P) + 'static>>,
    _marker: PhantomData<P>,
}

impl<P: Plugin> Debug for TestConfig<P> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TestConfig")
            .field("set_plug", &self.set_plug.is_some())
            .finish()
    }
}

impl<P: Plugin + Copy + 'static> TestConfig<P> {
    fn with_op<F: Fn(&mut P) + 'static>(op: F) -> Self {
        TestConfig {
            set_plug: Some(Box::new(op)),
            ..Default::default()
        }
    }

    fn test(self, case: impl Test<P>) {
        TestBed::new(self).test(case);
    }
}

#[derive(Debug)]
pub struct TestBed<P: Plugin> {
    clap_plugin: *const clap_plugin,
    pub ext_audio_ports: Option<ExtAudioPorts>,
    pub ext_latency: Option<ExtLatency>,
    pub ext_note_ports: Option<ExtNotePorts>,
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

        // Set plugin parameters specified by TestConfig
        if let Some(op) = config.set_plug.as_ref() {
            let mut plugin = unsafe { ClapPlugin::new_unchecked(clap_plugin) };
            op(unsafe { plugin.plugin() })
        }

        unsafe {
            Self {
                clap_plugin,
                ext_audio_ports: ExtAudioPorts::try_new_unchecked(clap_plugin),
                ext_note_ports: ExtNotePorts::try_new_unchecked(clap_plugin),
                ext_latency: ExtLatency::try_new_unchecked(clap_plugin),
                ext_params: ExtParams::try_new_unchecked(clap_plugin),
                _config: config,
            }
        }
    }

    pub const fn plugin(&self) -> ClapPlugin<P> {
        unsafe { ClapPlugin::new_unchecked(self.clap_plugin) }
    }

    pub fn activate(&self) -> bool {
        unsafe {
            self.clap_plugin.as_ref().unwrap().activate.unwrap()(self.clap_plugin, 48000.0, 1, 512)
        }
    }

    fn test(&mut self, case: impl Test<P>) -> &mut Self {
        case.test(self);
        self
    }
}

impl<P: Plugin> Drop for TestBed<P> {
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
                name: name.to_str().map(|s| s.to_owned()).unwrap(),
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
pub struct ExtNotePorts {
    clap_plugin: *const clap_plugin,
    clap_plugin_note_ports: *const clap_plugin_note_ports,
}

impl ExtNotePorts {
    /// # Safety
    ///
    /// clap_plugin must be non-null.
    pub unsafe fn try_new_unchecked(clap_plugin: *const clap_plugin) -> Option<Self> {
        assert!(!clap_plugin.is_null());
        let extension = unsafe {
            (*clap_plugin).get_extension.unwrap()(clap_plugin, CLAP_EXT_NOTE_PORTS.as_ptr())
        };

        unsafe { extension.as_ref() }.map(|ext| Self {
            clap_plugin,
            clap_plugin_note_ports: (&raw const *ext).cast(),
        })
    }

    pub fn count(&self, is_input: bool) -> u32 {
        let note_ports = unsafe { self.clap_plugin_note_ports.as_ref() }.unwrap();
        unsafe { note_ports.count.unwrap()(self.clap_plugin, is_input) }
    }

    pub fn get(&self, index: u32, is_input: bool) -> Option<NotePortInfo> {
        let note_ports = unsafe { self.clap_plugin_note_ports.as_ref() }.unwrap();
        let mut info = MaybeUninit::<clap_note_port_info>::uninit();

        if unsafe { note_ports.get.unwrap()(self.clap_plugin, index, is_input, info.as_mut_ptr()) }
        {
            let info = unsafe { info.assume_init() };

            let name = unsafe { CStr::from_ptr(info.name.as_ptr()) };

            Some(NotePortInfo {
                id: ClapId::try_from(info.id).unwrap_or(ClapId::invalid_id()),
                supported_dialects: info.supported_dialects,
                preferred_dialect: info.preferred_dialect,
                name: name.to_str().map(|s| s.to_owned()).unwrap(),
            })
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct ExtLatency {
    clap_plugin: *const clap_plugin,
    clap_plugin_latency: *const clap_plugin_latency,
}

impl ExtLatency {
    /// # Safety
    ///
    /// clap_plugin must be non-null.
    pub unsafe fn try_new_unchecked(clap_plugin: *const clap_plugin) -> Option<Self> {
        assert!(!clap_plugin.is_null());
        let extension = unsafe {
            (*clap_plugin).get_extension.unwrap()(clap_plugin, CLAP_EXT_LATENCY.as_ptr())
        };

        unsafe { extension.as_ref() }.map(|ext| Self {
            clap_plugin,
            clap_plugin_latency: (&raw const *ext).cast(),
        })
    }

    pub fn get(&self) -> u32 {
        let latency = unsafe { self.clap_plugin_latency.as_ref() }.unwrap();
        unsafe { latency.get.unwrap()(self.clap_plugin) }
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

    pub fn get_info(&self, param_index: u32) -> Option<ParamInfo> {
        let params = unsafe { self.clap_plugin_params.as_ref() }.unwrap();
        let mut info = MaybeUninit::uninit();
        if unsafe { params.get_info.unwrap()(self.clap_plugin, param_index, info.as_mut_ptr()) } {
            Some(unsafe { ParamInfo::try_from_unchecked(info.assume_init()) }.unwrap())
        } else {
            None
        }
    }

    pub fn get_value(&self, param_id: ClapId) -> Option<f64> {
        let params = unsafe { self.clap_plugin_params.as_ref() }.unwrap();

        let mut value = 0.0;
        unsafe { params.get_value.unwrap()(self.clap_plugin, param_id.into(), &raw mut value) }
            .then_some(value)
    }

    pub fn text_to_value(&self, param_id: ClapId, param_value_text: &str) -> Result<f64, Error> {
        let params = unsafe { self.clap_plugin_params.as_ref() }.unwrap();
        let text = CString::new(param_value_text).map_err(|_| Error::ConvertToValue)?;
        let mut out_value = 0.0;
        unsafe {
            params.text_to_value.unwrap()(
                self.clap_plugin,
                param_id.into(),
                text.as_ptr(),
                &raw mut out_value,
            )
        }
        .then_some(out_value)
        .ok_or(Error::ConvertToValue)
    }

    pub fn value_to_text(&self, param_id: ClapId, value: f64, buf: &mut [u8]) -> Result<(), Error> {
        let params = unsafe { self.clap_plugin_params.as_ref() }.unwrap();

        let mut out_buf = vec![1; buf.len() + 1];
        unsafe {
            params.value_to_text.unwrap()(
                self.clap_plugin,
                param_id.into(),
                value,
                out_buf.as_mut_ptr(),
                out_buf.len() as u32,
            )
        }
        .then_some(())
        .ok_or(Error::ConvertToText(value))?;

        for (d, s) in buf.iter_mut().zip(out_buf) {
            *d = s as u8;
        }

        Ok(())
    }

    pub fn flush(&self) {
        extern "C-unwind" fn size(_: *const clap_input_events) -> u32 {
            0
        }

        extern "C-unwind" fn get(_: *const clap_input_events, _: u32) -> *const clap_event_header {
            null()
        }

        extern "C-unwind" fn try_push(
            _: *const clap_output_events,
            _: *const clap_event_header,
        ) -> bool {
            false
        }

        let in_events = clap_input_events {
            ctx: null_mut(),
            size: Some(size),
            get: Some(get),
        };

        let out_events = clap_output_events {
            ctx: null_mut(),
            try_push: Some(try_push),
        };

        let params = unsafe { self.clap_plugin_params.as_ref() }.unwrap();
        unsafe { params.flush.unwrap()(self.clap_plugin, &in_events, &out_events) };
    }
}
