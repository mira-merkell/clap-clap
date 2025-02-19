//! Main idea:
//!
//! The host sees the plugin as an atomic entity; and acts as a controller on
//! top of its parameters. The plugin is responsible for keeping its audio
//! processor and its GUI in sync.
//!
//! The host can at any time read parameters' value on the [main-thread] using
//! @ref clap_plugin_params.get_value().
//!
//! There are two options to communicate parameter value changes, and they are
//! not concurrent.
//! - send automation points during clap_plugin.process()
//! - send automation points during clap_plugin_params.flush(), for parameter
//!   changes without processing audio
//!
//! When the plugin changes a parameter value, it must inform the host.
//! It will send @ref CLAP_EVENT_PARAM_VALUE event during process() or flush().
//! If the user is adjusting the value, don't forget to mark the beginning and
//! end of the gesture by sending CLAP_EVENT_PARAM_GESTURE_BEGIN and
//! CLAP_EVENT_PARAM_GESTURE_END events.
//!
//! @note MIDI CCs are tricky because you may not know when the parameter
//! adjustment ends. Also, if the host records incoming MIDI CC and parameter
//! change automation at the same time, there will be a conflict at playback:
//! MIDI CC vs Automation. The parameter automation will always target the same
//! parameter because the param_id is stable. The MIDI CC may have a different
//! mapping in the future and may result in a different playback.
//!
//! When a MIDI CC changes a parameter's value, set the flag
//! CLAP_EVENT_DONT_RECORD in clap_event_param.header.flags. That way the host
//! may record the MIDI CC automation, but not the parameter change and there
//! won't be conflict at playback.
//!
//! Scenarios:
//!
//! I. Loading a preset
//! - load the preset in a temporary state
//! - call @ref clap_host_params.rescan() if anything changed
//! - call @ref clap_host_latency.changed() if latency changed
//! - invalidate any other info that may be cached by the host
//! - if the plugin is activated and the preset will introduce breaking changes
//!   (latency, audio ports, new parameters, ...) be sure to wait for the host
//!   to deactivate the plugin to apply those changes. If there are no breaking
//!   changes, the plugin can apply them right away. The plugin is responsible
//!   for updating both its audio processor and its gui.
//!
//! II. Turning a knob on the DAW interface
//! - the host will send an automation event to the plugin via a process() or
//!   flush()
//!
//! III. Turning a knob on the Plugin interface
//! - the plugin is responsible for sending the parameter value to its audio
//!   processor
//! - call clap_host_params->request_flush() or clap_host->request_process().
//! - when the host calls either clap_plugin->process() or
//!   clap_plugin_params->flush(), send an automation event and don't forget to
//!   wrap the parameter change(s) with CLAP_EVENT_PARAM_GESTURE_BEGIN and
//!   CLAP_EVENT_PARAM_GESTURE_END to define the beginning and end of the
//!   gesture.
//!
//! IV. Turning a knob via automation
//! - host sends an automation point during clap_plugin->process() or
//!   clap_plugin_params->flush().
//! - the plugin is responsible for updating its GUI
//!
//! V. Turning a knob via plugin's internal MIDI mapping
//! - the plugin sends a CLAP_EVENT_PARAM_VALUE output event, set should_record
//!   to false
//! - the plugin is responsible for updating its GUI
//!
//! VI. Adding or removing parameters
//! - if the plugin is activated call clap_host->restart()
//! - once the plugin isn't active:
//!   - apply the new state
//!   - if a parameter is gone or is created with an id that may have been used
//!     before, call clap_host_params.clear(host, param_id,
//!     CLAP_PARAM_CLEAR_ALL)
//!   - call clap_host_params->rescan(CLAP_PARAM_RESCAN_ALL)
//!
//! CLAP allows the plugin to change the parameter range, yet the plugin
//! developer should be aware that doing so isn't without risk, especially if
//! you made the promise to never change the sound. If you want to be 100%
//! certain that the sound will not change with all host, then simply never
//! change the range.
//!
//! There are two approaches to automations, either you automate the plain
//! value, or you automate the knob position. The first option will be robust to
//! a range increase, while the second won't be.
//!
//! If the host goes with the second approach (automating the knob position), it
//! means that the plugin is hosted in a relaxed environment regarding sound
//! changes (they are accepted, and not a concern as long as they are
//! reasonable). Though, stepped parameters should be stored as plain value in
//! the document.
//!
//! If the host goes with the first approach, there will still be situation
//! where the sound may inevitably change. For example, if the plugin increase
//! the range, there is an automation playing at the max value and on top of
//! that an LFO is applied. See the following curve:
//!                                   .
//!                                  . .
//!          .....                  .   .
//! before: .     .     and after: .     .
//!
//! Persisting parameter values:
//!
//! Plugins are responsible for persisting their parameter's values between
//! sessions by implementing the state extension. Otherwise, parameter value
//! will not be recalled when reloading a project. Hosts should _not_ try to
//! save and restore parameter values for plugins that don't implement the state
//! extension.
//!
//! Advice for the host:
//!
//! - store plain values in the document (automation)
//! - store modulation amount in plain value delta, not in percentage
//! - when you apply a CC mapping, remember the min/max plain values so you can
//!   adjust
//! - do not implement a parameter saving fall back for plugins that don't
//!   implement the state extension
//!
//! Advice for the plugin:
//!
//! - think carefully about your parameter range when designing your DSP
//! - avoid shrinking parameter ranges, they are very likely to change the sound
//! - consider changing the parameter range as a tradeoff: what you improve vs
//!   what you break
//! - make sure to implement saving and loading the parameter values using the
//!   state extension
//! - if you plan to use adapters for other plugin formats, then you need to pay
//!   extra attention to the adapter requirements

