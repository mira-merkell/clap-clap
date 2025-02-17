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

use crate::{
    ffi::{
        CLAP_PARAM_CLEAR_ALL, CLAP_PARAM_CLEAR_AUTOMATIONS, CLAP_PARAM_CLEAR_MODULATIONS,
        CLAP_PARAM_IS_AUTOMATABLE, CLAP_PARAM_IS_AUTOMATABLE_PER_CHANNEL,
        CLAP_PARAM_IS_AUTOMATABLE_PER_KEY, CLAP_PARAM_IS_AUTOMATABLE_PER_NOTE_ID,
        CLAP_PARAM_IS_AUTOMATABLE_PER_PORT, CLAP_PARAM_IS_BYPASS, CLAP_PARAM_IS_ENUM,
        CLAP_PARAM_IS_HIDDEN, CLAP_PARAM_IS_MODULATABLE, CLAP_PARAM_IS_MODULATABLE_PER_CHANNEL,
        CLAP_PARAM_IS_MODULATABLE_PER_KEY, CLAP_PARAM_IS_MODULATABLE_PER_NOTE_ID,
        CLAP_PARAM_IS_MODULATABLE_PER_PORT, CLAP_PARAM_IS_PERIODIC, CLAP_PARAM_IS_READONLY,
        CLAP_PARAM_IS_STEPPED, CLAP_PARAM_REQUIRES_PROCESS, CLAP_PARAM_RESCAN_ALL,
        CLAP_PARAM_RESCAN_INFO, CLAP_PARAM_RESCAN_TEXT, CLAP_PARAM_RESCAN_VALUES,
    },
    impl_flags_u32,
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
