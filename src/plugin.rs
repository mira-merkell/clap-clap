use std::{
    ffi::NulError,
    fmt::Display,
    iter::empty,
    marker::PhantomData,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};

use crate::{
    ext::{Extensions, audio_ports::PluginAudioPorts},
    ffi::clap_plugin,
    host::Host,
    process,
    process::{Process, Status::Continue},
};

pub trait Plugin: Default + Extensions<Self> {
    type AudioThread: AudioThread<Self>;

    const ID: &'static str;
    const NAME: &'static str;
    const VENDOR: &'static str = "";
    const URL: &'static str = "";
    const MANUAL_URL: &'static str = "";
    const SUPPORT_URL: &'static str = "";
    const VERSION: &'static str = "";
    const DESCRIPTION: &'static str = "";

    /// Plugin features as an arbitrary list of keywords.
    ///
    /// They can be matched by the host indexer and used to classify the plugin.
    /// For some standard features, see module: [`plugin_features`].
    ///
    /// The default implementation returns an empty iterator.
    ///
    /// # Example
    ///
    /// ```no_compile,rust
    /// fn features() -> impl Iterator<Item = &'static str> {
    ///     "instrument stereo sampler".split_whitespace()
    /// }
    /// ```
    ///
    /// [`plugin_features`]: crate::plugin_features
    fn features() -> impl Iterator<Item = &'static str> {
        empty()
    }

    #[allow(unused_variables)]
    fn init(&mut self, host: Arc<Host>) -> Result<(), crate::Error> {
        Ok(())
    }

    fn activate(
        &mut self,
        sample_rate: f64,
        min_frames: u32,
        max_frames: u32,
    ) -> Result<Self::AudioThread, crate::Error>;

    fn on_main_thread(&mut self) {}
}

pub trait AudioThread<P: Plugin>: Send + Sync + Sized {
    fn start_processing(&mut self) -> Result<(), crate::Error> {
        Ok(())
    }

    fn stop_processing(&mut self) {}

    fn process(&mut self, process: &mut Process) -> Result<process::Status, crate::Error>;

    fn reset(&mut self) {}

    #[allow(unused_variables)]
    fn deactivate(self, plugin: &mut P) {}
}

impl<P: Plugin> AudioThread<P> for () {
    fn process(&mut self, _: &mut Process) -> Result<process::Status, crate::Error> {
        Ok(Continue)
    }
}

struct PluginExtensions<P> {
    audio_ports: Option<PluginAudioPorts<P>>,
    latency: Option<PluginLatency<P>>,
    note_ports: Option<PluginNotePorts<P>>,
    params: Option<PluginParams<P>>,
}

impl<P: Plugin> PluginExtensions<P> {
    fn new() -> Self {
        Self {
            audio_ports: <P as Extensions<P>>::audio_ports().map(PluginAudioPorts::new),
            latency: <P as Extensions<P>>::latency().map(PluginLatency::new),
            note_ports: <P as Extensions<P>>::note_ports().map(PluginNotePorts::new),
            params: <P as Extensions<P>>::params().map(PluginParams::new),
        }
    }
}

pub(crate) struct Runtime<P: Plugin> {
    pub(crate) active: AtomicBool,
    pub(crate) audio_thread: Option<P::AudioThread>,
    pub(crate) descriptor: PluginDescriptor,
    pub(crate) host: Arc<Host>,
    pub(crate) plugin: P,
    plugin_extensions: Mutex<PluginExtensions<P>>,
}

impl<P: Plugin> Runtime<P> {
    pub(crate) fn initialize(host: Arc<Host>) -> Result<Self, Error> {
        Ok(Self {
            active: AtomicBool::new(false),
            descriptor: PluginDescriptor::new::<P>()?,
            plugin: P::default(),
            audio_thread: None,
            host,
            plugin_extensions: Mutex::new(PluginExtensions::new()),
        })
    }

    pub(crate) fn into_clap_plugin(self) -> ClapPlugin<P> {
        // Safety:
        // The leaked (via Box::into_raw) pointer satisfies requirements
        // for a safe call to ClapPlugin::new():
        // 1. it is non-null
        // 2. it represents a valid clap_plugin tied to type P.
        unsafe { ClapPlugin::new_unchecked(Box::into_raw(ffi::box_clap_plugin(self))) }
    }

