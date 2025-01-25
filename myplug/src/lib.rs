use clap::{Error, Plugin, Extensions, Process};

#[derive(Default)]
pub struct MyPlug;

impl Plugin for MyPlug {
    const ID: &'static str = "com.plugggs.my_plug";
    const NAME: &'static str = "MyPlug";
    type Extensions = ();

    fn process(
        &mut self,
        _process: &mut Process,
    ) -> Result<clap::process::Status, clap::process::Error> {
        Ok(clap::process::Status::Continue)
    }
}

clap::entry!(MyPlug, MyPlug);
