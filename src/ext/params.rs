use std::{
    ffi::CStr,
    fmt::{Display, Formatter},
};

use crate::{
    events::{InputEvents, OutputEvents},
    ext,
    ffi::{
        CLAP_PARAM_CLEAR_ALL, CLAP_PARAM_CLEAR_AUTOMATIONS, CLAP_PARAM_CLEAR_MODULATIONS,
        CLAP_PARAM_IS_AUTOMATABLE, CLAP_PARAM_IS_AUTOMATABLE_PER_CHANNEL,
        CLAP_PARAM_IS_AUTOMATABLE_PER_KEY, CLAP_PARAM_IS_AUTOMATABLE_PER_NOTE_ID,
        CLAP_PARAM_IS_AUTOMATABLE_PER_PORT, CLAP_PARAM_IS_BYPASS, CLAP_PARAM_IS_ENUM,
        CLAP_PARAM_IS_HIDDEN, CLAP_PARAM_IS_MODULATABLE, CLAP_PARAM_IS_MODULATABLE_PER_CHANNEL,
        CLAP_PARAM_IS_MODULATABLE_PER_KEY, CLAP_PARAM_IS_MODULATABLE_PER_NOTE_ID,
        CLAP_PARAM_IS_MODULATABLE_PER_PORT, CLAP_PARAM_IS_PERIODIC, CLAP_PARAM_IS_READONLY,
        CLAP_PARAM_IS_STEPPED, CLAP_PARAM_REQUIRES_PROCESS, CLAP_PARAM_RESCAN_ALL,
        CLAP_PARAM_RESCAN_INFO, CLAP_PARAM_RESCAN_TEXT, CLAP_PARAM_RESCAN_VALUES, clap_host_params,
    },
    host::Host,
    id,
    id::ClapId,
    impl_flags_u32,
    plugin::Plugin,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u32)]
pub enum InfoFlags {
    /// Is this param stepped? (integer values only)
    /// if so the double value is converted to integer using a cast (equivalent
    /// to trunc).
    Stepped = CLAP_PARAM_IS_STEPPED,
    /// Useful for periodic parameters like a phase
    Periodic = CLAP_PARAM_IS_PERIODIC,
    /// The parameter should not be shown to the user, because it is currently
    /// not used. It is not necessary to process automation for this
    /// parameter.
    Hidden = CLAP_PARAM_IS_HIDDEN,
    /// The parameter can't be changed by the host.
    Readonly = CLAP_PARAM_IS_READONLY,
    /// This parameter is used to merge the plugin and host bypass button.
    /// It implies that the parameter is stepped.
    /// min: 0 -> bypass off
    /// max: 1 -> bypass on
    Bypass = CLAP_PARAM_IS_BYPASS,
    /// When set:
    /// - automation can be recorded
    /// - automation can be played back
    ///
    /// The host can send live user changes for this parameter regardless of
    /// this flag.
    ///
    /// If this parameter affects the internal processing structure of the
    /// plugin, ie: max delay, fft size, ... and the plugins needs to
    /// re-allocate its working buffers, then it should call
    /// host->request_restart(), and perform the change once the plugin is
    /// re-activated.
    Automatable = CLAP_PARAM_IS_AUTOMATABLE,
    /// Does this parameter support per note automations?
    AutomatablePerNoteId = CLAP_PARAM_IS_AUTOMATABLE_PER_NOTE_ID,
    /// Does this parameter support per key automations?
    AutomatablePerKey = CLAP_PARAM_IS_AUTOMATABLE_PER_KEY,
    /// Does this parameter support per channel automations?
    AutomatablePerChannel = CLAP_PARAM_IS_AUTOMATABLE_PER_CHANNEL,
    /// Does this parameter support per port automations?
    AutomatablePerPort = CLAP_PARAM_IS_AUTOMATABLE_PER_PORT,
    /// Does this parameter support the modulation signal?
    Modulatable = CLAP_PARAM_IS_MODULATABLE,
    /// Does this parameter support per note modulations?
    ModulatablePerNoteId = CLAP_PARAM_IS_MODULATABLE_PER_NOTE_ID,
    /// Does this parameter support per key modulations?
    ModulatablePerKey = CLAP_PARAM_IS_MODULATABLE_PER_KEY,
    /// Does this parameter support per channel modulations?
    ModulatablePerChannel = CLAP_PARAM_IS_MODULATABLE_PER_CHANNEL,
    /// Does this parameter support per port modulations?
    ModulatablePerPort = CLAP_PARAM_IS_MODULATABLE_PER_PORT,
    /// Any change to this parameter will affect the plugin output and requires
    /// to be done via process() if the plugin is active.
    ///
    /// A simple example would be a DC Offset, changing it will change the
    /// output signal and must be processed.
    RequiresProcess = CLAP_PARAM_REQUIRES_PROCESS,
    /// This parameter represents an enumerated value.
    /// If you set this flag, then you must set IsStepped too.
    /// All values from min to max must not have a blank value_to_text().
    Enum = CLAP_PARAM_IS_ENUM,
}

