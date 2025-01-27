use clap_sys::clap_host;
use std::ptr::NonNull;

/// This type is public to make it be visible from within clap::entry! macro.
pub struct ClapHost {
    _host: NonNull<clap_host>,
}

impl ClapHost {
    pub const fn new(host: NonNull<clap_host>) -> Self {
        Self { _host: host }
    }
}

pub struct Host<'a> {
    host: &'a ClapHost,
}