    /// Retake ownership of the runtime from the pointer to  clap_plugin.
    ///
    /// # Safety:
    ///
    /// The caller must assure it's only them who have access to the entire
    /// runtime: both main thread and the audio thread.
    /// This can requirement can be met during plugin initialization and
    /// destruction.
    unsafe fn from_clap_plugin(clap_plugin: ClapPlugin<P>) -> Self {
        let plugin_data = unsafe { clap_plugin.clap_plugin() }.plugin_data as *mut _;
        // Safety:
        // We can transmute the pointer to plugin_data like this, because:
        // 1. We have exclusive reference to it.
        // 2. We know the pointer's real type because of the constraints put on the
        //    constructor of ClapPlugin.
        // 3. We know that the pointer was initially leaked with Box::into_raw().
        *unsafe { Box::from_raw(plugin_data) }
    }
}

#[doc(hidden)]
/// Safe wrapper around a pointer to clap_plugin.
pub struct ClapPlugin<P: Plugin> {
    clap_plugin: *const clap_plugin,
    _marker: PhantomData<P>,
}

impl<P: Plugin> ClapPlugin<P> {
    /// # Safety
    ///
    /// 1. The user must assure the pointer to plugin is non-null.
    /// 2. The pointer must point to a valid clap_plugin structure tied to the
    ///    plugin type P, and living in the host.
    /// 3. There must be only one instance of ClapPlugin for a given pointer.
    ///
    /// Typically, a valid pointer comes from the host calling the plugin's
    /// methods, or from Runtime::into_clap_plugin()
    pub const unsafe fn new_unchecked(clap_plugin: *const clap_plugin) -> Self {
        Self {
            clap_plugin,
            _marker: PhantomData,
        }
    }

    /// # Safety
    ///
    /// The caller must ensure that the wrapped pointer to clap_plugin is
    /// dereferencable and that Rust aliasing rules of shared references hold.
    #[doc(hidden)]
    pub const unsafe fn clap_plugin<'a>(&self) -> &'a clap_plugin {
        // SAFETY: ClapPlugin constructor guarantees that dereferencing the inner
        // pointer is safe.
        unsafe { &*self.clap_plugin }
    }

    pub(crate) const fn into_inner(self) -> *const clap_plugin {
        self.clap_plugin
    }

    /// Obtain a mutable reference to the entire runtime.
    ///
    /// # Safety
    ///
    /// The caller must assure they're the only ones who access the runtime.
    pub(crate) const unsafe fn runtime(&mut self) -> &mut Runtime<P> {
        let runtime: *mut Runtime<P> = unsafe { *self.clap_plugin }.plugin_data as *mut _;
        unsafe { &mut *runtime }
    }

    pub fn is_active(&self) -> bool {
        let runtime: *mut Runtime<P> = unsafe { *self.clap_plugin }.plugin_data as *mut _;
        unsafe { (*runtime).active.load(Ordering::Acquire) }
    }

    /// Obtain a mutable reference to plugin.
    ///
    /// # Safety
    ///
    /// The caller must assure they're the only ones who access the plugin.
    pub const unsafe fn plugin(&mut self) -> &mut P {
        let runtime: *mut Runtime<P> = unsafe { *self.clap_plugin }.plugin_data as *mut _;
        unsafe { &mut (*runtime).plugin }
    }

    /// Obtain a mutable reference to audio thread.
    ///
    /// # Safety
    ///
    /// The caller must assure they're the only ones who access the
    /// audio_thread.
    pub const unsafe fn audio_thread(&mut self) -> Option<&mut P::AudioThread> {
        let runtime: *mut Runtime<P> = unsafe { *self.clap_plugin }.plugin_data as *mut _;
        unsafe { &mut (*runtime).audio_thread }.as_mut()
    }

    /// Obtain a mutex to plugin extensions.
    const fn plugin_extensions(&mut self) -> &Mutex<PluginExtensions<P>> {
        let runtime: *mut Runtime<P> = unsafe { *self.clap_plugin }.plugin_data as *mut _;
        unsafe { &(*runtime).plugin_extensions }
    }
}

mod desc {
    use std::{
        ffi::{CStr, CString, c_char},
        ptr::null,
    };

    use crate::{
        ffi::{CLAP_VERSION, clap_plugin_descriptor},
        plugin::{Error, Plugin},
    };