impl_flags_u32!(InfoFlags);

/// Describes a parameter.
#[derive(Debug, Clone, PartialEq)]
pub struct ParamInfo {
    /// Stable parameter identifier, it must never change.
    pub id: ClapId,

    pub flags: u32,

    /// The display name. eg: "Volume". This does not need to be unique. Do not
    /// include the module text in this. The host should concatenate/format
    /// the module + name in the case where showing the name alone would be
    /// too vague.
    pub name: String,

    // The module path containing the param, eg: "Oscillators/Wavetable 1".
    // '/' will be used as a separator to show a tree-like structure.
    pub module: String,

    /// Minimum plain value. Must be finite.
    pub min_value: f64,
    /// Maximum plain value. Must be finite.
    pub max_value: f64,
    /// Default plain value. Must be in [min, max] range.
    pub default_value: f64,
}

impl ParamInfo {
    /// # Safety
    ///
    /// The `values` fields: 'name' and 'module' must be valid, null-terminated
    /// C strings.
    pub unsafe fn try_from_unchecked(value: clap_param_info) -> Result<Self, Error> {
        Ok(Self {
            id: value.id.try_into().unwrap_or(ClapId::invalid_id()),
            flags: value.flags,
            // SAFETY: The safety condition is upheld by the caller.
            name: unsafe { CStr::from_ptr(value.name.as_ptr()) }
                .to_str()?
                .to_owned(),
            // SAFETY: The safety condition is upheld by the caller.
            module: unsafe { CStr::from_ptr(value.module.as_ptr()) }
                .to_str()?
                .to_owned(),
            min_value: value.min_value,
            max_value: value.max_value,
            default_value: value.default_value,
        })
    }
}

pub trait Params<P: Plugin> {
    fn count(plugin: &P) -> u32;

    fn get_info(plugin: &P, param_index: u32) -> Option<ParamInfo>;

    fn get_value(plugin: &P, param_id: ClapId) -> Option<f64>;

    /// Fills out_buffer with a null-terminated UTF-8 string that represents the
    /// parameter at the given 'value' argument. eg: "2.3 kHz". The host
    /// should always use this to format parameter values before displaying
    /// it to the user.
    fn value_to_text(
        plugin: &P,
        param_id: ClapId,
        value: f64,
        out_buf: &mut [u8],
    ) -> Result<(), crate::Error>;

    /// Converts the null-terminated UTF-8 param_value_text into a double and
    /// writes it to out_value. The host can use this to convert user input
    /// into a parameter value.
    fn text_to_value(
        plugin: &P,
        param_id: ClapId,
        param_value_text: &str,
    ) -> Result<f64, crate::Error>;

