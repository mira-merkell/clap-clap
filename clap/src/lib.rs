use std::fmt::Display;
use std::ptr::NonNull;

pub use clap_sys::CLAP_AUDIO_PORT_IS_MAIN;

#[derive(Debug, Clone)]
pub enum Error {}

impl Display for Error {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        Ok(())
    }
}

impl std::error::Error for Error {}

pub trait Plugin: Default + Sync + Send {
    const ID: &'static str;
    const NAME: &'static str = "";
    const VENDOR: &'static str = "";
    const URL: &'static str = "";
    const MANUAL_URL: &'static str = "";
    const SUPPORT_URL: &'static str = "";
    const VERSION: &'static str = "";
    const DESCRIPTION: &'static str = "";
    /// Arbitrary keywords separated by whitespace.
    /// For example: `"fx stereo distortion"`.
    const FEATURES: &'static str = "";

    type E: Extensions<Self>;

    fn activate(
        &mut self,
        _sample_rate: f64,
        _min_frames: u32,
        _max_frames: u32,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn deactivate(&mut self) {}

    fn start_processing(&mut self) -> Result<(), process::Error> {
        Ok(())
    }

    fn stop_processing(&mut self) {}

    fn process(&mut self, process: &mut Process) -> Result<process::Status, process::Error>;

    fn reset(&mut self) {}

    fn on_main_thread(&self) {}
}

pub trait Extensions<P: Plugin> {
    fn audio_ports() -> Option<impl AudioPorts<P>> {
        None::<()>
    }
}

pub mod ext {
    use crate::ext::audio_ports::AudioPortInfo;
    use crate::{Extensions, Plugin};

    impl<P: Plugin> Extensions<P> for () {}

    pub trait AudioPorts<P>
    where
        P: Plugin,
    {
        fn inputs(plugin: &P) -> usize;
        fn outputs(plugin: &P) -> usize;

        fn input_info(plugin: &P, index: usize) -> Option<AudioPortInfo>;
        fn output_info(plugin: &P, index: usize) -> Option<AudioPortInfo>;
    }

    pub mod audio_ports {
        use crate::ext::audio_ports::ffi::clap_plugin_audio_ports;
        use crate::ext::AudioPorts;
        use crate::Plugin;
        use clap_sys::{clap_plugin_audio_ports, CLAP_AUDIO_PORT_IS_MAIN};
        use std::marker::PhantomData;

        pub struct AudioPortInfo {
            pub id: usize,
            pub name: String,
            pub flags: Option<u32>,
            pub channel_count: u32,
            pub port_type: Option<AudioPortType>,
            pub in_place_pair: Option<usize>,
        }

        pub enum AudioPortType {
            Mono,
            Stereo,
            Surround,
            Ambisonic,
        }

        impl<P: Plugin> AudioPorts<P> for () {
            fn inputs(_plugin: &P) -> usize {
                0
            }

            fn outputs(_plugin: &P) -> usize {
                0
            }

            fn input_info(_plugin: &P, _index: usize) -> Option<AudioPortInfo> {
                None
            }

            fn output_info(_plugin: &P, _index: usize) -> Option<AudioPortInfo> {
                None
            }
        }

        /// Single static stereo port, in and aut.
        pub struct StereoPort;

        impl<P: Plugin> AudioPorts<P> for StereoPort {
            fn inputs(_: &P) -> usize {
                1
            }

            fn outputs(_: &P) -> usize {
                1
            }

            fn input_info(_: &P, index: usize) -> Option<AudioPortInfo> {
                (index == 0).then_some(AudioPortInfo {
                    id: 0,
                    name: "Main In".to_string(),
                    flags: Some(CLAP_AUDIO_PORT_IS_MAIN),
                    channel_count: 2,
                    port_type: Some(AudioPortType::Stereo),
                    in_place_pair: None,
                })
            }

