use clap_clap::{
    clap,
    clap::{Error, plugin::Plugin},
};

#[derive(Default)]
struct Dummy;

impl Plugin for Dummy {
    const ID: &'static str = "dummy";
    type AudioThread = ();
    type Extensions = ();

    fn activate(&mut self, _: f64, _: usize, _: usize) -> Result<Self::AudioThread, Error> {
        Ok(())
    }
}

clap::entry!(Dummy);
