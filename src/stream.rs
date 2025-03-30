//! CLAP I/O streams for storing and loading plugin state.
//!
//! # Notes on using streams
//!
//! When working with `IStream` and `OStream` objects to load and save
//! state, it is important to keep in mind that the host may limit the number of
//! bytes that can be read or written at a time. The return values for the
//! stream read and write functions indicate how many bytes were actually read
//! or written. You need to use a loop to ensure that you read or write the
//! entirety of your state. Don't forget to also consider the negative return
//! values for the end of file and IO error codes.

use std::{
    ffi::c_void,
    io::{Read, Write},
};

use crate::ffi::{clap_istream, clap_ostream};

#[derive(Debug)]
struct IStream(*const clap_istream);

impl IStream {
    /// # Safety
    ///
    /// The pointer to `clap_istream` must be non-null and must point to a valid
    /// input stream handle provided by the host. In particular, the function
    /// pointer clap_istream.read must be non-null.
    pub(crate) const unsafe fn new_unchecked(clap_istream: *const clap_istream) -> Self {
        Self(clap_istream)
    }

    pub(crate) const fn clap_istream(&self) -> &clap_istream {
        // SAFETY: By construction, the pointer is non-null and points to a valid
        // clap_istream instance.
        unsafe { self.0.as_ref().unwrap() }
    }
}

impl Read for IStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let size = buf.len() as u64;
        let buffer = buf.as_mut_ptr() as *mut c_void;

        // SAFETY: The pointer to `read` method is non-null and safe to call by
        // construction.
        let n = unsafe { self.clap_istream().read.unwrap()(self.0, buffer, size) };

        if n >= 0 {
            Ok(n as usize)
        } else {
            Err(std::io::Error::other("read error"))
        }
    }
}

#[derive(Debug)]
struct OStream(*const clap_ostream);

impl OStream {
    /// # Safety
    ///
    /// The pointer to `clap_ostream` must be non-null and must point to a valid
    /// input stream handle provided by the host. In particular, the function
    /// pointer clap_ostream.write must be non-null.
    pub(crate) const unsafe fn new_unchecked(clap_ostream: *const clap_ostream) -> Self {
        Self(clap_ostream)
    }

    pub(crate) const fn clap_ostream(&self) -> &clap_ostream {
        // SAFETY: By construction, the pointer is non-null and points to a valid
        // clap_ostream instance.
        unsafe { self.0.as_ref().unwrap() }
    }
}

impl Write for OStream {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let size = buf.len() as u64;
        let buffer = buf.as_ptr() as *mut c_void;

        // SAFETY: The pointer to `write` method is non-null and safe to call by
        // construction.
        let n = unsafe { self.clap_ostream().write.unwrap()(self.0, buffer, size) };
        if n >= 0 {
            Ok(n as usize)
        } else {
            Err(std::io::Error::other("write error"))
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