            fn output_info(_: &P, index: usize) -> Option<AudioPortInfo> {
                (index == 0).then_some(AudioPortInfo {
                    id: 1,
                    name: "Main Out".to_string(),
                    flags: Some(CLAP_AUDIO_PORT_IS_MAIN),
                    channel_count: 2,
                    port_type: Some(AudioPortType::Stereo),
                    in_place_pair: None,
                })
            }
        }

        pub(crate) struct ClapPluginAudioPorts<P> {
            pub(crate) raw: clap_plugin_audio_ports,
            _marker: PhantomData<P>,
        }

        impl<P: Plugin> ClapPluginAudioPorts<P> {
            pub(crate) fn new<A: AudioPorts<P>>(_ports: A) -> Self {
                Self {
                    raw: clap_plugin_audio_ports::<A, P>(),
                    _marker: PhantomData,
                }
            }
        }

        mod ffi {
            use crate::ext::audio_ports::AudioPortType;
            use crate::ext::AudioPorts;
            use crate::{wrap_clap_plugin_from_host, Plugin};
            use clap_sys::{
                clap_audio_port_info, clap_plugin, clap_plugin_audio_ports, CLAP_INVALID_ID,
                CLAP_PORT_AMBISONIC, CLAP_PORT_MONO, CLAP_PORT_STEREO, CLAP_PORT_SURROUND,
            };
            use std::ffi::CString;
            use std::ptr::null;
            use std::str::FromStr;

            extern "C" fn count<A, P>(plugin: *const clap_plugin, is_input: bool) -> u32
            where
                P: Plugin,
                A: AudioPorts<P>,
            {
                let wrapper = unsafe { wrap_clap_plugin_from_host::<P>(plugin) };
                let plugin = wrapper.plugin();
                if is_input {
                    A::inputs(plugin) as u32
                } else {
                    A::outputs(plugin) as u32
                }
            }

            extern "C" fn get<A, P>(
                plugin: *const clap_plugin,
                index: u32,
                is_input: bool,
                info: *mut clap_audio_port_info,
            ) -> bool
            where
                P: Plugin,
                A: AudioPorts<P>,
            {
                let wrapper = unsafe { wrap_clap_plugin_from_host::<P>(plugin) };
                let plugin = wrapper.plugin();
                let port_info = if is_input {
                    A::input_info(plugin, index as usize)
                } else {
                    A::output_info(plugin, index as usize)
                };

                let Some(port_info) = port_info else {
                    return false;
                };

                let info = unsafe { &mut *info };
                info.id = port_info.id as u32;

                let name = CString::from_str(&port_info.name).unwrap();
                let name = name.as_bytes_with_nul();
                for i in 0..name.len() {
                    info.name[i] = name[i] as _;
                }

                info.flags = port_info.flags.unwrap_or(0);
                info.channel_count = port_info.channel_count;
                info.port_type = match port_info.port_type {
                    Some(AudioPortType::Mono) => CLAP_PORT_MONO.as_ptr(),
                    Some(AudioPortType::Stereo) => CLAP_PORT_STEREO.as_ptr(),
                    Some(AudioPortType::Surround) => CLAP_PORT_SURROUND.as_ptr(),
                    Some(AudioPortType::Ambisonic) => CLAP_PORT_AMBISONIC.as_ptr(),
                    None => null(),
                };
                info.in_place_pair = port_info
                    .in_place_pair
                    .map(|x| u32::try_from(x).expect("port index should fit into u32"))
                    .unwrap_or(CLAP_INVALID_ID);
                true
            }

            pub(crate) const fn clap_plugin_audio_ports<A, P>() -> clap_plugin_audio_ports
            where
                P: Plugin,
                A: AudioPorts<P>,
            {
                clap_plugin_audio_ports {
                    count: Some(count::<A, P>),
                    get: Some(get::<A, P>),
                }
            }
        }
    }
}

use crate::clap_plugin::ClapPluginWrapper;
use crate::ext::AudioPorts;
pub use process::Process;

pub mod process {
    use clap_sys::{
        clap_audio_buffer, clap_process, clap_process_status,
        CLAP_PROCESS_CONTINUE, CLAP_PROCESS_CONTINUE_IF_NOT_QUIET, CLAP_PROCESS_SLEEP, CLAP_PROCESS_TAIL,
    };
    use std::ptr::{slice_from_raw_parts, slice_from_raw_parts_mut};

