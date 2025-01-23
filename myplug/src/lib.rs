use clap::Plugin;

#[derive(Default)]
pub struct MyPlug;

impl Plugin for MyPlug {
    const ID: &'static str = "";
    const NAME: &'static str = "";
    const VENDOR: &'static str = "";

    fn init(&mut self) -> Result<(), clap::Error> {
        todo!()
    }

    fn activate(
        &mut self,
        sample_rate: f64,
        min_frames: u32,
        max_frames: u32,
    ) -> Result<(), clap::Error> {
        todo!()
    }

    fn deactivate(&mut self) {
        todo!()
    }

    fn start_processing(&mut self) -> Result<(), clap::Error> {
        todo!()
    }

    fn stop_processing(&mut self) {
        todo!()
    }

    fn reset(&mut self) {
        todo!()
    }
}
