use crate::factory::FactoryHost;

pub struct Host<'a> {
    _host: &'a FactoryHost,
}

impl<'a> Host<'a> {
    pub(crate) fn new(host: &'a FactoryHost) -> Self {
        Self { _host: host }
    }
}
