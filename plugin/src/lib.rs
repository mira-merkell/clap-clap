static PLUGIN_DESC: clap::PluginDestriptor = clap::PluginDestriptor {};

pub struct Plugin;

impl clap::Plugin for Plugin {}

struct Factory;

static FACTORY: Factory = Factory {};

impl clap::Factory for Factory {
    type Plugin = Plugin;
    fn plugin_count(&self) -> u32 {
        1
    }

    fn plugin_descriptor(&self) -> &'static clap::PluginDestriptor {
        &PLUGIN_DESC
    }

    fn create_plugin(&self) -> Box<Self::Plugin> {
        Box::new(Self::Plugin {})
    }
}

struct Entry;

impl clap::Entry for Entry {
    const CLAP_VERSION: (u32, u32, u32) = clap::CLAP_VERSION;

    type Factory = Factory;

    fn init(_plugin_path: &str) -> Result<(), clap::Error> {
        Ok(())
    }

    fn deinit() {}

    fn get_factory(_id: &str) -> &'static Self::Factory {
        &FACTORY
    }
}

clap::entry! { Entry }
