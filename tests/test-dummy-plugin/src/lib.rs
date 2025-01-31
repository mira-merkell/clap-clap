use clap_clap::{
    clap,
    clap::{Error, plugin::Plugin},
};

#[derive(Default)]
struct PluginExport;

impl Plugin for PluginExport {
    const ID: &'static str = "test.dummy.plugin";
    type AudioThread = ();
    type Extensions = ();

    fn activate(&mut self, _: f64, _: usize, _: usize) -> Result<Self::AudioThread, Error> {
        Ok(())
    }
}

clap::entry!(PluginExport);