    #[allow(dead_code)]
    pub struct PluginDescriptor {
        clap_plugin_descriptor: clap_plugin_descriptor,
        clap_features: Box<[*const c_char]>,

        id: CString,
        name: CString,
        vendor: CString,
        url: CString,
        manual_url: CString,
        support_url: CString,
        version: CString,
        description: CString,
        features: Box<[CString]>,
    }

    impl PluginDescriptor {
        pub fn new<P: Plugin>() -> Result<Self, Error> {
            let id = CString::new(P::ID)?;
            let name = CString::new(P::NAME)?;
            let vendor = CString::new(P::VENDOR)?;
            let url = CString::new(P::URL)?;
            let manual_url = CString::new(P::MANUAL_URL)?;
            let support_url = CString::new(P::SUPPORT_URL)?;
            let version = CString::new(P::VERSION)?;
            let description = CString::new(P::DESCRIPTION)?;

            let features: Box<[CString]> =
                P::features().map(CString::new).collect::<Result<_, _>>()?;
            let mut clap_features: Vec<*const c_char> =
                features.iter().map(|s| s.as_c_str().as_ptr()).collect();
            clap_features.push(null());
            let clap_features = clap_features.into_boxed_slice();

            Ok(Self {
                clap_plugin_descriptor: clap_plugin_descriptor {
                    clap_version: CLAP_VERSION,
                    id: id.as_c_str().as_ptr(),
                    name: name.as_c_str().as_ptr(),
                    vendor: vendor.as_c_str().as_ptr(),
                    url: url.as_c_str().as_ptr(),
                    manual_url: manual_url.as_c_str().as_ptr(),
                    support_url: support_url.as_c_str().as_ptr(),
                    version: version.as_c_str().as_ptr(),
                    description: description.as_c_str().as_ptr(),
                    features: clap_features.as_ptr(),
                },
                clap_features,
                id,
                name,
                vendor,
                url,
                manual_url,
                support_url,
                version,
                description,
                features,
            })
        }

        pub fn plugin_id(&self) -> &CStr {
            self.id.as_c_str()
        }

        pub const fn clap_plugin_descriptor(&self) -> &clap_plugin_descriptor {
            &self.clap_plugin_descriptor
        }
    }
}

#[doc(hidden)]
pub use desc::PluginDescriptor;

use crate::ext::{latency::PluginLatency, note_ports::PluginNotePorts, params::PluginParams};

mod ffi {
    use std::{
        ffi::{CStr, c_char, c_void},
        mem,
        ptr::{NonNull, null},
        sync::atomic::Ordering,
    };

    use crate::{
        ffi::{
            CLAP_EXT_AUDIO_PORTS, CLAP_EXT_LATENCY, CLAP_EXT_NOTE_PORTS, CLAP_EXT_PARAMS,
            CLAP_PROCESS_ERROR, clap_plugin, clap_process, clap_process_status,
        },
        plugin::{AudioThread, ClapPlugin, Plugin, Runtime},
        process::Process,
    };

    #[allow(warnings, unused)]
    unsafe extern "C-unwind" fn init<P: Plugin>(plugin: *const clap_plugin) -> bool {
        if plugin.is_null() {
            return false;
        }
        // SAFETY: We just checked that the pointer is non-null and the plugin
        // has been obtained from host, and is tied to type P.
        let mut clap_plugin = unsafe { ClapPlugin::<P>::new_unchecked(plugin) };

        // SAFETY: This function is called on the main thread during the initialization.
        // It is guaranteed that we are the only function accessing the entire runtime.
        let runtime = unsafe { clap_plugin.runtime() };
        let host = runtime.host.clone();

        runtime.plugin.init(host).is_ok()
    }

    unsafe extern "C-unwind" fn destroy<P: Plugin>(plugin: *const clap_plugin) {
        if plugin.is_null() {
            return;
        }
        // SAFETY: We just checked that the pointer is non-null and the plugin
        // has been obtained from host and is tied to type P.
        let clap_plugin = unsafe { ClapPlugin::<P>::new_unchecked(plugin) };

        // SAFETY: This function is called on the main thread to destroy the plugin.
        // It is guaranteed that we are the only function accessing the runtime now.
        // So retaking the ownership of the runtime is safe.
        let runtime = unsafe { Runtime::from_clap_plugin(clap_plugin) };

        drop(runtime)
    }

