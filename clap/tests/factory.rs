use clap::Error;
use clap::factory::{Factory, FactoryHost, FactoryPluginDescriptor};
use clap::host::Host;
use clap::plugin::{AudioThread, Plugin};
use clap::process::Status::Continue;
use std::ffi::CString;
use std::ptr::{NonNull, null, null_mut};
use std::sync::Arc;

#[derive(Default)]
struct TestPlug;

impl AudioThread<TestPlug> for () {
    fn process(
        &mut self,
        _: &mut clap::process::Process<'_>,
    ) -> Result<clap::process::Status, Error> {
        Ok(Continue)
    }
}

impl Plugin for TestPlug {
    const ID: &'static str = "test.plugin";
    type AudioThread = ();
    type Extensions = ();

    fn init(&mut self, host: Arc<Host>) -> Result<(), Error> {
        host.get_extension().log()?.warning("this is a test")?;
        Ok(())
    }

    fn activate(&mut self, _: f64, _: usize, _: usize) -> Result<Self::AudioThread, Error> {
        Ok(())
    }
}

struct TestHost {
    log: Vec<CString>,
}

#[test]
#[should_panic(expected = "called `Result::unwrap()` on an `Err` value: PluginIdNotFound")]
fn factory_plug_wrong_id() {
    let factory = Factory::new(vec![Box::new(
        FactoryPluginDescriptor::<TestPlug>::allocate(),
    )]);

    let mut host = clap_sys::clap_host {
        clap_version: clap_sys::CLAP_VERSION,
        host_data: null_mut(),
        name: null(),
        vendor: null(),
        url: null(),
        version: null(),
        get_extension: None,
        request_restart: None,
        request_process: None,
        request_callback: None,
    };
    factory
        .boxed_clap_plugin(
            c"testxxxn",
            FactoryHost::new(unsafe { NonNull::new_unchecked(&raw mut host) }),
        )
        .unwrap();
}
