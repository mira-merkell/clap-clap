use std::fmt::{Display, Formatter};

pub(crate) use ffi::ClapPluginAudioPorts;

use crate::{
    ffi::{
        CLAP_AUDIO_PORTS_RESCAN_CHANNEL_COUNT, CLAP_AUDIO_PORTS_RESCAN_FLAGS,
        CLAP_AUDIO_PORTS_RESCAN_IN_PLACE_PAIR, CLAP_AUDIO_PORTS_RESCAN_LIST,
        CLAP_AUDIO_PORTS_RESCAN_NAMES, CLAP_AUDIO_PORTS_RESCAN_PORT_TYPE, clap_host_audio_ports,
    },
    impl_flags_u32,
    plugin::Plugin,
    prelude::Host,
};

pub trait AudioPorts<P>
where
    P: Plugin,
{
    fn count(plugin: &P, is_input: bool) -> u32;
    fn get(plugin: &P, index: u32, is_input: bool) -> Option<AudioPortInfo>;
}

mod ffi {
    use std::marker::PhantomData;

    use crate::{
        ext::audio_ports::AudioPorts,
        ffi::{clap_audio_port_info, clap_plugin, clap_plugin_audio_ports},
        plugin::{ClapPlugin, Plugin},
    };

    extern "C-unwind" fn count<A, P>(plugin: *const clap_plugin, is_input: bool) -> u32
    where
        P: Plugin,
        A: AudioPorts<P>,
    {
        if plugin.is_null() {
            return 0;
        }
        // SAFETY: We just checked that the pointer is non-null and the plugin
        // has been obtained from host and is tied to type P.
        let mut clap_plugin = unsafe { ClapPlugin::<P>::new(plugin) };

        // SAFETY: This function is called on the main thread.
        // It is guaranteed that we are the only function accessing the plugin now.
        // So the mutable reference to plugin for the duration of this call is
        // safe.
        let plugin = unsafe { clap_plugin.plugin() };

        A::count(plugin, is_input)
    }

    extern "C-unwind" fn get<A, P>(
        plugin: *const clap_plugin,
        index: u32,
        is_input: bool,
        info: *mut clap_audio_port_info,
    ) -> bool
    where
        P: Plugin,
        A: AudioPorts<P>,
    {
        if plugin.is_null() {
            return false;
        }
        // SAFETY: We just checked that the pointer is non-null and the plugin
        // has been obtained from host and is tied to type P.
        let mut clap_plugin = unsafe { ClapPlugin::<P>::new(plugin) };

        // SAFETY: This function is called on the main thread.
        // It is guaranteed that we are the only function accessing the plugin now.
        // So the mutable reference to plugin for the duration of this call is
        // safe.
        let plugin = unsafe { clap_plugin.plugin() };

        // SAFETY: The host guarantees we are the only function that can access info
        // for the duration of the function call.  So obtaining a mutable reference
        // is safe.
        let info = unsafe { &mut *info };

        A::get(plugin, index, is_input)
            .map(|x| x.fill_clap_audio_port_info(info))
            .is_some()
    }

    pub struct ClapPluginAudioPorts<P> {
        #[allow(unused)]
        clap_plugin_audio_ports: clap_plugin_audio_ports,
        _marker: PhantomData<P>,
    }

    impl<P: Plugin> ClapPluginAudioPorts<P> {
        pub fn new<A: AudioPorts<P>>(_: A) -> Self {
            Self {
                clap_plugin_audio_ports: clap_plugin_audio_ports {
                    count: Some(count::<A, P>),
                    get: Some(get::<A, P>),
                },
                _marker: PhantomData,
            }
        }
    }
}

mod port_info {
    use std::ptr::null;

