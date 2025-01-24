use clap::{Error, Plugin, Process};

#[derive(Default)]
pub struct MyPlug;

impl Plugin for MyPlug {
    const ID: &'static str = "com.plugggs.my_plug";
    const NAME: &'static str = "MyPlug";

    fn process(&mut self, _process: &mut Process) -> Result<(), Error> {
        Ok(())
    }
}

#[derive(Default)]
pub struct MyPlug2;

impl Plugin for MyPlug2 {
    const ID: &'static str = "com.plugggs.my_plug2";
    const NAME: &'static str = "MyPlug2";

    fn process(&mut self, _process: &mut Process) -> Result<(), Error> {
        Ok(())
    }
}

clap::entry!(MyPlug, MyPlug2);