    /// Flushes a set of parameter changes.
    /// This method must not be called concurrently to clap_plugin->process().
    ///
    /// Note: if the plugin is processing, then the process() call will already
    /// achieve the parameter update (bidirectional), so a call to flush
    /// isn't required, also be aware that the plugin may use the sample
    /// offset in process(), while this information would be lost within
    /// flush().
    fn flush_inactive(plugin: &P, in_events: &InputEvents, out_events: &OutputEvents);

    fn flush(audio_thread: &P::AudioThread, in_events: &InputEvents, out_events: &OutputEvents);
}

impl<P: Plugin> Params<P> for () {
    fn count(_: &P) -> u32 {
        0
    }

    fn get_info(_: &P, _: u32) -> Option<ParamInfo> {
        None
    }

    fn get_value(_: &P, _: ClapId) -> Option<f64> {
        None
    }

    fn value_to_text(_: &P, _: ClapId, value: f64, _: &mut [u8]) -> Result<(), crate::Error> {
        Err(Error::ConvertToText(value).into())
    }

    fn text_to_value(_: &P, _: ClapId, _: &str) -> Result<f64, crate::Error> {
        Err(Error::ConvertToValue.into())
    }

    fn flush_inactive(_: &P, _: &InputEvents, _: &OutputEvents) {}

    fn flush(_: &P::AudioThread, _: &InputEvents, _: &OutputEvents) {}
}

pub(crate) use ffi::PluginParams;

use crate::ffi::clap_param_info;

mod ffi {
    use std::{
        ffi::{CStr, c_char},
        marker::PhantomData,
        ptr::{copy_nonoverlapping, slice_from_raw_parts_mut},
    };

    use crate::{
        events::{InputEvents, OutputEvents},
        ext::params::{Error, Params},
        ffi::{
            clap_id, clap_input_events, clap_output_events, clap_param_info, clap_plugin,
            clap_plugin_params,
        },
        plugin::{ClapPlugin, Plugin},
    };

    /// # SAFETY:
    ///
    /// `N` must be larger than 0. `src` and `dst` buffers must be
    /// non-overlapping.
    ///
    /// A trailing null byte will be added.  At most `N-1` bytes will be copied.
    unsafe fn copy_utf8_to_cstr<const N: usize>(src: &str, dst: &mut [c_char; N]) {
        // N > 0, so subtracting 1 won't underflow.
        let n = src.len().min(N - 1);
        // SAFETY: The caller upholds the safety requirements.
        unsafe {
            copy_nonoverlapping(src.as_ptr(), dst.as_mut_ptr() as *mut _, n);
        }
        // n is within bounds.
        dst[n] = b'\0' as _;
    }

    extern "C-unwind" fn count<E, P>(plugin: *const clap_plugin) -> u32
    where
        P: Plugin,
        E: Params<P>,
    {
        if plugin.is_null() {
            return 0;
        }
        // SAFETY: We just checked that the pointer is non-null and the plugin
        // has been obtained from host and is tied to type P.
        let mut clap_plugin = unsafe { ClapPlugin::<P>::new_unchecked(plugin) };

        // SAFETY: This function is called on the main thread.
        // It is guaranteed that we are the only function accessing the plugin now.
        // So the mutable reference to plugin for the duration of this call is
        // safe.
        let plugin = unsafe { clap_plugin.plugin() };

        E::count(plugin)
    }

