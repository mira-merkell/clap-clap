use crate::factory::FactoryHost;

pub struct Host<'a> {
    _clap_host: &'a FactoryHost,
}

impl<'a> Host<'a> {
    pub(crate) fn new(clap_host: &'a FactoryHost) -> Self {
        Self { _clap_host: clap_host }
    }
}
