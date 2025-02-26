use std::{
    ffi::{CStr, c_void},
    fmt::{Display, Formatter},
};

use crate::{
    ext::{audio_ports::HostAudioPorts, log::HostLog, params::HostParams},
    ffi::{
        CLAP_EXT_AUDIO_PORTS, CLAP_EXT_LOG, CLAP_EXT_PARAMS, clap_host, clap_host_audio_ports,
        clap_host_log, clap_host_params,
    },
    version::ClapVersion,
};

#[derive(Debug, PartialEq)]
pub struct Host {
    clap_host: *const clap_host,
}

impl Host {
    /// # Safety
    ///
    /// The host argument must be a non-null pointer to a CLAP host, obtained as
    /// the argument passed to factory create_plugin(). In particular, all
    /// fields, except for host_data, of clap_host struct must be valid
    /// pointers.
    ///
    /// # Panic
    ///
    /// The function will panic if host description strings aren't properly
    /// validated UTF-8 strings.
    #[doc(hidden)]
    pub const unsafe fn new_unchecked(clap_host: *const clap_host) -> Self {
        #[cfg(debug_assertions)]
        {
            assert!(!clap_host.is_null());
            let clap_host = unsafe { &*clap_host };
            assert!(clap_host.get_extension.is_some());
            assert!(clap_host.request_callback.is_some());
            assert!(clap_host.request_process.is_some());
            assert!(clap_host.request_restart.is_some());
        }

        Self { clap_host }
    }

    pub const fn clap_host(&self) -> &clap_host {
        // SAFETY: by construction, we can obtain a shared reference to clap_host for
        // the lifetime of self.
        unsafe { &*self.clap_host }
    }

    pub const fn clap_version(&self) -> ClapVersion {
        self.clap_host().clap_version
    }

    pub const fn get_extension(&self) -> HostExtensions {
        // SAFETY: By construction, the function pointer to `get_extension()` is valid.
        unsafe { HostExtensions::new_unchecked(self) }
    }
}

macro_rules! impl_host_get_str {
    ($($description:tt),*) => {
        impl Host {
            $(
                /// # Panic
                ///
                /// This method will panic if the host returns an invalid UTF-8 string.
                pub fn $description(&self) -> &str {
                    unsafe { CStr::from_ptr(self.clap_host().$description)
                                .to_str()
                                .expect("host description must be a valid UTF-8 string")
                    }
                }
            )*
        }
    };
}

impl_host_get_str!(name, vendor, url, version);

macro_rules! impl_host_request {
    ($($request_method:tt),*) => {
        impl Host {
            $(
                pub fn $request_method(&self) {
                    let clap_host = self.clap_host();
                    if let Some(callback) = clap_host.$request_method {
                        // SAFETY: By the Host constructon, the callback is non-null.
                        // The pointer is a valid function obtained from the CLAP host. It is
                        // guaranteed be the host that the call is safe.
                        unsafe { callback(&raw const *self.clap_host()) }
                    }
                }
            )*
        }
    };
}

impl_host_request!(request_process, request_restart, request_callback);

pub struct HostExtensions<'a> {
    host: &'a Host,
}

impl<'a> HostExtensions<'a> {
    /// # Safety
    ///
    /// The function pointer:  `host.clap_host().get_extension()` must be Some.
    const unsafe fn new_unchecked(host: &'a Host) -> Self {
        Self { host }
    }

    fn get_extension_ptr(&self, extension_id: &CStr) -> Option<*const c_void> {
        // HostExtensions constructor guarantees that unwrap won't panic.
        let callback = self.host.clap_host().get_extension.unwrap();
        // SAFETY: Host constructor guarantees that the call is safe.
        let ext_ptr = unsafe { callback(self.host.clap_host(), extension_id.as_ptr()) };
        (!ext_ptr.is_null()).then_some(ext_ptr)
    }

    pub fn audio_ports(&self) -> Result<HostAudioPorts<'a>, Error> {
        let clap_host_audio_ports = self
            .get_extension_ptr(CLAP_EXT_AUDIO_PORTS)
            .ok_or(Error::ExtensionNotFound("audio_ports"))?;

        // SAFETY: We just checked if the pointer to clap_host_audio_ports
        // is non-null. We return a reference to it for the lifetime of Host.
        let clap_host_audio_ports: &clap_host_audio_ports =
            unsafe { &*clap_host_audio_ports.cast() };

        let _ = clap_host_audio_ports
            .is_rescan_flag_supported
            .ok_or(Error::Callback("is_rescan_flag_supported"))?;
        let _ = clap_host_audio_ports
            .rescan
            .ok_or(Error::Callback("rescan"))?;

        // SAFETY: We just checked if the methods are non-null (Some).
        Ok(unsafe { HostAudioPorts::new_unchecked(self.host, clap_host_audio_ports) })
    }

    pub fn log(&self) -> Result<HostLog<'a>, Error> {
        let clap_host_log = self
            .get_extension_ptr(CLAP_EXT_LOG)
            .ok_or(Error::ExtensionNotFound("log"))?;

        // SAFETY: We just checked if the pointer to clap_log is non-null. We return a
        // reference to it for the lifetime of Host.
        let clap_host_log: &clap_host_log = unsafe { &*clap_host_log.cast() };

        let _ = clap_host_log.log.ok_or(Error::Callback("log"))?;

        // SAFETY: We just checked if the pointer to clap_host_log, and all its methods,
        // are non-null.
        Ok(unsafe { HostLog::new_unchecked(self.host, clap_host_log) })
    }

    pub fn params(&self) -> Result<HostParams<'a>, Error> {
        let clap_host_params = self
            .get_extension_ptr(CLAP_EXT_PARAMS)
            .ok_or(Error::ExtensionNotFound("params"))?;

        // SAFETY: We just checked if the pointer to clap_host_params is non-null. We
        // return a reference to it for the lifetime of Host.
        let clap_host_params: &clap_host_params = unsafe { &*clap_host_params.cast() };

        let _ = clap_host_params.rescan.ok_or(Error::Callback("rescan"))?;
        let _ = clap_host_params.clear.ok_or(Error::Callback("clear"))?;
        let _ = clap_host_params
            .request_flush
            .ok_or(Error::Callback("request_flush"))?;

        // SAFETY: We just checked if the pointer to clap_host_params, and all its
        // methods are non-null.
        Ok(unsafe { HostParams::new_unchecked(self.host, clap_host_params) })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    ExtensionNotFound(&'static str),
    Callback(&'static str),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::ExtensionNotFound(name) => write!(f, "extension not found: {name}"),
            Error::Callback(name) => write!(f, "extension callback not found: {name}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<Error> for crate::Error {
    fn from(value: Error) -> Self {
        Self::Host(value)
    }
}
