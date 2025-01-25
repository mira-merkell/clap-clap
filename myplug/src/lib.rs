use clap::ext::AudioPorts;
use clap::{Error, Plugin, Process};

#[derive(Default)]
struct MyPlug(Vec<[f32; 2]>);

struct Extensions {}

impl clap::Extensions<MyPlug> for Extensions {
    fn audio_ports() -> Option<impl AudioPorts<MyPlug>> {
        Some(clap::ext::audio_ports::StereoPort)
    }
}

impl Plugin for MyPlug {
    const ID: &'static str = "com.plugggs.my_plug";
    const NAME: &'static str = "MyPlug";
    type E = Extensions;

    fn activate(
        &mut self,
        _sample_rate: f64,
        _min_frames: u32,
        max_frames: u32,
    ) -> Result<(), Error> {
        self.0 = vec![[0.0, 0.0]; max_frames as usize];
        
        Ok(())
    }

    fn process(
        &mut self,
        process: &mut Process,
    ) -> Result<clap::process::Status, clap::process::Error> {
        let in_port = process.audio_input(0).unwrap();
        for k in 0..=1 {
            let ch = in_port.channel(k).unwrap();
            for (i, sample) in ch.iter().enumerate() {
                self.0[i][k] = *sample;
            }
        }

        let mut out_port = process.audio_output(0).unwrap();
        for k in 0..=1 {
            let ch = out_port.channel_mut(k).unwrap();
            for (i, sample) in ch.iter_mut().enumerate() {
                *sample = self.0[i][k];
            }
        }

        Ok(clap::process::Status::Continue)
    }
}

clap::entry!(MyPlug);
