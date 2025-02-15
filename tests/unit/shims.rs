#![allow(unused)]

macro_rules! impl_shim {
    ($NewType:tt, $ClapType:ty) => {
        #[derive(Debug)]
        pub struct $NewType($ClapType);

        impl $NewType {
            pub const fn as_ref(&self) -> &$ClapType {
                &self.0
            }
        }

        unsafe impl Send for $NewType {}
        unsafe impl Sync for $NewType {}
    };
}

pub mod events {
    pub mod input_events {
        use std::ptr::{null, null_mut};

        use clap_clap::ffi::{clap_event_header, clap_input_events};

        extern "C-unwind" fn size(_: *const clap_input_events) -> u32 {
            0
        }

        extern "C-unwind" fn get(_: *const clap_input_events, _: u32) -> *const clap_event_header {
            null()
        }

        impl_shim!(ClapInputEvents, clap_input_events);

        pub static SHIM_CLAP_INPUT_EVENTS: ClapInputEvents = ClapInputEvents(clap_input_events {
            ctx: null_mut(),
            size: Some(size),
            get: Some(get),
        });
    }

    pub mod output_events {
        use std::ptr::null_mut;

        use clap_clap::ffi::{clap_event_header, clap_output_events};

        extern "C-unwind" fn try_push(
            _: *const clap_output_events,
            _: *const clap_event_header,
        ) -> bool {
            false
        }

        impl_shim!(ClapOutputEvents, clap_output_events);

        pub static SHIM_CLAP_OUTPUT_EVENTS: ClapOutputEvents =
            ClapOutputEvents(clap_output_events {
                ctx: null_mut(),
                try_push: Some(try_push),
            });
    }
}

pub mod host {
    use std::{
        ffi::{c_char, c_void},
        ptr::{null, null_mut},
    };

    use clap_clap::ffi::{CLAP_VERSION, clap_host};

    extern "C-unwind" fn get_extension(_: *const clap_host, _: *const c_char) -> *const c_void {
        null()
    }

    extern "C-unwind" fn request_restart(_: *const clap_host) {}

    extern "C-unwind" fn request_process(_: *const clap_host) {}

    extern "C-unwind" fn request_callback(_: *const clap_host) {}

    impl_shim!(ClapHost, clap_host);

    pub static SHIM_CLAP_HOST: ClapHost = ClapHost(clap_host {
        clap_version: CLAP_VERSION,
        host_data: null_mut(),
        name: c"".as_ptr(),
        vendor: c"".as_ptr(),
        url: c"".as_ptr(),
        version: c"".as_ptr(),
        get_extension: Some(get_extension),
        request_restart: Some(request_restart),
        request_process: Some(request_process),
        request_callback: Some(request_callback),
    });
}

pub mod audio_buffer {
    use std::ptr::null_mut;

    use clap_clap::ffi::clap_audio_buffer;

    impl_shim!(ClapAudioBuffer, clap_audio_buffer);

    pub static SHIM_CLAP_AUDIO_BUFFER: ClapAudioBuffer = ClapAudioBuffer(clap_audio_buffer {
        data32: null_mut(),
        data64: null_mut(),
        channel_count: 0,
        latency: 0,
        constant_mask: 0,
    });
}

pub mod process {
    use std::ptr::{null, null_mut};

    use clap_clap::ffi::clap_process;

    use crate::shims::{
        audio_buffer::SHIM_CLAP_AUDIO_BUFFER,
        events::{input_events::SHIM_CLAP_INPUT_EVENTS, output_events::SHIM_CLAP_OUTPUT_EVENTS},
    };

    impl_shim!(ClapProcess, clap_process);

    pub static SHIM_CLAP_PROCESS: ClapProcess = ClapProcess(clap_process {
        steady_time: 0,
        frames_count: 0,
        transport: null(),
        audio_inputs: SHIM_CLAP_AUDIO_BUFFER.as_ref(),
        audio_outputs: null_mut(),
        audio_inputs_count: 1,
        audio_outputs_count: 0,
        in_events: SHIM_CLAP_INPUT_EVENTS.as_ref(),
        out_events: SHIM_CLAP_OUTPUT_EVENTS.as_ref(),
    });
}

pub mod plugin {
    use clap_clap::{Error, plugin::Plugin};

    #[derive(Default)]
    pub struct ShimPlugin;

    impl Plugin for ShimPlugin {
        type AudioThread = ();
        type Extensions = ();
        const ID: &'static str = "";
        const NAME: &'static str = "";

        fn activate(&mut self, _: f64, _: u32, _: u32) -> Result<Self::AudioThread, Error> {
            Ok(())
        }
    }
}