    unsafe extern "C-unwind" fn activate<P: Plugin>(
        plugin: *const clap_plugin,
        sample_rate: f64,
        min_frames_count: u32,
        max_frames_count: u32,
    ) -> bool {
        if plugin.is_null() {
            return false;
        }
        // SAFETY: We just checked that the pointer is non-null and the plugin
        // has been obtained from host and is tied to type P.
        let mut clap_plugin = unsafe { ClapPlugin::<P>::new_unchecked(plugin) };

        // SAFETY: This function is called on the main thread. It is guaranteed that we
        // are the only function accessing runtime now, because the audio thread
        // hasn't started yet. So a mutable reference to runtime is safe.
        let runtime = unsafe { clap_plugin.runtime() };
        let (plugin, audio_thread) = (&mut runtime.plugin, &mut runtime.audio_thread);

        let should_be_none = mem::replace(
            audio_thread,
            plugin
                .activate(sample_rate, min_frames_count, max_frames_count)
                .ok(),
        );

        (should_be_none.is_none() && audio_thread.is_some())
            .then(|| runtime.active.store(true, Ordering::Release))
            .is_some()
    }

    unsafe extern "C-unwind" fn deactivate<P: Plugin>(plugin: *const clap_plugin) {
        if plugin.is_null() {
            return;
        }
        // SAFETY: We just checked that the pointer is non-null and the plugin
        // has been obtained from host and is tied to type P.
        let mut clap_plugin = unsafe { ClapPlugin::<P>::new_unchecked(plugin) };

        // SAFETY: This function is called on the main thread.
        // It is guaranteed that we are the only function accessing runtime.audio_thread
        // now, and we are on the main thread -- so it is guaranteed we are the only
        // function that has access to the entire runtime now.
        // So the mutable reference to the entire runtime for the duration of this call
        // is safe.
        let runtime = unsafe { clap_plugin.runtime() };

        if let Some(audio_thread) = runtime.audio_thread.take() {
            audio_thread.deactivate(&mut runtime.plugin);
        }

        runtime.active.store(false, Ordering::Release)
    }

    unsafe extern "C-unwind" fn start_processing<P: Plugin>(plugin: *const clap_plugin) -> bool {
        if plugin.is_null() {
            return false;
        }
        // SAFETY: We just checked that the pointer is non-null and the plugin
        // has been obtained from host and is tied to type P.
        let mut clap_plugin = unsafe { ClapPlugin::<P>::new_unchecked(plugin) };

        // SAFETY: This function is called on the audio thread.  It is guaranteed that
        // we are the only function accessing audio_thread now. So a mutable reference
        // to audio_thread for the duration of this call is safe.
        let Some(audio_thread) = (unsafe { clap_plugin.audio_thread() }) else {
            return false;
        };

        audio_thread.start_processing().is_ok()
    }

    unsafe extern "C-unwind" fn stop_processing<P: Plugin>(plugin: *const clap_plugin) {
        if plugin.is_null() {
            return;
        }
        // SAFETY: We just checked that the pointer is non-null and the plugin
        // has been obtained from host and is tied to type P.
        let mut clap_plugin = unsafe { ClapPlugin::<P>::new_unchecked(plugin) };

        // SAFETY: This function is called on the audio thread.  It is guaranteed that
        // we are the only function accessing audio_thread now. So a mutable reference
        // to audio_thread for the duration of this call is safe.
        let Some(audio_thread) = (unsafe { clap_plugin.audio_thread() }) else {
            return;
        };

        audio_thread.stop_processing();
    }

    unsafe extern "C-unwind" fn reset<P: Plugin>(plugin: *const clap_plugin) {
        if plugin.is_null() {
            return;
        }
        // SAFETY: We just checked that the pointer is non-null and the plugin
        // has been obtained from host and is tied to type P.
        let mut clap_plugin = unsafe { ClapPlugin::<P>::new_unchecked(plugin) };

        // SAFETY: This function is called on the audio thread.  It is guaranteed that
        // we are the only function accessing audio_thread now. So a mutable reference
        // to audio_thread for the duration of this call is safe.
        let Some(audio_thread) = (unsafe { clap_plugin.audio_thread() }) else {
            return;
        };

        audio_thread.reset();
    }