    pub struct Process(pub(crate) clap_process);

    impl Process {
        pub fn steady_time(&self) -> i64 {
            self.0.steady_time
        }

        pub fn frames_count(&self) -> usize {
            self.0.frames_count as _
        }

        pub fn audio_inputs_count(&self) -> usize {
            self.0.audio_inputs_count as usize
        }

        pub unsafe fn audio_input_unchecked(&self, n: usize) -> Input<'_> {
            Input {
                buf: unsafe { &*self.0.audio_inputs.add(n) },
                frames_count: self.0.frames_count as usize,
            }
        }

        pub fn audio_input(&self, n: usize) -> Option<Input<'_>> {
            (n < self.0.audio_inputs_count as usize)
                .then_some(unsafe { self.audio_input_unchecked(n) })
        }

        pub fn audio_inputs_iter(&self) -> impl Iterator<Item = Input<'_>> {
            (0..self.audio_inputs_count()).map(|n| unsafe { self.audio_input_unchecked(n) })
        }

        pub fn audio_outputs_count(&self) -> usize {
            self.0.audio_outputs_count as usize
        }

        pub unsafe fn audio_output_unchecked(&mut self, n: usize) -> Output<'_> {
            Output {
                buf: unsafe { &mut *self.0.audio_outputs.add(n) },
                frames_count: self.0.frames_count as usize,
            }
        }

        pub fn audio_output(&mut self, n: usize) -> Option<Output<'_>> {
            (n < self.0.audio_outputs_count as usize)
                .then_some(unsafe { self.audio_output_unchecked(n) })
        }
    }

    pub struct Input<'a> {
        buf: &'a clap_audio_buffer,
        frames_count: usize,
    }

    impl<'a> Input<'a> {
        pub fn channel_count(&self) -> usize {
            self.buf.channel_count as _
        }

        pub fn latency(&self) -> usize {
            self.buf.latency as _
        }

        pub fn constant_mask(&self) -> u64 {
            self.buf.constant_mask
        }

        pub unsafe fn channel_unchecked(&self, n: usize) -> &[f32] {
            let samples = unsafe { *self.buf.data32.add(n) };
            unsafe { &*slice_from_raw_parts(samples, self.frames_count) }
        }

        pub fn channel(&self, n: usize) -> Option<&[f32]> {
            (n < self.buf.channel_count as usize).then_some(unsafe { self.channel_unchecked(n) })
        }

        pub fn channel_iter(&self) -> impl Iterator<Item = &[f32]> {
            (0..self.channel_count()).map(|n| unsafe { self.channel_unchecked(n) })
        }
    }

    pub struct Output<'a> {
        buf: &'a mut clap_audio_buffer,
        frames_count: usize,
    }

    impl<'a> Output<'a> {
        pub fn channel_count(&self) -> usize {
            self.buf.channel_count as _
        }

        pub fn latency(&self) -> usize {
            self.buf.latency as _
        }

        pub fn constant_mask(&self) -> u64 {
            self.buf.constant_mask
        }

        pub unsafe fn channel_unchecked(&mut self, n: usize) -> &mut [f32] {
            let samples = unsafe { *self.buf.data32.add(n) };
            unsafe { &mut *slice_from_raw_parts_mut(samples, self.frames_count) }
        }

        pub fn channel_mut(&mut self, n: usize) -> Option<&mut [f32]> {
            (n < self.buf.channel_count as usize).then_some(unsafe { self.channel_unchecked(n) })
        }
    }

    pub enum Status {
        Continue,
        ContinueIfNotQuiet,
        Tail,
        Sleep,
    }

    impl From<Status> for clap_process_status {
        fn from(value: Status) -> Self {
            use Status::*;
            match value {
                Continue => CLAP_PROCESS_CONTINUE,
                ContinueIfNotQuiet => CLAP_PROCESS_CONTINUE_IF_NOT_QUIET,
                Tail => CLAP_PROCESS_TAIL,
                Sleep => CLAP_PROCESS_SLEEP,
            }
        }
    }

    pub enum Error {
        Init,
        Process,
    }
}

