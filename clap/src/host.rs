use clap_sys::clap_host;
use std::ptr::NonNull;

pub struct ClapHost {
    _host: NonNull<clap_host>,
}

impl ClapHost {
    pub const fn new(host: NonNull<clap_host>) -> Self {
        Self { _host: host }
    }
}