    #[allow(warnings, unused)]
    unsafe extern "C-unwind" fn process<P: Plugin>(
        plugin: *const clap_plugin,
        process: *const clap_process,
    ) -> clap_process_status {
        if plugin.is_null() {
            return CLAP_PROCESS_ERROR;
        }
        // SAFETY: We just checked that the pointer is non-null and the plugin
        // has been obtained from host, and is tied to type P.
        let mut clap_plugin = unsafe { ClapPlugin::<P>::new_unchecked(plugin) };

        // SAFETY: This function is called on the audio thread.  It is guaranteed that
        // we are the only function accessing audio_thread now. So a mutable reference
        // to audio_thread for the duration of this call is safe.
        let Some(audio_thread) = (unsafe { clap_plugin.audio_thread() }) else {
            return CLAP_PROCESS_ERROR;
        };

        if process.is_null() {
            return CLAP_PROCESS_ERROR;
        }
        // SAFETY: The pointer to clap_process is guaranteed to be valid and pointing
        // to an exclusive struct for the duration of this call.
        // So a mutable reference to process is safe.
        let process = unsafe { &mut *(process as *mut _) };
        let process = &mut unsafe { Process::new_unchecked(NonNull::new_unchecked(process)) };
        audio_thread
            .process(process)
            .map(Into::into)
            .unwrap_or(CLAP_PROCESS_ERROR)
    }

    #[allow(warnings, unused)]
    unsafe extern "C-unwind" fn get_extension<P: Plugin>(
        plugin: *const clap_plugin,
        id: *const c_char,
    ) -> *const c_void {
        if plugin.is_null() {
            return null();
        }
        // SAFETY: We just checked that the pointer is non-null and the plugin
        // has been obtained from host and is tied to type P.
        let mut clap_plugin = unsafe { ClapPlugin::<P>::new_unchecked(plugin) };

        // SAFETY: The plugin id is a valid C string obtained from the host.  The C
        // string lifetime extends for the duration of this function call.
        let id = unsafe { CStr::from_ptr(id) };

        // SAFETY: This function must be thread-safe.
        // We're accessing only runtime.plugin_extensions that is guarded by a Mutex.
        let mutex = clap_plugin.plugin_extensions();
        let Ok(extensions) = mutex.lock() else {
            return null();
        };

        if id == CLAP_EXT_AUDIO_PORTS {
            if let Some(ext) = &extensions.audio_ports {
                return (&raw const *ext).cast();
            }
        } else if id == CLAP_EXT_NOTE_PORTS {
            if let Some(ext) = &extensions.note_ports {
                return (&raw const *ext).cast();
            }
        } else if id == CLAP_EXT_LATENCY {
            if let Some(ext) = &extensions.latency {
                return (&raw const *ext).cast();
            }
        } else if id == CLAP_EXT_PARAMS {
            if let Some(ext) = &extensions.params {
                return (&raw const *ext).cast();
            }
        }

        null()
    }

    unsafe extern "C-unwind" fn on_main_thread<P: Plugin>(plugin: *const clap_plugin) {
        if plugin.is_null() {
            return;
        }
        // SAFETY: We just checked that the pointer is non-null and the plugin
        // has been obtained from host and is tied to type P.
        let mut clap_plugin = unsafe { ClapPlugin::<P>::new_unchecked(plugin) };

        // SAFETY: This function is called on the main thread.
        // It is guaranteed that we are the only function accessing the plugin now.
        // So the mutable reference to plugin for the duration of this call is safe.
        let plugin = unsafe { clap_plugin.plugin() };

        plugin.on_main_thread();
    }

    pub(crate) fn box_clap_plugin<P: Plugin>(data: Runtime<P>) -> Box<clap_plugin> {
        let data = Box::new(data);
        let desc = &raw const *data.descriptor.clap_plugin_descriptor();
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

#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    MissingFields,
    NulError(NulError),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Error::MissingFields => write!(f, "missing fields in plugin description"),
            Error::NulError(_) => write!(f, "null error while converting C string"),
        }
    }
}

impl std::error::Error for Error {}

impl From<NulError> for Error {
    fn from(value: NulError) -> Self {
        Self::NulError(value)
    }
}

impl From<Error> for crate::Error {
    fn from(value: Error) -> Self {
        Self::Plugin(value)
    }
}