use std::{
    ffi::c_void,
    fmt::{Display, Formatter},
    ptr::NonNull,
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
pub enum ParamInfoFlags {
    /// Is this param stepped? (integer values only)
    /// if so the double value is converted to integer using a cast (equivalent
    /// to trunc).
    IsStepped = CLAP_PARAM_IS_STEPPED,
    /// Useful for periodic parameters like a phase
    IsPeriodic = CLAP_PARAM_IS_PERIODIC,
    /// The parameter should not be shown to the user, because it is currently
    /// not used. It is not necessary to process automation for this
    /// parameter.
    IsHidden = CLAP_PARAM_IS_HIDDEN,
    /// The parameter can't be changed by the host.
    IsReadonly = CLAP_PARAM_IS_READONLY,
    /// This parameter is used to merge the plugin and host bypass button.
    /// It implies that the parameter is stepped.
    /// min: 0 -> bypass off
    /// max: 1 -> bypass on
    IsBypass = CLAP_PARAM_IS_BYPASS,
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
    IsAutomatable = CLAP_PARAM_IS_AUTOMATABLE,
    /// Does this parameter support per note automations?
    IsAutomatablePerNoteId = CLAP_PARAM_IS_AUTOMATABLE_PER_NOTE_ID,
    /// Does this parameter support per key automations?
    IsAutomatablePerKey = CLAP_PARAM_IS_AUTOMATABLE_PER_KEY,
    /// Does this parameter support per channel automations?
    IsAutomatablePerChannel = CLAP_PARAM_IS_AUTOMATABLE_PER_CHANNEL,
    /// Does this parameter support per port automations?
    IsAutomatablePerPort = CLAP_PARAM_IS_AUTOMATABLE_PER_PORT,
    /// Does this parameter support the modulation signal?
    IsModulatable = CLAP_PARAM_IS_MODULATABLE,
    /// Does this parameter support per note modulations?
    IsModulatablePerNoteId = CLAP_PARAM_IS_MODULATABLE_PER_NOTE_ID,
    /// Does this parameter support per key modulations?
    IsModulatablePerKey = CLAP_PARAM_IS_MODULATABLE_PER_KEY,
    /// Does this parameter support per channel modulations?
    IsModulatablePerChannel = CLAP_PARAM_IS_MODULATABLE_PER_CHANNEL,
    /// Does this parameter support per port modulations?
    IsModulatablePerPort = CLAP_PARAM_IS_MODULATABLE_PER_PORT,
    /// Any change to this parameter will affect the plugin output and requires
    /// to be done via process() if the plugin is active.
    ///
    /// A simple example would be a DC Offset, changing it will change the
    /// output signal and must be processed.
    RequiresProcess = CLAP_PARAM_REQUIRES_PROCESS,
    /// This parameter represents an enumerated value.
    /// If you set this flag, then you must set IsStepped too.
    /// All values from min to max must not have a blank value_to_text().
    IsEnum = CLAP_PARAM_IS_ENUM,
}

impl_flags_u32!(ParamInfoFlags);

/// Describes a parameter.
#[derive(Debug, Clone)]
pub struct ParamInfo {
    /// Stable parameter identifier, it must never change.
    pub id: ClapId,

    pub flags: u32,

    /// This value is optional and set by the plugin.
    /// Its purpose is to provide fast access to the plugin parameter object by
    /// caching its pointer. For instance:
    ///
    /// in clap_plugin_params.get_info():
    ///    Parameter *p = findParameter(param_id);
    ///    param_info->cookie = p;
    ///
    /// later, in clap_plugin.process():
    ///
    ///    Parameter *p = (Parameter *)event->cookie;
    ///    if (!p) [[unlikely]]
    ///       p = findParameter(event->param_id);
    ///
    /// where findParameter() is a function the plugin implements to map
    /// parameter ids to internal objects.
    ///
    /// Important:
    ///  - The cookie is invalidated by a call to
    ///    clap_host_params->rescan(CLAP_PARAM_RESCAN_ALL) or when the plugin is
    ///    destroyed.
    ///  - The host will either provide the cookie as issued or nullptr in
    ///    events addressing parameters.
    ///  - The plugin must gracefully handle the case of a cookie which is
    ///    nullptr.
    ///  - Many plugins will process the parameter events more quickly if the
    ///    host can provide the cookie in a faster time than a hashmap lookup
    ///    per param per event.
    pub cookie: Option<NonNull<c_void>>,

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
    ) -> Result<(), Error>;

    /// Converts the null-terminated UTF-8 param_value_text into a double and
    /// writes it to out_value. The host can use this to convert user input
    /// into a parameter value.
    fn text_to_value(plugin: &P, param_id: ClapId, param_value_text: &str) -> Result<f64, Error>;

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

    fn value_to_text(_: &P, _: ClapId, value: f64, _: &mut [u8]) -> Result<(), Error> {
        Err(Error::ConvertToText(value))
    }

    fn text_to_value(_: &P, _: ClapId, _: &str) -> Result<f64, Error> {
        Err(Error::ConvertToValue)
    }

    fn flush_inactive(_: &P, _: &InputEvents, _: &OutputEvents) {}

    fn flush(_: &P::AudioThread, _: &InputEvents, _: &OutputEvents) {}
}

