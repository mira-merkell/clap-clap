mod plugin_state {
    use std::{
        io::Write,
        marker::PhantomData,
        sync::{Arc, Mutex},
    };

    use clap_clap::{
        Error,
        ext::{Extensions, state::State},
        plugin::Plugin,
        stream::{IStream, OStream},
    };

    use crate::{
        ext::{Test, TestBed},
        shims::plugin::ShimPlugin,
    };

    #[derive(Debug, Default)]
    struct CheckExtImpl<P> {
        _marker: PhantomData<P>,
    }

    impl<P: Plugin + 'static> Test<P> for CheckExtImpl<P> {
        fn test(self, bed: &mut TestBed<P>) {
            if P::state().is_some() {
                assert!(bed.ext_state.is_some());
            } else {
                assert!(bed.ext_state.is_none());
            }
        }
    }

    #[test]
    fn ext_impl_shim() {
        TestBed::<ShimPlugin>::default().test(CheckExtImpl::default());
    }

    #[derive(Default, Clone)]
    struct TestPlug {
        state: Arc<Mutex<[u8; 5]>>,
    }

    impl Plugin for TestPlug {
        type AudioThread = ();
        const ID: &'static str = "";
        const NAME: &'static str = "";

        fn activate(&mut self, _: f64, _: u32, _: u32) -> Result<Self::AudioThread, Error> {
            Ok(())
        }
    }

    impl Extensions<Self> for TestPlug {
        fn state() -> Option<impl State<Self>> {
            Some(TestState)
        }
    }

    #[derive(Debug)]
    struct TestState;

    impl State<TestPlug> for TestState {
        fn save(plugin: &TestPlug, stream: &mut OStream) -> Result<(), Error> {
            let state = plugin.state.lock().unwrap();
            let n = state.len();

            let mut i = 0;
            while i < n {
                let written = stream
                    .write(&state[i..n])
                    .map_err(|_| clap_clap::ext::state::Error::Write)?;
                if written == 0 {
                    return Err(clap_clap::ext::state::Error::Eof.into());
                }

                i += written;
            }

            Ok(())
        }

        fn load(plugin: &TestPlug, stream: &mut IStream) -> Result<(), Error> {
            todo!()
        }
    }

    #[test]
    fn ext_impl_state() {
        TestBed::<TestPlug>::default().test(CheckExtImpl::default());
    }
}
