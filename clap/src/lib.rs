pub enum Error {}

pub trait Plugin: Default {
    const ID: &'static str;
    const NAME: &'static str;
    const VENDOR: &'static str;

    fn init(&mut self) -> Result<(), Error>;
    fn activate(&mut self, sample_rate: f64, min_frames: u32, max_frames: u32)
    -> Result<(), Error>;
    fn deactivate(&mut self);
    fn start_processing(&mut self) -> Result<(), Error>;
    fn stop_processing(&mut self);
    fn reset(&mut self);
}