    use crate::{
        ffi::{
            CLAP_AUDIO_PORT_IS_MAIN, CLAP_AUDIO_PORT_PREFERS_64BITS,
            CLAP_AUDIO_PORT_REQUIRES_COMMON_SAMPLE_SIZE, CLAP_AUDIO_PORT_SUPPORTS_64BITS,
            CLAP_INVALID_ID, CLAP_PORT_AMBISONIC, CLAP_PORT_MONO, CLAP_PORT_STEREO,
            CLAP_PORT_SURROUND, clap_audio_port_info,
        },
        id::ClapId,
        impl_flags_u32,
        plugin::Plugin,
        prelude::AudioPorts,
    };

    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    #[repr(u32)]
    pub enum AudioPortFlags {
        /// This port is the main audio input or output. There can be only one
        /// main input and main output. Main port must be at index 0.
        IsMain = CLAP_AUDIO_PORT_IS_MAIN,
        /// This port can be used with 64 bits audio
        Supports64bits = CLAP_AUDIO_PORT_SUPPORTS_64BITS,
        /// 64 bits audio is preferred with this port
        Prefers64bits = CLAP_AUDIO_PORT_PREFERS_64BITS,
        /// This port must be used with the same sample size as all the other
        /// ports which have this flag. In other words if all ports have
        /// this flag then the plugin may either be used entirely with
        /// 64 bits audio or 32 bits audio, but it can't be mixed.
        RequiresCommonSampleSize = CLAP_AUDIO_PORT_REQUIRES_COMMON_SAMPLE_SIZE,
    }

    impl_flags_u32!(AudioPortFlags);

    #[derive(Debug, Default, Clone, PartialEq)]
    pub struct AudioPortInfo {
        pub id: ClapId,
        pub name: Option<String>,
        pub flags: u32,
        pub channel_count: u32,
        pub port_type: Option<AudioPortType>,
        pub in_place_pair: Option<ClapId>,
    }

    impl AudioPortInfo {
        pub fn builder() -> AudioPortInfoBuilder {
            AudioPortInfoBuilder::new(Self::default())
        }

        pub(super) fn fill_clap_audio_port_info(&self, info: &mut clap_audio_port_info) {
            info.id = self.id.into();

            if let Some(name) = &self.name {
                let n = name.len().min(info.name.len());
                unsafe {
                    std::ptr::copy_nonoverlapping(
                        name.as_ptr(),
                        info.name.as_mut_ptr() as *mut _,
                        n,
                    )
                }
                info.name[n] = b'\0' as _;
            } else {
                info.name[0] = b'\0' as _;
            };

            info.flags = self.flags;

            info.channel_count = self.channel_count;

            info.port_type = match self.port_type {
                Some(AudioPortType::Mono) => CLAP_PORT_MONO.as_ptr(),
                Some(AudioPortType::Stereo) => CLAP_PORT_STEREO.as_ptr(),
                Some(AudioPortType::Surround) => CLAP_PORT_SURROUND.as_ptr(),
                Some(AudioPortType::Ambisonic) => CLAP_PORT_AMBISONIC.as_ptr(),
                None => null(),
            };

            info.in_place_pair = self
                .in_place_pair
                .map(Into::into)
                .unwrap_or(CLAP_INVALID_ID);
        }
    }

    pub struct AudioPortInfoBuilder {
        info: AudioPortInfo,
    }

    impl AudioPortInfoBuilder {
        const fn new(info: AudioPortInfo) -> Self {
            Self { info }
        }

        pub const fn id(&mut self, id: ClapId) -> &mut Self {
            self.info.id = id;
            self
        }

        pub fn name(&mut self, name: &str) -> &mut Self {
            self.info.name = Some(name.to_string());
            self
        }

        pub const fn port_is_main(&mut self) -> &mut Self {
            self.info.flags |= CLAP_AUDIO_PORT_IS_MAIN;
            self
        }

        pub const fn prefers_64_bits(&mut self) -> &mut Self {
            self.info.flags |= CLAP_AUDIO_PORT_PREFERS_64BITS;
            self
        }

        pub const fn supports_64_bits(&mut self) -> &mut Self {
            self.info.flags |= CLAP_AUDIO_PORT_SUPPORTS_64BITS;
            self
        }

        pub const fn requires_common_sample_size(&mut self) -> &mut Self {
            self.info.flags |= CLAP_AUDIO_PORT_REQUIRES_COMMON_SAMPLE_SIZE;
            self
        }

