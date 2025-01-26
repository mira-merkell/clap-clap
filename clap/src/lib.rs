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

    type Extensions: Extensions<Self>;

    fn activate(
        &mut self,
        _sample_rate: f64,
        _min_frames: usize,
        _max_frames: usize,
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

pub mod extensions {
    use crate::extensions::audio_ports::AudioPortInfo;
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
        use crate::Plugin;
        use crate::extensions::AudioPorts;
        use crate::extensions::audio_ports::ffi::clap_plugin_audio_ports;
        use clap_sys::{CLAP_AUDIO_PORT_IS_MAIN, clap_plugin_audio_ports};
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
        pub struct StereoPorts;

        impl<P: Plugin> AudioPorts<P> for StereoPorts {
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
                    flags: Some(CLAP_AUDIO_PORT_IS_MAIN.try_into().unwrap()),
                    channel_count: 2,
                    port_type: Some(AudioPortType::Stereo),
                    in_place_pair: None,
                })
            }

            fn output_info(_: &P, index: usize) -> Option<AudioPortInfo> {
                (index == 0).then_some(AudioPortInfo {
                    id: 1,
                    name: "Main Out".to_string(),
                    flags: Some(CLAP_AUDIO_PORT_IS_MAIN.try_into().unwrap()),
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
            use crate::extensions::AudioPorts;
            use crate::extensions::audio_ports::AudioPortType;
            use crate::{Plugin, wrap_clap_plugin_from_host};
            use clap_sys::{
                CLAP_INVALID_ID, CLAP_PORT_AMBISONIC, CLAP_PORT_MONO, CLAP_PORT_STEREO,
                CLAP_PORT_SURROUND, clap_audio_port_info, clap_plugin, clap_plugin_audio_ports,
            };
            use std::ptr::null;

            extern "C" fn count<A, P>(plugin: *const clap_plugin, is_input: bool) -> u32
            where
                P: Plugin,
                A: AudioPorts<P>,
            {
                let wrapper = unsafe { wrap_clap_plugin_from_host::<P>(plugin) };
                let plugin = &wrapper.clap_plugin().plugin;
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
                let plugin = &wrapper.clap_plugin().plugin;
                let index = usize::try_from(index).expect("index must fit into usize");
                let Some(query) = (if is_input {
                    A::input_info(plugin, index)
                } else {
                    A::output_info(plugin, index)
                }) else {
                    return false;
                };
                let info = unsafe { &mut *info };
                info.id = query.id.try_into().expect("id must fit into u32");

                let n = query.name.len().min(info.name.len());
                unsafe {
                    std::ptr::copy_nonoverlapping(
                        query.name.as_ptr(),
                        info.name.as_mut_ptr() as *mut _,
                        n,
                    )
                }
                info.name[n] = b'\0' as _;

                info.flags = query.flags.unwrap_or(0);
                info.channel_count = query.channel_count;
                info.port_type = match query.port_type {
                    Some(AudioPortType::Mono) => CLAP_PORT_MONO.as_ptr(),
                    Some(AudioPortType::Stereo) => CLAP_PORT_STEREO.as_ptr(),
                    Some(AudioPortType::Surround) => CLAP_PORT_SURROUND.as_ptr(),
                    Some(AudioPortType::Ambisonic) => CLAP_PORT_AMBISONIC.as_ptr(),
                    None => null(),
                };
                info.in_place_pair = query
                    .in_place_pair
                    .map(|x| x.try_into().expect("port index should fit into u32"))
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

use crate::extensions::AudioPorts;
use crate::plugin::ClapPluginWrapper;
pub use process::Process;

pub mod process {
    use clap_sys::{
        CLAP_PROCESS_CONTINUE, CLAP_PROCESS_CONTINUE_IF_NOT_QUIET, CLAP_PROCESS_SLEEP,
        CLAP_PROCESS_TAIL, clap_audio_buffer, clap_process, clap_process_status,
    };
    use std::ptr::{slice_from_raw_parts, slice_from_raw_parts_mut};

    pub struct Process(pub(crate) clap_process);

    impl Process {
        pub fn steady_time(&self) -> i64 {
            self.0.steady_time
        }

        pub fn frames_count(&self) -> usize {
            self.0
                .frames_count
                .try_into()
                .expect("frame_count must fit into usize")
        }

        pub fn audio_inputs_count(&self) -> usize {
            self.0
                .audio_inputs_count
                .try_into()
                .expect("audio_inputs_count must fit into usize")
        }

        pub unsafe fn audio_input_unchecked(&self, n: usize) -> Input<'_> {
            Input {
                buf: unsafe { &*self.0.audio_inputs.add(n) },
                frames_count: self.frames_count(),
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
            self.0
                .audio_outputs_count
                .try_into()
                .expect("audio_outputs_count must fit into usize")
        }

        pub unsafe fn audio_output_unchecked(&mut self, n: usize) -> Output<'_> {
            Output {
                buf: unsafe { &mut *self.0.audio_outputs.add(n) },
                frames_count: self.frames_count(),
            }
        }

        pub fn audio_output(&mut self, n: usize) -> Option<Output<'_>> {
            (n < self.0.audio_outputs_count as usize)
                .then_some(unsafe { self.audio_output_unchecked(n) })
        }

        pub unsafe fn link_ports_unchecked(&mut self, port_in: usize, port_out: usize) -> Link<'_> {
            let channel_count = unsafe { self.audio_input_unchecked(port_in).channel_count() };
            let port_in = unsafe { &*self.0.audio_inputs.add(port_in) };
            let port_out = unsafe { &mut *self.0.audio_outputs.add(port_out) };

            unsafe { Link::new_unchecked(port_in, port_out, channel_count, self.frames_count()) }
        }

        pub fn link_ports(&mut self, port_in: usize, port_out: usize) -> Result<Link<'_>, Error> {
            let port_in = ((port_in as u32) < self.0.audio_inputs_count)
                .then_some(unsafe { &*self.0.audio_inputs.add(port_in) })
                .ok_or(Error::Link)?;
            let port_out = ((port_out as u32) < self.0.audio_outputs_count)
                .then_some(unsafe { &mut *self.0.audio_outputs.add(port_out) })
                .ok_or(Error::Link)?;

            Link::new(port_in, port_out, self.frames_count()).ok_or(Error::Link)
        }
    }

    pub struct Input<'a> {
        buf: &'a clap_audio_buffer,
        frames_count: usize,
    }

    impl Input<'_> {
        pub fn channel_count(&self) -> usize {
            self.buf
                .channel_count
                .try_into()
                .expect("channel count must fit into usize")
        }

        pub fn latency(&self) -> usize {
            self.buf
                .latency
                .try_into()
                .expect("latency must fit into usize")
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

    impl Output<'_> {
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

    /// Up to 8 channel ports.
    pub struct Link<'a> {
        port_in: &'a clap_audio_buffer,
        port_out: &'a mut clap_audio_buffer,
        channel_count: usize,
        frames_count: usize,
        frame: [f32; 8],
    }

    impl<'a> Link<'a> {
        unsafe fn new_unchecked(
            port_in: &'a clap_audio_buffer,
            port_out: &'a mut clap_audio_buffer,
            channel_count: usize,
            frames_count: usize,
        ) -> Self {
            Self {
                port_in,
                port_out,
                channel_count,
                frames_count,
                frame: [0.0; 8],
            }
        }

        fn new(
            port_in: &'a clap_audio_buffer,
            port_out: &'a mut clap_audio_buffer,
            frames_count: usize,
        ) -> Option<Self> {
            let channel_count = usize::try_from(port_in.channel_count).ok()?;
            (channel_count <= 8 && port_in.channel_count == port_out.channel_count).then_some(
                unsafe { Self::new_unchecked(port_in, port_out, channel_count, frames_count) },
            )
        }

        pub fn with_op(&mut self, op: impl FnMut(&mut [f32])) {
            let mut op = op;

            for i in 0..self.frames_count {
                for k in 0..self.channel_count {
                    let sample = unsafe { *(*self.port_in.data32.add(k)).add(i) };
                    self.frame[k] = sample;
                }

                op(&mut self.frame[0..self.channel_count]);

                for k in 0..self.channel_count {
                    let sample = unsafe { &mut *(*self.port_out.data32.add(k)).add(i) };
                    *sample = self.frame[k];
                }
            }
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
        Link,
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

pub mod plugin {
    use crate::host::ClapHost;
    use crate::{Extensions, Plugin};
    use clap_sys::{CLAP_VERSION, clap_plugin, clap_plugin_descriptor};

    use crate::extensions::audio_ports::ClapPluginAudioPorts;
    use std::{
        ffi::CString, ffi::c_char, marker::PhantomData, ptr::NonNull, ptr::null, str::FromStr,
    };

    #[allow(warnings, unused)]
    pub struct PluginDescriptor<P> {
        pub(crate) id: CString,
        name: CString,
        vendor: CString,
        url: CString,
        manual_url: CString,
        support_url: CString,
        version: CString,
        description: CString,
        features: Box<[CString]>,

        raw_features: Box<[*const c_char]>,
        pub(crate) raw_descriptor: clap_plugin_descriptor,
        _marker: PhantomData<P>,
    }

    impl<P: Plugin> PluginDescriptor<P> {
        pub fn allocate() -> Self {
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
                _marker: PhantomData,
            }
        }
    }

    pub(crate) struct ClapPlugin<P> {
        pub(crate) descriptor: PluginDescriptor<P>,
        pub(crate) _host: ClapHost,
        pub(crate) plugin: P,
        pub(crate) audio_ports: Option<ClapPluginAudioPorts<P>>,
    }

    impl<P: Plugin> ClapPlugin<P> {
        pub(crate) fn generate(plugin: P, host: ClapHost) -> Self {
            let audio_ports = P::Extensions::audio_ports().map(|ap| ClapPluginAudioPorts::new(ap));

            Self {
                descriptor: PluginDescriptor::allocate(),
                plugin,
                _host: host,
                audio_ports,
            }
        }

        pub(crate) fn boxed_clap_plugin(self) -> Box<clap_plugin> {
            ffi::box_clap_plugin(self)
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

        pub(crate) fn clap_plugin(&self) -> &ClapPlugin<P> {
            let data = unsafe { self.clap_plugin.as_ref() }.plugin_data;
            unsafe { &*(data as *const _) }
        }

        pub(crate) fn clap_plugin_mut(&mut self) -> &mut ClapPlugin<P> {
            let data = unsafe { self.clap_plugin.as_ref() }.plugin_data;
            unsafe { &mut *(data as *mut _) }
        }

        unsafe fn take(self) -> ClapPlugin<P> {
            let clap_plugin = unsafe { Box::from_raw(self.clap_plugin.as_ptr()) };
            let data: *mut ClapPlugin<P> = clap_plugin.plugin_data as *mut _;

            *unsafe { Box::from_raw(data) }
        }
    }

    mod ffi {
        use crate::plugin::ClapPlugin;
        use crate::{Plugin, Process, wrap_clap_plugin_from_host};
        use clap_sys::{CLAP_EXT_AUDIO_PORTS, CLAP_PROCESS_ERROR, clap_plugin};
        use clap_sys::{clap_process, clap_process_status};
        use std::ffi::{CStr, c_char, c_void};
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
                .clap_plugin_mut()
                .plugin
                .activate(
                    sample_rate,
                    min_frames_count as usize,
                    max_frames_count as usize,
                )
                .is_ok()
        }

        extern "C" fn deactivate<P: Plugin>(plugin: *const clap_plugin) {
            unsafe { wrap_clap_plugin_from_host::<P>(plugin) }
                .clap_plugin_mut()
                .plugin
                .deactivate()
        }

        extern "C" fn start_processing<P: Plugin>(plugin: *const clap_plugin) -> bool {
            unsafe { wrap_clap_plugin_from_host::<P>(plugin) }
                .clap_plugin_mut()
                .plugin
                .start_processing()
                .is_ok()
        }

        extern "C" fn stop_processing<P: Plugin>(plugin: *const clap_plugin) {
            unsafe { wrap_clap_plugin_from_host::<P>(plugin) }
                .clap_plugin_mut()
                .plugin
                .stop_processing()
        }

        extern "C" fn reset<P: Plugin>(plugin: *const clap_plugin) {
            unsafe { wrap_clap_plugin_from_host::<P>(plugin) }
                .clap_plugin_mut()
                .plugin
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
                .clap_plugin_mut()
                .plugin
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
            if id == CLAP_EXT_AUDIO_PORTS && plugin.clap_plugin().audio_ports.is_some() {
                if let Some(audio_ports) = &plugin.clap_plugin().audio_ports {
                    return &raw const audio_ports.raw as *const _;
                }
            }

            null()
        }

        extern "C" fn on_main_thread<P: Plugin>(plugin: *const clap_plugin) {
            unsafe { wrap_clap_plugin_from_host::<P>(plugin) }
                .clap_plugin()
                .plugin
                .on_main_thread()
        }

        pub(crate) fn box_clap_plugin<P: Plugin>(data: ClapPlugin<P>) -> Box<clap_plugin> {
            let data = Box::new(data);
            let desc = &raw const data.descriptor.raw_descriptor;
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
    use crate::Plugin;
    use crate::host::ClapHost;
    use crate::plugin::{ClapPlugin, PluginDescriptor};
    use clap_sys::{clap_plugin, clap_plugin_descriptor};
    use std::collections::HashMap;
    use std::ffi::{CStr, CString};

    pub trait FactoryPlugin {
        fn plugin_id(&self) -> &CStr;
        fn clap_plugin_descriptor(&self) -> &clap_plugin_descriptor;
        fn boxed_clap_plugin(&self, host: ClapHost) -> Box<clap_plugin>;
    }

    impl<P: Plugin> FactoryPlugin for PluginDescriptor<P> {
        fn plugin_id(&self) -> &CStr {
            &self.id
        }

        fn clap_plugin_descriptor(&self) -> &clap_plugin_descriptor {
            &self.raw_descriptor
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

        pub fn plugins_count(&self) -> u32 {
            self.plugins
                .len()
                .try_into()
                .expect("plugins_count should fit into u32")
        }

        pub fn descriptor(&self, index: u32) -> &clap_plugin_descriptor {
            self.plugins[usize::try_from(index).expect("index should fit into usize")]
                .clap_plugin_descriptor()
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
        CLAP_PLUGIN_FACTORY_ID, CLAP_VERSION, clap_host, clap_plugin, clap_plugin_descriptor,
        clap_plugin_entry, clap_plugin_factory,
    };

    pub use crate::factory::Factory;
    pub use crate::host::ClapHost;
    pub use crate::plugin::PluginDescriptor;

    pub fn plugin_prototype<P: Plugin>() -> Box<PluginDescriptor<P>> {
        Box::new(PluginDescriptor::allocate())
    }

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
                    factory_init_once().plugins_count()
                }

                extern "C" fn get_plugin_descriptor(
                    _: *const clap_plugin_factory,
                    index: u32,
                ) -> *const clap_plugin_descriptor {
                    factory_init_once().descriptor(index)
                }

                extern "C" fn create_plugin(
                    _: *const clap_plugin_factory,
                    host: *const clap_host,
                    plugin_id: *const std::ffi::c_char,
                ) -> *const clap_plugin {
                    if plugin_id.is_null() || host.is_null() {
                        return std::ptr::null();
                    }

                    // Safety: We just checked that host is non-null.
                    let host = ClapHost::new(unsafe{ std::ptr::NonNull::new_unchecked(host as *mut _)});
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

                extern "C" fn init(_plugin_path: *const std::ffi::c_char) -> bool {
                    true
                }

                extern "C" fn deinit() {}

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
                static clap_entry: clap_plugin_entry = clap_plugin_entry {
                    clap_version: CLAP_VERSION,
                    init: Some(init),
                    deinit: Some(deinit),
                    get_factory: Some(get_factory),
                };
            }
        };
    }
}