pub(crate) use ffi::ClapPluginParams;

mod ffi {
    use std::{
        ffi::{CStr, c_char},
        marker::PhantomData,
        ptr::{null_mut, slice_from_raw_parts_mut},
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
        let mut clap_plugin = unsafe { ClapPlugin::<P>::new(plugin) };

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
        let mut clap_plugin = unsafe { ClapPlugin::<P>::new(plugin) };

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
        param_info.cookie = if let Some(cookie) = info.cookie {
            cookie.as_ptr()
        } else {
            null_mut()
        };

        let n = info.name.len().min(param_info.name.len());
        unsafe {
            std::ptr::copy_nonoverlapping(
                info.name.as_ptr(),
                param_info.name.as_mut_ptr() as *mut _,
                n,
            )
        }
        param_info.name[n] = b'\0' as _;

        let n = info.module.len().min(param_info.module.len());
        unsafe {
            std::ptr::copy_nonoverlapping(
                info.module.as_ptr(),
                param_info.module.as_mut_ptr() as *mut _,
                n,
            )
        }
        param_info.module[n] = b'\0' as _;

        param_info.default_value = info.default_value;
        param_info.min_value = info.min_value;
        param_info.max_value = info.max_value;
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
        let mut clap_plugin = unsafe { ClapPlugin::<P>::new(plugin) };

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
        let mut clap_plugin = unsafe { ClapPlugin::<P>::new(plugin) };

        // SAFETY: This function is called on the main thread.
        // It is guaranteed that we are the only function accessing the plugin now.
        // So the mutable reference to plugin for the duration of this call is
        // safe.
        let plugin = unsafe { clap_plugin.plugin() };

        let out_buffer_capacity = if out_buffer_capacity > 0 {
            debug_assert!((out_buffer_capacity as u64) < usize::MAX as u64);
            out_buffer_capacity as usize
        } else {
            return true;
        };
        let buf = if !out_buffer.is_null() {
            unsafe { &mut *slice_from_raw_parts_mut(out_buffer as *mut u8, out_buffer_capacity) }
        } else {
            return false;
        };
        let Ok(param_id) = param_id.try_into() else {
            return false;
        };
        E::value_to_text(
            plugin,
            param_id,
            value,
            &mut buf[0..out_buffer_capacity - 1],
        )
        .map(|_| {
            buf[out_buffer_capacity - 1] = b'\0';
        })
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
        let mut clap_plugin = unsafe { ClapPlugin::<P>::new(plugin) };

        // SAFETY: This function is called on the main thread.
        // It is guaranteed that we are the only function accessing the plugin now.
        // So the mutable reference to plugin for the duration of this call is
        // safe.
        let plugin = unsafe { clap_plugin.plugin() };

        let write_value = || -> Result<(), Error> {
            let text = unsafe { param_value_text.as_ref() }
                .map(|p| unsafe { CStr::from_ptr(p) }.to_str())
                .ok_or(Error::Nullptr)??;
            let value = E::text_to_value(plugin, param_id.try_into()?, text)?;
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
            unsafe { InputEvents::new_unchecked(&*r#in) }
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
        let mut clap_plugin = unsafe { ClapPlugin::<P>::new(plugin) };

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

    pub struct ClapPluginParams<P> {
        #[allow(unused)]
        clap_plugin_params: clap_plugin_params,
        _marker: PhantomData<P>,
    }

    impl<P: Plugin> ClapPluginParams<P> {
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
pub enum ParamRescanFlags {
    /// The parameter values did change, e.g. after loading a preset.
    /// The host will scan all the parameters value.
    /// The host will not record those changes as automation points.
    /// New values takes effect immediately.
    ClapParamRescanValues = CLAP_PARAM_RESCAN_VALUES,
    /// The value to text conversion changed, and the text needs to be rendered
    /// again.
    ClapParamRescanText = CLAP_PARAM_RESCAN_TEXT,
    /// The parameter info did change, use this flag for:
    /// - name change
    /// - module change
    /// - is_periodic (flag)
    /// - is_hidden (flag)
    ///
    /// New info takes effect immediately.
    ClapParamRescanInfo = CLAP_PARAM_RESCAN_INFO,
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
    ClapParamRescanAll = CLAP_PARAM_RESCAN_ALL,
}

impl_flags_u32!(ParamRescanFlags);

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u32)]
pub enum ParamClearFlags {
    /// Clears all possible references to a parameter
    ClapParamClearAll = CLAP_PARAM_CLEAR_ALL,
    /// Clears all automations to a parameter
    ClapParamClearAutomations = CLAP_PARAM_CLEAR_AUTOMATIONS,
    /// Clears all modulations to a parameter
    ClapParamClearModulations = CLAP_PARAM_CLEAR_MODULATIONS,
}

impl_flags_u32!(ParamClearFlags);

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
        todo!()
    }

    /// Clears references to a parameter.
    pub fn clear(&self, param_id: ClapId, flags: u32) {
        todo!()
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
        todo!()
    }
}

#[derive(Debug)]
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
