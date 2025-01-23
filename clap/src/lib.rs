pub enum Error {}

pub trait Plugin: Default + Sync + Send {
    const ID: &'static str;
    const NAME: &'static str = "";
    const VENDOR: &'static str = "";
    const URL: &'static str = "";
    const MANUAL_URL: &'static str = "";
    const SUPPORT_URL: &'static str = "";
    const VERSION: &'static str = "0.0.0";
    const DESCRIPTION: &'static str = "";
    const FEATURES: &'static str = "fx stereo";

    fn init(&mut self) -> Result<(), Error> {
        Ok(())
    }
    
    fn activate(
        &mut self,
        sample_rate: f64,
        min_frames: u32,
        max_frames: u32,
    ) -> Result<(), Error> {
        Ok(())
    }
    
    fn deactivate(&mut self) {}
    
    fn start_processing(&mut self) -> Result<(), Error> {
        Ok(())
    }
    
    fn stop_processing(&mut self) {}
    
    fn process(&mut self, process: &mut Process) -> Result<(), Error> {
        Ok(())
    }
    
    fn reset(&mut self) {}
    
    fn on_main_thread(&self) {}
}

pub struct Process;