mod host {
    use clap_sys::clap_host;
    use std::ptr::NonNull;

    pub struct ClapHost {
        _host: NonNull<clap_host>,
    }

    impl ClapHost {
        pub const fn new(host: NonNull<clap_host>) -> Self {
            Self { _host: host }
        }
    }
}

const unsafe fn wrap_clap_plugin_from_host<P: Plugin>(
    plugin: *const clap_sys::clap_plugin,
) -> ClapPluginWrapper<P> {
    let plugin = plugin as *mut _;
    unsafe { ClapPluginWrapper::<P>::new(NonNull::new_unchecked(plugin)) }
}

pub(crate) mod clap_plugin {
    use crate::host::ClapHost;
    use crate::{Extensions, Plugin};
    use clap_sys::{clap_plugin, clap_plugin_descriptor, CLAP_VERSION};

    use crate::ext::audio_ports::ClapPluginAudioPorts;
    use std::ffi::CStr;
    use std::{
        ffi::c_char, ffi::CString, marker::PhantomData, ptr::null, ptr::NonNull, str::FromStr,
    };

    #[allow(warnings, unused)]
    pub struct ClapPluginDescriptor {
        id: CString,
        name: CString,
        vendor: CString,
        url: CString,
        manual_url: CString,
        support_url: CString,
        version: CString,
        description: CString,
        features: Box<[CString]>,

        raw_features: Box<[*const c_char]>,
        raw_descriptor: clap_plugin_descriptor,
    }

    impl ClapPluginDescriptor {
        pub fn allocate<P: Plugin>() -> Self {
            let id = CString::from_str(P::ID).unwrap();
            let name = CString::from_str(P::NAME).unwrap();
            let vendor = CString::from_str(P::VENDOR).unwrap();
            let url = CString::from_str(P::URL).unwrap();
            let manual_url = CString::from_str(P::MANUAL_URL).unwrap();
            let support_url = CString::from_str(P::SUPPORT_URL).unwrap();
            let version = CString::from_str(P::VERSION).unwrap();
            let description = CString::from_str(P::DESCRIPTION).unwrap();

            let features: Vec<_> = String::from_str(P::FEATURES)
                .unwrap()
                .split_whitespace()
                .map(|s| CString::from_str(s).unwrap())
                .collect();
            let mut features_raw: Vec<_> = features.iter().map(|f| f.as_c_str().as_ptr()).collect();
            features_raw.push(null());
            let features_raw = features_raw.into_boxed_slice();

            let raw = clap_plugin_descriptor {
                clap_version: CLAP_VERSION,
                id: id.as_c_str().as_ptr(),
                name: name.as_c_str().as_ptr(),
                vendor: vendor.as_c_str().as_ptr(),
                url: url.as_c_str().as_ptr(),
                manual_url: manual_url.as_c_str().as_ptr(),
                support_url: support_url.as_c_str().as_ptr(),
                version: version.as_c_str().as_ptr(),
                description: description.as_c_str().as_ptr(),
                features: features_raw.as_ptr(),
            };

            Self {
                id,
                name,
                vendor,
                url,
                manual_url,
                support_url,
                version,
                description,
                features: features.into(),
                raw_features: features_raw,
                raw_descriptor: raw,
            }
        }

        pub(crate) fn plugin_id(&self) -> &CStr {
            &self.id
        }