    extern "C-unwind" fn get_info<E, P>(
        plugin: *const clap_plugin,
        param_index: u32,
        param_info: *mut clap_param_info,
    ) -> bool
    where
        P: Plugin,
        E: Params<P>,
    {
        if plugin.is_null() {
            return false;
        }
        // SAFETY: We just checked that the pointer is non-null and the plugin
        // has been obtained from host and is tied to type P.
        let mut clap_plugin = unsafe { ClapPlugin::<P>::new_unchecked(plugin) };

        // SAFETY: This function is called on the main thread.
        // It is guaranteed that we are the only function accessing the plugin now.
        // So the mutable reference to plugin for the duration of this call is
        // safe.
        let plugin = unsafe { clap_plugin.plugin() };

        // SAFETY: We just checked if parm_info is non-null.
        let Some(param_info) = (unsafe { param_info.as_mut() }) else {
            return false;
        };
        let Some(info) = E::get_info(plugin, param_index) else {
            return false;
        };

        param_info.id = info.id.into();
        param_info.flags = info.flags;

        // SAFETY: `param_info.name.len() > 0`, and the buffers aren't overlapping.
        unsafe { copy_utf8_to_cstr(&info.name, &mut param_info.name) };
        // SAFETY: `param_info.module.len() > 0`, and the buffers aren't overlapping.
        unsafe { copy_utf8_to_cstr(&info.module, &mut param_info.module) };

        param_info.min_value = info.min_value;
        param_info.max_value = info.max_value;
        param_info.default_value = info.default_value;
        true
    }

    extern "C-unwind" fn get_value<E, P>(
        plugin: *const clap_plugin,
        param_id: clap_id,
        out_value: *mut f64,
    ) -> bool
    where
        P: Plugin,
        E: Params<P>,
    {
        if plugin.is_null() {
            return false;
        }
        // SAFETY: We just checked that the pointer is non-null and the plugin
        // has been obtained from host and is tied to type P.
        let mut clap_plugin = unsafe { ClapPlugin::<P>::new_unchecked(plugin) };

        // SAFETY: This function is called on the main thread.
        // It is guaranteed that we are the only function accessing the plugin now.
        // So the mutable reference to plugin for the duration of this call is
        // safe.
        let plugin = unsafe { clap_plugin.plugin() };

        let Ok(param_id) = param_id.try_into() else {
            return false;
        };
        let Some(value) = E::get_value(plugin, param_id) else {
            return false;
        };

        unsafe { out_value.as_mut() }.map(|v| *v = value).is_some()
    }

    extern "C-unwind" fn value_to_text<E, P>(
        plugin: *const clap_plugin,
        param_id: clap_id,
        value: f64,
        out_buffer: *mut c_char,
        out_buffer_capacity: u32,
    ) -> bool
    where
        P: Plugin,
        E: Params<P>,
    {
        if plugin.is_null() {
            return false;
        }
        // SAFETY: We just checked that the pointer is non-null and the plugin
        // has been obtained from host and is tied to type P.
        let mut clap_plugin = unsafe { ClapPlugin::<P>::new_unchecked(plugin) };

        // SAFETY: This function is called on the main thread.
        // It is guaranteed that we are the only function accessing the plugin now.
        // So the mutable reference to plugin for the duration of this call is
        // safe.
        let plugin = unsafe { clap_plugin.plugin() };

        let out_buffer_capacity = if out_buffer_capacity > 0 {
            debug_assert!(usize::try_from(out_buffer_capacity).is_ok());
            out_buffer_capacity as usize
        } else {
            return true;
        };
        let buf = if !out_buffer.is_null() {
            unsafe { &mut *slice_from_raw_parts_mut(out_buffer as *mut u8, out_buffer_capacity) }
        } else {
            return false;
        };
        // We fill `buf` with zeroes, so that the user supplied string will be
        // null-terminated no matter what length.
        buf.fill(b'\0');

        let Ok(param_id) = param_id.try_into() else {
            return false;
        };
        E::value_to_text(
            plugin,
            param_id,
            value,
            &mut buf[0..out_buffer_capacity - 1],
        )
        .is_ok()
    }

