use crate::plugin::{ClapPluginData, wrap_clap_plugin_from_host, Plugin};
use crate::process::Process;
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
    let wrap = unsafe { wrap_clap_plugin_from_host::<P>(plugin) };
    if id.is_null() {
        return null();
    }
    let id = unsafe { CStr::from_ptr(id) };
    if id == CLAP_EXT_AUDIO_PORTS && wrap.clap_plugin().plugin_extensions.audio_ports.is_some() {
        if let Some(audio_ports) = &wrap.clap_plugin().plugin_extensions.audio_ports {
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

pub(crate) fn box_clap_plugin<P: Plugin>(data: ClapPluginData<P>) -> Box<clap_plugin> {
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