        pub(crate) fn raw_descriptor(&self) -> &clap_plugin_descriptor {
            &self.raw_descriptor
        }
    }

    pub(crate) struct ClapPlugin<P> {
        desc: ClapPluginDescriptor,
        pub(crate) _host: ClapHost,
        plugin: P,
        pub(crate) audio_ports: Option<ClapPluginAudioPorts<P>>,
    }

    impl<P: Plugin> ClapPlugin<P> {
        pub(crate) fn generate(plugin: P, host: ClapHost) -> Self {
            let audio_ports = P::E::audio_ports().map(|ap| ClapPluginAudioPorts::new(ap));

            Self {
                desc: ClapPluginDescriptor::allocate::<P>(),
                plugin,
                _host: host,
                audio_ports,
            }
        }

        pub(crate) fn boxed_clap_plugin(self) -> Box<clap_plugin> {
            ffi::box_clap_plugin(self)
        }

        pub(crate) fn raw_descriptor(&self) -> &clap_plugin_descriptor {
            self.desc.raw_descriptor()
        }
    }

    pub(crate) struct ClapPluginWrapper<P> {
        clap_plugin: NonNull<clap_plugin>,
        _marker: PhantomData<P>,
    }

    impl<P: Plugin> ClapPluginWrapper<P> {
        pub(crate) const unsafe fn new(clap_plugin: NonNull<clap_plugin>) -> Self {
            Self {
                clap_plugin,
                _marker: PhantomData,
            }
        }

        pub(crate) fn plugin_data(&self) -> &ClapPlugin<P> {
            let data = unsafe { self.clap_plugin.as_ref() }.plugin_data;
            unsafe { &*(data as *const _) }
        }

        pub(crate) fn plugin(&self) -> &P {
            &self.plugin_data().plugin
        }

        pub(crate) fn plugin_data_mut(&mut self) -> &mut ClapPlugin<P> {
            let data = unsafe { self.clap_plugin.as_ref() }.plugin_data;
            unsafe { &mut *(data as *mut _) }
        }

        pub(crate) fn plugin_mut(&mut self) -> &mut P {
            &mut self.plugin_data_mut().plugin
        }

        unsafe fn take(self) -> ClapPlugin<P> {
            let clap_plugin = unsafe { Box::from_raw(self.clap_plugin.as_ptr()) };
            let data: *mut ClapPlugin<P> = clap_plugin.plugin_data as *mut _;

            *unsafe { Box::from_raw(data) }
        }
    }

    mod ffi {
        use crate::clap_plugin::ClapPlugin;
        use crate::{wrap_clap_plugin_from_host, Plugin, Process};
        use clap_sys::{clap_plugin, CLAP_EXT_AUDIO_PORTS, CLAP_PROCESS_ERROR};
        use clap_sys::{clap_process, clap_process_status};
        use std::ffi::{c_char, c_void, CStr};
        use std::ptr::null;

        #[allow(warnings, unused)]
        extern "C" fn init<P: Plugin>(plugin: *const clap_plugin) -> bool {
            true
        }

        extern "C" fn destroy<P: Plugin>(plugin: *const clap_plugin) {
            unsafe { wrap_clap_plugin_from_host::<P>(plugin).take() };
        }

        extern "C" fn activate<P: Plugin>(
            plugin: *const clap_plugin,
            sample_rate: f64,
            min_frames_count: u32,
            max_frames_count: u32,
        ) -> bool {
            unsafe { wrap_clap_plugin_from_host::<P>(plugin) }
                .plugin_mut()
                .activate(sample_rate, min_frames_count, max_frames_count)
                .is_ok()
        }

        extern "C" fn deactivate<P: Plugin>(plugin: *const clap_plugin) {
            unsafe { wrap_clap_plugin_from_host::<P>(plugin) }
                .plugin_mut()
                .deactivate()
        }

