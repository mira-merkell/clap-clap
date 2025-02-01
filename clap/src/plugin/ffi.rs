use std::{
    ffi::{CStr, c_char, c_void},
    mem,
    ptr::{NonNull, null},
};

use clap_sys::{
    CLAP_EXT_AUDIO_PORTS, CLAP_PROCESS_ERROR, clap_plugin, clap_process, clap_process_status,
};

use crate::{
    plugin::{AudioThread, ClapPlugin, Plugin, Runtime},
    process::Process,
};

#[allow(warnings, unused)]
extern "C" fn init<P: Plugin>(plugin: *const clap_plugin) -> bool {
    if plugin.is_null() {
        return false;
    }
    // Safety:
    // We just checked that the pointer is non-null and the plugin
    // has been obtained from host, and is tied to type P.
    let mut clap_plugin = unsafe { ClapPlugin::<P>::new(plugin) };

    // Safety:
    // This function is called on the main thread during the initialization.
    // It is guaranteed that we are the only function accessing the entire runtime.
    let runtime = unsafe { clap_plugin.runtime() };
    let host = runtime.host.clone();

    runtime.plugin.init(host).is_ok()
}

extern "C" fn destroy<P: Plugin>(plugin: *const clap_plugin) {
    if plugin.is_null() {
        return;
    }
    // Safety:
    // We just checked that the pointer is non-null and the plugin
    // has been obtained from host and is tied to type P.
    let clap_plugin = unsafe { ClapPlugin::<P>::new(plugin) };

    // Safety:
    // This function is called on the main thread to destroy the plugin.
    // It is guaranteed that we are the only function accessing the runtime now.
    // So retaking the ownership of the runtime is safe.
    let runtime = unsafe { Runtime::from_clap_plugin(clap_plugin) };

    drop(runtime)
}

extern "C" fn activate<P: Plugin>(
    plugin: *const clap_plugin,
    sample_rate: f64,
    min_frames_count: u32,
    max_frames_count: u32,
) -> bool {
    if plugin.is_null() {
        return false;
    }
    // Safety:
    // We just checked that the pointer is non-null and the plugin
    // has been obtained from host and is tied to type P.
    let mut clap_plugin = unsafe { ClapPlugin::<P>::new(plugin) };

    // Safety:
    // This function is called on the main thread. It is guaranteed that we are the
    // only function accessing runtime now, because the audio thread hasn't
    // started yet. So a mutable reference to runtime is safe.
    let runtime = unsafe { clap_plugin.runtime() };
    let (plugin, audio_thread) = (&mut runtime.plugin, &mut runtime.audio_thread);

    let should_be_none = mem::replace(
        audio_thread,
        plugin
            .activate(sample_rate, min_frames_count, max_frames_count)
            .ok(),
    );

    should_be_none.is_none() && audio_thread.is_some()
}

extern "C" fn deactivate<P: Plugin>(plugin: *const clap_plugin) {
    if plugin.is_null() {
        return;
    }
    // Safety:
    // We just checked that the pointer is non-null and the plugin
    // has been obtained from host and is tied to type P.
    let mut clap_plugin = unsafe { ClapPlugin::<P>::new(plugin) };

    // Safety:
    // This function is called on the main thread.
    // It is guaranteed that we are the only function accessing runtime.audio_thread
    // now, and we are on the main thread -- so it is guaranteed we are the only
    // function that has access to the entire runtime now.
    // So the mutable reference to the entire runtime for the duration of this call
    // is safe.
    let runtime = unsafe { clap_plugin.runtime() };

    if let Some(audio_thread) = runtime.audio_thread.take() {
        audio_thread.deactivate(&mut runtime.plugin);
    }
}

extern "C" fn start_processing<P: Plugin>(plugin: *const clap_plugin) -> bool {
    if plugin.is_null() {
        return false;
    }
    // Safety:
    // We just checked that the pointer is non-null and the plugin
    // has been obtained from host and is tied to type P.
    let mut clap_plugin = unsafe { ClapPlugin::<P>::new(plugin) };

    // Safety:
    // This function is called on the audio thread.  It is guaranteed that
    // we are the only function accessing audio_thread now. So a mutable reference
    // to audio_thread for the duration of this call is safe.
    let Some(audio_thread) = (unsafe { clap_plugin.audio_thread() }) else {
        return false;
    };

    audio_thread.start_processing().is_ok()
}

