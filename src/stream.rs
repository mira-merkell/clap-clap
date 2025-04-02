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
    fmt::{Display, Formatter},
    io::{Read, Write},
};

use crate::ffi::{clap_istream, clap_ostream};

#[derive(Debug)]
pub struct IStream(*const clap_istream);

impl IStream {
    /// # Safety
    ///
    /// The pointer to `clap_istream` must be non-null and must point to a valid
    /// input stream handle provided by the host. In particular, the function
    /// pointer clap_istream.read must be non-null.
    #[doc(hidden)]
    pub const unsafe fn new_unchecked(clap_istream: *const clap_istream) -> Self {
        Self(clap_istream)
    }

    #[doc(hidden)]
    pub const fn clap_istream(&self) -> &clap_istream {
        // SAFETY: By construction, the pointer is non-null and points to a valid
        // clap_istream instance.
        unsafe { self.0.as_ref().unwrap() }
    }

    /// Attempt to read the content of the stream and fill the provided buffer.
    ///
    /// # Return
    ///
    /// If the entire buffer was filled with data, return `Ok(n)` where `n` is
    /// the length or the buffer.
    /// Return `Error::Eof(n)`, where `n` is the number of bytes read, if the
    /// stream was drained before the buffer was filled. In case of an IO
    /// error, return `Error::IO`.
    pub fn try_read_into(&mut self, buffer: &mut [u8]) -> Result<usize, Error> {
        let n = buffer.len();

        let mut i = 0;
        while i < n {
            let bytes_read = self.read(&mut buffer[i..n])?;
            if bytes_read == 0 {
                return Err(Error::Eof(i));
            }
            i += bytes_read;
        }
        Ok(n)
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
pub struct OStream(*const clap_ostream);

impl OStream {
    /// # Safety
    ///
    /// The pointer to `clap_ostream` must be non-null and must point to a valid
    /// input stream handle provided by the host. In particular, the function
    /// pointer clap_ostream.write must be non-null.
    #[doc(hidden)]
    pub const unsafe fn new_unchecked(clap_ostream: *const clap_ostream) -> Self {
        Self(clap_ostream)
    }

    #[doc(hidden)]
    pub const fn clap_ostream(&self) -> &clap_ostream {
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

#[derive(Debug)]
pub enum Error {
    IO(std::io::Error),
    Eof(usize),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IO(e) => write!(f, "IO: {e}"),
            Error::Eof(n) => write!(f, "EOF at byte {n}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::IO(value)
    }
}

impl From<Error> for crate::Error {
    fn from(value: Error) -> Self {
        crate::Error::Stream(value)
    }
}
