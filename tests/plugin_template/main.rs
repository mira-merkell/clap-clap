#[path = "../../examples/plugin_template.rs"]
mod plugin;

use plugin::_clap_entry::clap_entry as ENTRY;

mod fixtures;

mod initialize {
    use clap_clap::ffi::CLAP_LOG_INFO;

    use crate::fixtures::{TestHostConfig, TestPlugin};

    #[test]
    fn create() {
        let host = TestHostConfig::default().build();

        let _ = TestPlugin::new(&host);
    }

    #[should_panic]
    #[test]
    fn init_no_log() {
        let host = TestHostConfig::default().build();

        let plugin = TestPlugin::new(&host);
        assert!(unsafe { plugin.init.unwrap()(&*plugin) });
    }

    #[test]
    fn init() {
        let host = TestHostConfig {
            impl_ext_log: true,
            ..Default::default()
        }
        .build();

        let plugin = TestPlugin::new(&host);
        assert!(unsafe { plugin.init.unwrap()(&*plugin) });

        let buf = host.ext_log_messages.lock().unwrap();
        assert_eq!(buf[0].0, CLAP_LOG_INFO);
        assert_eq!(buf[0].1.as_c_str(), c"hello, sonic world");
    }

    #[test]
    fn activate() {
        let host = TestHostConfig {
            impl_ext_log: true,
            ..Default::default()
        }
        .build();

        let plugin = TestPlugin::new(&host);
        assert!(unsafe { plugin.init.unwrap()(&*plugin) });
        assert!(unsafe { plugin.activate.unwrap()(&*plugin, 0.0, 0, 0) });
        unsafe { plugin.deactivate.unwrap()(&*plugin) };
    }
}