        pub fn channel_count(&mut self, n: u32) -> &mut Self {
            self.info.channel_count = n;
            self
        }

        pub const fn port_type(&mut self, port_type: AudioPortType) -> &mut Self {
            self.info.port_type = Some(port_type);
            self
        }

        pub const fn in_place_pair(&mut self, id: ClapId) -> &mut Self {
            self.info.in_place_pair = Some(id);
            self
        }

        pub fn build(&self) -> AudioPortInfo {
            self.info.clone()
        }
    }

    #[derive(Debug, Copy, Clone, PartialEq)]
    pub enum AudioPortType {
        Mono,
        Stereo,
        Surround,
        Ambisonic,
    }

    impl TryFrom<&str> for AudioPortType {
        type Error = crate::ext::audio_ports::Error;

        fn try_from(value: &str) -> Result<Self, Self::Error> {
            if value == "mono" {
                Ok(AudioPortType::Mono)
            } else if value == "stereo" {
                Ok(AudioPortType::Stereo)
            } else if value == "ambisonic" {
                Ok(AudioPortType::Ambisonic)
            } else if value == "surround" {
                Ok(AudioPortType::Surround)
            } else {
                Err(Self::Error::PortType)
            }
        }
    }

    impl<P: Plugin> AudioPorts<P> for () {
        fn count(_: &P, _: bool) -> u32 {
            0
        }

        fn get(_: &P, _: u32, _: bool) -> Option<AudioPortInfo> {
            None
        }
    }
}
pub use port_info::{AudioPortFlags, AudioPortInfo, AudioPortInfoBuilder, AudioPortType};

mod static_ports {
    use crate::{
        ext::audio_ports::{AudioPortInfo, AudioPortType, AudioPorts},
        plugin::Plugin,
    };

    /// Static mono ports, in and out.
    #[derive(Default, Debug, Copy, Clone)]
    pub struct MonoPorts<const IN: u32, const OUT: u32>;

    impl<const IN: u32, const OUT: u32> MonoPorts<IN, OUT> {
        pub const fn new() -> Self {
            Self {}
        }
    }

    impl<P, const IN: u32, const OUT: u32> AudioPorts<P> for MonoPorts<IN, OUT>
    where
        P: Plugin,
    {
        fn count(_: &P, is_input: bool) -> u32 {
            if is_input { IN } else { OUT }
        }

        fn get(_: &P, index: u32, is_input: bool) -> Option<AudioPortInfo> {
            if is_input {
                (index < IN).then_some(if index == 0 {
                    AudioPortInfo::builder()
                        .id(index.try_into().unwrap())
                        .name("Main In")
                        .port_is_main()
                        .channel_count(1)
                        .port_type(AudioPortType::Mono)
                        .build()
                } else {
                    AudioPortInfo::builder()
                        .id(index.try_into().unwrap())
                        .name(format!("In {index}").as_str())
                        .channel_count(1)
                        .port_type(AudioPortType::Mono)
                        .build()
                })
            } else {
                (index < OUT).then_some(if index == 0 {
                    AudioPortInfo::builder()
                        .id((IN + index).try_into().unwrap())
                        .name("Main Out")
                        .port_is_main()
                        .channel_count(1)
                        .port_type(AudioPortType::Mono)
                        .build()
                } else {
                    AudioPortInfo::builder()
                        .id((IN + index).try_into().unwrap())
                        .name(format!("Out {index}").as_str())
                        .channel_count(1)
                        .port_type(AudioPortType::Mono)
                        .build()
                })
            }
        }
    }

    /// Single static stereo port, in and out.
    #[derive(Debug, Copy, Clone)]
    pub struct StereoPorts<const IN: u32, const OUT: u32>;