    extern "C-unwind" fn text_to_value<E, P>(
        plugin: *const clap_plugin,
        param_id: clap_id,
        param_value_text: *const c_char,
        out_value: *mut f64,
    ) -> bool
    where
        P: Plugin,
        E: Params<P>,
    {
        if plugin.is_null() {
            return false;
        }
        // SAFETY: We just checked that the pointer is non-null and the plugin
        // has been obtained from host and is tied to type P.
        let mut clap_plugin = unsafe { ClapPlugin::<P>::new_unchecked(plugin) };

        // SAFETY: This function is called on the main thread.
        // It is guaranteed that we are the only function accessing the plugin now.
        // So the mutable reference to plugin for the duration of this call is
        // safe.
        let plugin = unsafe { clap_plugin.plugin() };

        let write_value = || -> Result<(), Error> {
            let text = unsafe { param_value_text.as_ref() }
                .map(|p| unsafe { CStr::from_ptr(p) }.to_str())
                .ok_or(Error::Nullptr)??;
            let value = E::text_to_value(plugin, param_id.try_into()?, text)
                .map_err(|_| Error::ConvertToValue)?;
            unsafe { out_value.as_mut() }
                .map(|v| *v = value)
                .ok_or(Error::Nullptr)
        };

        write_value().is_ok()
    }

    extern "C-unwind" fn flush<E, P>(
        plugin: *const clap_plugin,
        r#in: *const clap_input_events,
        out: *const clap_output_events,
    ) where
        P: Plugin,
        E: Params<P>,
    {
        let Some(r#in) = (unsafe { r#in.as_ref() }) else {
            return;
        };
        let in_events = if r#in.size.is_some() && r#in.get.is_some() {
            unsafe { InputEvents::new_unchecked(r#in) }
        } else {
            return;
        };

        let Some(r#out) = (unsafe { out.as_ref() }) else {
            return;
        };
        let out_events = if out.try_push.is_some() {
            unsafe { OutputEvents::new_unchecked(out) }
        } else {
            return;
        };

        if plugin.is_null() {
            return;
        }
        // SAFETY: We just checked that the pointer is non-null and the plugin
        // has been obtained from host and is tied to type P.
        let mut clap_plugin = unsafe { ClapPlugin::<P>::new_unchecked(plugin) };

        if clap_plugin.is_active() {
            // SAFETY: This function is called on the audio thread.  It is guaranteed that
            // we are the only function accessing audio_thread now. So a mutable reference
            // to audio_thread for the duration of this call is safe.
            let audio_thread = unsafe { clap_plugin.audio_thread() }.unwrap();
            E::flush(audio_thread, &in_events, &out_events)
        } else {
            // SAFETY: This function is called on the main thread.
            // It is guaranteed that we are the only function accessing the plugin now.
            // So the mutable reference to plugin for the duration of this call is
            // safe.
            let plugin = unsafe { clap_plugin.plugin() };
            E::flush_inactive(plugin, &in_events, &out_events);
        }
    }

    pub struct PluginParams<P> {
        #[allow(unused)]
        clap_plugin_params: clap_plugin_params,
        _marker: PhantomData<P>,
    }

    impl<P: Plugin> PluginParams<P> {
        pub fn new<E: Params<P>>(_: E) -> Self {
            Self {
                clap_plugin_params: clap_plugin_params {
                    count: Some(count::<E, P>),
                    get_info: Some(get_info::<E, P>),
                    get_value: Some(get_value::<E, P>),
                    value_to_text: Some(value_to_text::<E, P>),
                    text_to_value: Some(text_to_value::<E, P>),
                    flush: Some(flush::<E, P>),
                },
                _marker: PhantomData,
            }
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u32)]
pub enum RescanFlags {
    /// The parameter values did change, e.g. after loading a preset.
    /// The host will scan all the parameters value.
    /// The host will not record those changes as automation points.
    /// New values takes effect immediately.
    Values = CLAP_PARAM_RESCAN_VALUES,
    /// The value to text conversion changed, and the text needs to be rendered
    /// again.
    Text = CLAP_PARAM_RESCAN_TEXT,
    /// The parameter info did change, use this flag for:
    /// - name change
    /// - module change
    /// - is_periodic (flag)
    /// - is_hidden (flag)
    ///
    /// New info takes effect immediately.
    Info = CLAP_PARAM_RESCAN_INFO,
    /// Invalidates everything the host knows about parameters.
    /// It can only be used while the plugin is deactivated.
    /// If the plugin is activated use clap_host->restart() and delay any change
    /// until the host calls clap_plugin->deactivate().
    ///
    /// You must use this flag if:
    /// - some parameters were added or removed.
    /// - some parameters had critical changes:
    ///   - is_per_note (flag)
    ///   - is_per_key (flag)
    ///   - is_per_channel (flag)
    ///   - is_per_port (flag)
    ///   - is_readonly (flag)
    ///   - is_bypass (flag)
    ///   - is_stepped (flag)
    ///   - is_modulatable (flag)
    ///   - min_value
    ///   - max_value
    ///   - cookie
    All = CLAP_PARAM_RESCAN_ALL,
}

impl_flags_u32!(RescanFlags);

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u32)]
pub enum ClearFlags {
    /// Clears all possible references to a parameter
    All = CLAP_PARAM_CLEAR_ALL,
    /// Clears all automations to a parameter
    Automations = CLAP_PARAM_CLEAR_AUTOMATIONS,
    /// Clears all modulations to a parameter
    Modulations = CLAP_PARAM_CLEAR_MODULATIONS,
}

