use crate::{
    ffi::{
        CLAP_PARAM_IS_AUTOMATABLE, CLAP_PARAM_IS_AUTOMATABLE_PER_CHANNEL,
        CLAP_PARAM_IS_AUTOMATABLE_PER_KEY, CLAP_PARAM_IS_AUTOMATABLE_PER_NOTE_ID,
        CLAP_PARAM_IS_AUTOMATABLE_PER_PORT, CLAP_PARAM_IS_BYPASS, CLAP_PARAM_IS_ENUM,
        CLAP_PARAM_IS_HIDDEN, CLAP_PARAM_IS_MODULATABLE, CLAP_PARAM_IS_MODULATABLE_PER_CHANNEL,
        CLAP_PARAM_IS_MODULATABLE_PER_KEY, CLAP_PARAM_IS_MODULATABLE_PER_NOTE_ID,
        CLAP_PARAM_IS_MODULATABLE_PER_PORT, CLAP_PARAM_IS_PERIODIC, CLAP_PARAM_IS_READONLY,
        CLAP_PARAM_IS_STEPPED, CLAP_PARAM_REQUIRES_PROCESS,
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