        extern "C" fn start_processing<P: Plugin>(plugin: *const clap_plugin) -> bool {
            unsafe { wrap_clap_plugin_from_host::<P>(plugin) }
                .plugin_mut()
                .start_processing()
                .is_ok()
        }

        extern "C" fn stop_processing<P: Plugin>(plugin: *const clap_plugin) {
            unsafe { wrap_clap_plugin_from_host::<P>(plugin) }
                .plugin_mut()
                .stop_processing()
        }

        extern "C" fn reset<P: Plugin>(plugin: *const clap_plugin) {
            unsafe { wrap_clap_plugin_from_host::<P>(plugin) }
                .plugin_mut()
                .reset()
        }

        #[allow(warnings, unused)]
        extern "C" fn process<P: Plugin>(
            plugin: *const clap_plugin,
            process: *const clap_process,
        ) -> clap_process_status {
            if process.is_null() {
                return CLAP_PROCESS_ERROR;
            }

            let mut process = Process(unsafe { *process });
            unsafe { wrap_clap_plugin_from_host::<P>(plugin) }
                .plugin_mut()
                .process(&mut process)
                .map(Into::into)
                .unwrap_or(CLAP_PROCESS_ERROR)
        }

        #[allow(warnings, unused)]
        extern "C" fn get_extension<P: Plugin>(
            plugin: *const clap_plugin,
            id: *const c_char,
        ) -> *const c_void {
            let plugin = unsafe { wrap_clap_plugin_from_host::<P>(plugin) };
            if id.is_null() {
                return null();
            }
            let id = unsafe { CStr::from_ptr(id) };
            if id == CLAP_EXT_AUDIO_PORTS && plugin.plugin_data().audio_ports.is_some() {
                if let Some(audio_ports) = &plugin.plugin_data().audio_ports {
                    return &raw const audio_ports.raw as *const _;
                }
            }

            null()
        }

        extern "C" fn on_main_thread<P: Plugin>(plugin: *const clap_plugin) {
            unsafe { wrap_clap_plugin_from_host::<P>(plugin) }
                .plugin()
                .on_main_thread()
        }

        pub(crate) fn box_clap_plugin<P: Plugin>(data: ClapPlugin<P>) -> Box<clap_plugin> {
            let data = Box::new(data);
            let desc = &raw const *data.raw_descriptor();
            let data = Box::into_raw(data);

            Box::new(clap_plugin {
                desc,
                plugin_data: data as *mut _,
                init: Some(init::<P>),
                destroy: Some(destroy::<P>),
                activate: Some(activate::<P>),
                deactivate: Some(deactivate::<P>),
                start_processing: Some(start_processing::<P>),
                stop_processing: Some(stop_processing::<P>),
                reset: Some(reset::<P>),
                process: Some(process::<P>),
                get_extension: Some(get_extension::<P>),
                on_main_thread: Some(on_main_thread::<P>),
            })
        }
    }
}

pub mod factory {
    use crate::clap_plugin::{ClapPlugin, ClapPluginDescriptor};
    use crate::host::ClapHost;
    use crate::Plugin;
    use clap_sys::{clap_plugin, clap_plugin_descriptor};
    use std::collections::HashMap;
    use std::ffi::{CStr, CString};
    use std::marker::PhantomData;

    pub struct PluginPrototype<P> {
        descriptor: ClapPluginDescriptor,
        _marker: PhantomData<P>,
    }

    impl<P: Plugin> PluginPrototype<P> {
        pub fn allocate() -> Self {
            Self {
                descriptor: ClapPluginDescriptor::allocate::<P>(),
                _marker: PhantomData,
            }
        }
    }

    pub trait FactoryPlugin {
        fn plugin_id(&self) -> &CStr;
        fn clap_plugin_descriptor(&self) -> &clap_plugin_descriptor;
        fn boxed_clap_plugin(&self, host: ClapHost) -> Box<clap_plugin>;
    }

    impl<P: Plugin> FactoryPlugin for PluginPrototype<P> {
        fn plugin_id(&self) -> &CStr {
            self.descriptor.plugin_id()
        }