impl_flags_u32!(ClearFlags);

#[derive(Debug)]
pub struct HostParams<'a> {
    host: &'a Host,
    clap_host_params: &'a clap_host_params,
}

impl<'a> HostParams<'a> {
    /// # Safety
    ///
    /// All extension interface function pointers must be non-null (Some), and
    /// the functions must be thread-safe.
    pub(crate) const unsafe fn new_unchecked(
        host: &'a Host,
        clap_host_params: &'a clap_host_params,
    ) -> Self {
        Self {
            host,
            clap_host_params,
        }
    }

    /// Rescan the full list of parameters according to the flags.
    pub fn rescan(&self, flags: u32) {
        // SAFETY: By construction the function pointer: `clap_host_param.rescan` is
        // non-null (Some).
        unsafe { self.clap_host_params.rescan.unwrap()(self.host.clap_host(), flags) }
    }

    /// Clears references to a parameter.
    pub fn clear(&self, param_id: ClapId, flags: u32) {
        // SAFETY: By construction the function pointer: `clap_host_param.clear` is
        // non-null (Some).
        unsafe {
            self.clap_host_params.clear.unwrap()(self.host.clap_host(), param_id.into(), flags)
        }
    }

    /// Request a parameter flush.
    ///
    /// The host will then schedule a call to either:
    /// - clap_plugin.process()
    /// - clap_plugin_params.flush()
    ///
    /// This function is always safe to use and should not be called from an
    /// audio-thread as the plugin would already be within process() or
    /// flush().
    pub fn request_flush(&self) {
        // SAFETY: By construction the function pointer: `clap_host_param.request_flush`
        // is non-null (Some).
        unsafe { self.clap_host_params.request_flush.unwrap()(self.host.clap_host()) }
    }
}

#[derive(Debug, PartialEq)]
pub enum Error {
    ConvertToText(f64),
    ConvertToValue,
    IdError(id::Error),
    Nullptr,
    Utf8Error(std::str::Utf8Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::ConvertToText(val) => write!(f, "conversion from value to text: {val}"),
            Error::ConvertToValue => write!(f, "conversion from text to value"),
            Error::IdError(e) => write!(f, "ClapId error: {e}"),
            Error::Nullptr => write!(f, "null pointer"),
            Error::Utf8Error(e) => write!(f, "UTF-8 encoding error: {e}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::str::Utf8Error> for Error {
    fn from(value: std::str::Utf8Error) -> Self {
        Self::Utf8Error(value)
    }
}

impl From<id::Error> for Error {
    fn from(value: id::Error) -> Self {
        Self::IdError(value)
    }
}

impl From<Error> for crate::Error {
    fn from(value: Error) -> Self {
        ext::Error::Params(value).into()
    }
}