    impl<P, const IN: u32, const OUT: u32> AudioPorts<P> for StereoPorts<IN, OUT>
    where
        P: Plugin,
    {
        fn count(_: &P, is_input: bool) -> u32 {
            if is_input { IN } else { OUT }
        }

        fn get(_: &P, index: u32, is_input: bool) -> Option<AudioPortInfo> {
            if is_input {
                (index < IN).then_some(if index == 0 {
                    AudioPortInfo::builder()
                        .id(index.try_into().unwrap())
                        .name("Main In")
                        .port_is_main()
                        .channel_count(2)
                        .port_type(AudioPortType::Stereo)
                        .build()
                } else {
                    AudioPortInfo::builder()
                        .id(index.try_into().unwrap())
                        .name(format!("In {index}").as_str())
                        .channel_count(2)
                        .port_type(AudioPortType::Stereo)
                        .build()
                })
            } else {
                (index < OUT).then_some(if index == 0 {
                    AudioPortInfo::builder()
                        .id((IN + index).try_into().unwrap())
                        .name("Main Out")
                        .port_is_main()
                        .channel_count(2)
                        .port_type(AudioPortType::Stereo)
                        .build()
                } else {
                    AudioPortInfo::builder()
                        .id((IN + index).try_into().unwrap())
                        .name(format!("Out {index}").as_str())
                        .channel_count(2)
                        .port_type(AudioPortType::Stereo)
                        .build()
                })
            }
        }
    }
}
pub use static_ports::{MonoPorts, StereoPorts};

/// Port rescan flags.
///
/// # Example
///
/// ```rust
/// # use clap_clap::ext::audio_ports::RescanFlags;
/// assert_eq!(RescanFlags::Names as u32, 0b1);
/// assert!(RescanFlags::Names.is_set(0b101));
/// assert_eq!(RescanFlags::Names.set(0b100), 0b101);
/// assert_eq!(RescanFlags::Names.clear(0b101), 0b100);
/// ```
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u32)]
pub enum RescanFlags {
    /// The ports name did change, the host can scan them right away.
    Names = CLAP_AUDIO_PORTS_RESCAN_NAMES,
    ///  The flags did change
    Flags = CLAP_AUDIO_PORTS_RESCAN_FLAGS,
    ///  The channel_count did change
    ChannelCount = CLAP_AUDIO_PORTS_RESCAN_CHANNEL_COUNT,
    ///  The port type did change
    PortType = CLAP_AUDIO_PORTS_RESCAN_PORT_TYPE,
    ///  The in-place pair did change
    InPlacePair = CLAP_AUDIO_PORTS_RESCAN_IN_PLACE_PAIR,
    ///  The list of ports have changed: entries have been removed/added.
    List = CLAP_AUDIO_PORTS_RESCAN_LIST,
}

impl_flags_u32!(RescanFlags);

#[derive(Debug)]
pub struct HostAudioPorts<'a> {
    host: &'a Host,
    clap_host_audio_ports: &'a clap_host_audio_ports,
}

impl<'a> HostAudioPorts<'a> {
    /// # Safety
    ///
    /// All extension interface function pointers must be non-null (Some), and
    /// the functions must be thread-safe.
    pub(crate) const unsafe fn new_unchecked(
        host: &'a Host,
        clap_host_audio_ports: &'a clap_host_audio_ports,
    ) -> Self {
        Self {
            host,
            clap_host_audio_ports,
        }
    }

    pub fn is_rescan_flag_supported(&self, flag: RescanFlags) -> bool {
        // SAFETY: By construction, the callback must be a valid function pointer,
        // and the call is thread-safe.
        let callback = self.clap_host_audio_ports.is_rescan_flag_supported.unwrap();
        unsafe { callback(self.host.clap_host(), flag as u32) }
    }

    pub fn rescan(&self, flags: u32) {
        // SAFETY: By construction, the callback must be a valid function pointer,
        // and the call is thread-safe.
        let callback = self.clap_host_audio_ports.rescan.unwrap();
        unsafe { callback(self.host.clap_host(), flags) };
    }
}

#[derive(Debug)]
pub enum Error {
    PortType,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::PortType => write!(f, "unknown port type"),
        }
    }
}

impl std::error::Error for Error {}

impl From<Error> for crate::Error {
    fn from(value: Error) -> Self {
        match value {
            Error::PortType => crate::ext::Error::AudioPorts(value).into(),
        }
    }
}
