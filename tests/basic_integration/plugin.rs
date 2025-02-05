use clap_clap::plugin::Plugin;

#[derive(Default, Debug)]
pub struct Dummy;

impl Plugin for Dummy {
    type AudioThread = ();
    type Extensions = ();

    const ID: &'static str = "dummy";
    const NAME: &'static str = "Dummy";

    fn activate(&mut self, _: f64, _: u32, _: u32) -> Result<Self::AudioThread, clap_clap::Error> {
        Ok(())
    }
}