        fn clap_plugin_descriptor(&self) -> &clap_plugin_descriptor {
            self.descriptor.raw_descriptor()
        }

        fn boxed_clap_plugin(&self, host: ClapHost) -> Box<clap_plugin> {
            ClapPlugin::generate(P::default(), host).boxed_clap_plugin()
        }
    }

    pub struct Factory {
        id_map: HashMap<CString, usize>,
        plugins: Vec<Box<dyn FactoryPlugin>>,
    }

    impl Factory {
        pub fn new(plugins: Vec<Box<dyn FactoryPlugin>>) -> Self {
            Self {
                id_map: plugins
                    .iter()
                    .enumerate()
                    .map(|(i, p)| (CString::from(p.plugin_id()), i))
                    .collect(),
                plugins,
            }
        }

        pub fn plugins_count(&self) -> usize {
            self.plugins.len()
        }

        pub fn descriptor(&self, index: usize) -> &clap_plugin_descriptor {
            self.plugins[index].clap_plugin_descriptor()
        }

        pub fn boxed_clap_plugin(
            &self,
            plugin_id: &CStr,
            host: ClapHost,
        ) -> Option<Box<clap_plugin>> {
            self.id_map
                .get(plugin_id)
                .map(|&i| self.plugins[i].boxed_clap_plugin(host))
        }
    }

    unsafe impl Send for Factory {}
    unsafe impl Sync for Factory {}
}

pub mod entry {
    use crate::Plugin;

    pub use clap_sys::{
        clap_host, clap_plugin, clap_plugin_descriptor, clap_plugin_entry, clap_plugin_factory,
        CLAP_PLUGIN_FACTORY_ID, CLAP_VERSION,
    };
    pub use std::ptr::NonNull;

    pub use crate::factory::{Factory, PluginPrototype};

    #[macro_export]
    macro_rules! entry {
        ($($plug:ty),*) => {
            mod _clap_entry {

                use $crate::entry::*;

                use super::*; // Access the types supplied as macro arguments.

                static FACTORY: std::sync::OnceLock<Factory> = std::sync::OnceLock::new();

                fn factory_init_once<'a>() -> &'a Factory {
                    FACTORY.get_or_init(|| Factory::new(vec![$(plugin_prototype::<$plug>(),)*]))
                }

                extern "C" fn get_plugin_count(_: *const clap_plugin_factory) -> u32 {
                    factory_init_once().plugins_count() as u32
                }

                extern "C" fn get_plugin_descriptor(
                    _: *const clap_plugin_factory,
                    index: u32,
                ) -> *const clap_plugin_descriptor {
                    factory_init_once().descriptor(index as usize)
                }

                extern "C" fn create_plugin(
                    _: *const clap_plugin_factory,
                    host: *const clap_host,
                    plugin_id: *const std::ffi::c_char,
                ) -> *const clap_plugin {
                    if plugin_id.is_null() || host.is_null() {
                        return std::ptr::null();
                    }
                    let host = ClapHost::new(unsafe{ NonNull::new_unchecked(host as *mut _)});
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

                extern "C" fn init(_plugin_path: *const std::ffi::c_char) -> bool {
                    true
                }

                extern "C" fn deinit() {}

                extern "C" fn get_factory(factory_id: *const std::ffi::c_char) -> *const std::ffi::c_void {
                    if factory_id.is_null() {
                        return std::ptr::null();
                    }
                    let id = unsafe { std::ffi::CStr::from_ptr(factory_id) };
                    if id != CLAP_PLUGIN_FACTORY_ID {
                        return std::ptr::null();
                    }

                    &raw const CLAP_PLUGIN_FACTORY as *const _
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
        };
    }

    pub use crate::host::ClapHost;

    pub fn plugin_prototype<P: Plugin>() -> Box<PluginPrototype<P>> {
        Box::new(PluginPrototype::allocate())
    }
}
