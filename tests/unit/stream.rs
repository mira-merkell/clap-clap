mod istream {
    use std::{ffi::c_void, io::Read, marker::PhantomPinned, pin::Pin, ptr::null_mut};

    use clap_clap::{ffi::clap_istream, stream::IStream};

    trait Test {
        fn test(self, bed: Pin<&mut TestBed>);
    }

    #[derive(Debug, Default, Clone)]
    struct TestConfig {
        buf: Option<Vec<u8>>, // `None` will result in read error.
    }

    impl TestConfig {
        fn test(self, case: impl Test) -> Self {
            TestBed::new(self.clone()).as_mut().test(case);
            self
        }
    }

    #[derive(Debug)]
    struct TestBed {
        config: TestConfig,
        clap_istream: clap_istream,
        istream: Option<IStream>,

        _marker: PhantomPinned,
    }

    impl TestBed {
        fn new(config: TestConfig) -> Pin<Box<Self>> {
            let mut bed = Box::new(Self {
                config,
                clap_istream: clap_istream {
                    ctx: null_mut(),
                    read: Some(stream_read),
                },
                istream: None,
                _marker: PhantomPinned,
            });

            bed.clap_istream.ctx = (&raw mut *bed).cast();
            bed.istream = Some(unsafe { IStream::new_unchecked(&raw const bed.clap_istream) });

            Box::into_pin(bed)
        }

        fn buf(&self) -> Option<&Vec<u8>> {
            self.config.buf.as_ref()
        }

        fn istream(self: Pin<&mut Self>) -> &mut IStream {
            unsafe { self.get_unchecked_mut().istream.as_mut().unwrap() }
        }

        fn test(mut self: Pin<&mut Self>, case: impl Test) -> Pin<&mut Self> {
            case.test(self.as_mut());
            self
        }
    }

    extern "C-unwind" fn stream_read(
        stream: *const clap_istream,
        buffer: *mut c_void,
        size: u64,
    ) -> i64 {
        assert!(!stream.is_null());
        let bed: &mut TestBed = unsafe {
            stream
                .as_ref()
                .unwrap()
                .ctx
                .cast::<TestBed>()
                .as_mut()
                .unwrap()
        };

        if let Some(bed_buf) = &bed.buf() {
            let n = bed_buf.len().min(size as usize);
            unsafe { buffer.copy_from_nonoverlapping(bed_buf.as_ptr().cast(), n) };
            n as i64
        } else {
            -1
        }
    }

    struct CheckReadError {
        size: usize,
    }

    impl Test for CheckReadError {
        fn test(self, bed: Pin<&mut TestBed>) {
            assert!(bed.buf().is_none());

            let mut buf = vec![0; self.size];
            let err = bed.istream().read(&mut buf).unwrap_err();
            assert_eq!(err.kind(), std::io::ErrorKind::Other);
            assert_eq!(err.to_string(), "read error");
        }
    }

    #[test]
    fn read_error() {
        TestConfig::default()
            .test(CheckReadError { size: 0 })
            .test(CheckReadError { size: 1 })
            .test(CheckReadError { size: 10 });
    }

    struct CheckRead {
        size: usize,
    }

    impl Test for CheckRead {
        fn test(self, mut bed: Pin<&mut TestBed>) {
            assert!(bed.buf().is_some());
            let mut buf = vec![0; self.size];

            let n = self.size.min(bed.buf().unwrap().len());
            let n_exp = bed.as_mut().istream().read(&mut buf).unwrap();
            assert_eq!(n, n_exp);

            assert_eq!(buf[0..n], bed.buf().unwrap()[0..n]);
        }
    }

    #[test]
    fn read_0() {
        TestConfig { buf: Some(vec![]) }
            .test(CheckRead { size: 0 })
            .test(CheckRead { size: 1 })
            .test(CheckRead { size: 2 })
            .test(CheckRead { size: 3 });
    }

    #[test]
    fn read_1() {
        TestConfig { buf: Some(vec![1]) }
            .test(CheckRead { size: 0 })
            .test(CheckRead { size: 1 })
            .test(CheckRead { size: 2 })
            .test(CheckRead { size: 3 });
    }

    #[test]
    fn read_2() {
        TestConfig {
            buf: Some(vec![1, 2]),
        }
        .test(CheckRead { size: 0 })
        .test(CheckRead { size: 1 })
        .test(CheckRead { size: 2 })
        .test(CheckRead { size: 3 });
    }

    #[test]
    fn read_3() {
        TestConfig {
            buf: Some(vec![1, 2, 3]),
        }
        .test(CheckRead { size: 0 })
        .test(CheckRead { size: 1 })
        .test(CheckRead { size: 2 })
        .test(CheckRead { size: 3 });
    }
}