extern "C" fn stop_processing<P: Plugin>(plugin: *const clap_plugin) {
    if plugin.is_null() {
        return;
    }
    // Safety:
    // We just checked that the pointer is non-null and the plugin
    // has been obtained from host and is tied to type P.
    let mut clap_plugin = unsafe { ClapPlugin::<P>::new(plugin) };

    // Safety:
    // This function is called on the audio thread.  It is guaranteed that
    // we are the only function accessing audio_thread now. So a mutable reference
    // to audio_thread for the duration of this call is safe.
    let Some(audio_thread) = (unsafe { clap_plugin.audio_thread() }) else {
        return;
    };

    audio_thread.stop_processing();
}

extern "C" fn reset<P: Plugin>(plugin: *const clap_plugin) {
    if plugin.is_null() {
        return;
    }
    // Safety:
    // We just checked that the pointer is non-null and the plugin
    // has been obtained from host and is tied to type P.
    let mut clap_plugin = unsafe { ClapPlugin::<P>::new(plugin) };

    // Safety:
    // This function is called on the audio thread.  It is guaranteed that
    // we are the only function accessing audio_thread now. So a mutable reference
    // to audio_thread for the duration of this call is safe.
    let Some(audio_thread) = (unsafe { clap_plugin.audio_thread() }) else {
        return;
    };

    audio_thread.reset();
}

#[allow(warnings, unused)]
extern "C" fn process<P: Plugin>(
    plugin: *const clap_plugin,
    process: *const clap_process,
) -> clap_process_status {
    if plugin.is_null() {
        return CLAP_PROCESS_ERROR;
    }
    // Safety:
    // We just checked that the pointer is non-null and the plugin
    // has been obtained from host, and is tied to type P.
    let mut clap_plugin = unsafe { ClapPlugin::<P>::new(plugin) };

    // Safety:
    // This function is called on the audio thread.  It is guaranteed that
    // we are the only function accessing audio_thread now. So a mutable reference
    // to audio_thread for the duration of this call is safe.
    let Some(audio_thread) = (unsafe { clap_plugin.audio_thread() }) else {
        return CLAP_PROCESS_ERROR;
    };

    if process.is_null() {
        return CLAP_PROCESS_ERROR;
    }
    // Safety:
    // The pointer to clap_process is guaranteed to be valid and pointing
    // to an exclusive struct for the duration of this call.
    // So a mutable reference to process is safe.
    let process = unsafe { &mut *(process as *mut _) };
    let process = &mut unsafe { Process::new(NonNull::new_unchecked(process)) };
    audio_thread
        .process(process)
        .map(Into::into)
        .unwrap_or(CLAP_PROCESS_ERROR)
}

#[allow(warnings, unused)]
extern "C" fn get_extension<P: Plugin>(
    plugin: *const clap_plugin,
    id: *const c_char,
) -> *const c_void {
    if plugin.is_null() {
        return null();
    }
    // Safety:
    // We just checked that the pointer is non-null and the plugin
    // has been obtained from host and is tied to type P.
    let mut clap_plugin = unsafe { ClapPlugin::<P>::new(plugin) };

    // Safety:
    // The plugin id is a valid C string obtained from the host.  The C string
    // lifetime extends for the duration of this function call.
    let id = unsafe { CStr::from_ptr(id) };

    // Safety:
    // This function must be thread-safe.
    // We're accessing only runtime.plugin_extensions that is guarded by a Mutex.
    let mutex = clap_plugin.plugin_extensions();
    let Ok(extensions) = mutex.lock() else {
        return null();
    };

    if id == CLAP_EXT_AUDIO_PORTS {
        if let Some(audio_ports) = &extensions.audio_ports {
            return &raw const *audio_ports as *const c_void;
        }
    }

    null()
}

extern "C" fn on_main_thread<P: Plugin>(plugin: *const clap_plugin) {
    if plugin.is_null() {
        return;
    }
    // Safety:
    // We just checked that the pointer is non-null and the plugin
    // has been obtained from host and is tied to type P.
    let mut clap_plugin = unsafe { ClapPlugin::<P>::new(plugin) };

    // Safety:
    // This function is called on the main thread.
    // It is guaranteed that we are the only function accessing the plugin now.
    // So the mutable reference to plugin for the duration of this call is safe.
    let plugin = unsafe { clap_plugin.plugin() };

    plugin.on_main_thread();
}

pub(crate) fn box_clap_plugin<P: Plugin>(data: Runtime<P>) -> Box<clap_plugin> {
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
