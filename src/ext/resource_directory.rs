//! Plugin-side resource-directory extension (CLAP draft).
//!
//! This module wraps the CLAP draft extension `clap.resource-directory/1`,
//! the standard replacement for the removed file-reference draft. It lets a
//! plugin store the external files it references (samples, impulse
//! responses, model files, ...) in a directory managed by the host, so that
//! hosts can collect every file a plugin uses into the project/session
//! directory.
//!
//! The typical workflow is:
//!
//! 1. The plugin requests a directory from the host, shared among all plugin
//!    instances (`is_shared = true`) or exclusive to the instance, via
//!    [`HostResourceDirectory::request_directory()`].
//! 2. If the host grants the request, it sets the directory via
//!    [`ResourceDirectory::set_directory()`]. The directory remains valid
//!    until it is overridden or the plugin is destroyed.
//! 3. The host asks the plugin to copy its referenced resources into the
//!    directory via [`ResourceDirectory::collect()`].
//! 4. The host may enumerate the files the plugin uses in the shared
//!    directory via [`ResourceDirectory::files_count()`] and
//!    [`ResourceDirectory::file_path()`], in order to garbage collect unused
//!    files.
//!
//! The plugin must store relative paths toward the resource directory in its
//! state, so that projects remain relocatable. A null or blank path passed
//! to [`ResourceDirectory::set_directory()`] clears the directory location.

use std::fmt::{Display, Formatter};

use crate::{
    ffi::clap_host_resource_directory,
    host::Host,
    plugin::Plugin,
};

/// Plugin-side resource-directory extension.
///
/// Implement this trait to let the host manage the external files the plugin
/// depends on. See the [module-level documentation](self) for the workflow.
pub trait ResourceDirectory<P: Plugin> {
    /// Sets the directory in which the plugin can save its resources.
    ///
    /// `path` is absolute and remains valid until it is overridden or the
    /// plugin is destroyed. `None` or a blank path means the host cleared
    /// the directory location. `is_shared` tells whether the directory is
    /// shared among plugin instances (read-only content) or exclusive to
    /// this instance (the host may duplicate or delete it with the
    /// instance).
    fn set_directory(plugin: &mut P, path: Option<&str>, is_shared: bool);

    /// Asks the plugin to put its resources into the resource directory.
    ///
    /// It is not necessary to collect files which belong to the plugin's
    /// factory content unless `all` is `true`.
    fn collect(plugin: &mut P, all: bool);

    /// Returns the number of files used by the plugin in the shared resource
    /// folder.
    fn files_count(plugin: &P) -> u32;

    /// Writes the relative path of the shared-folder file at `index` into
    /// `path`.
    ///
    /// Returns `Ok` if the path was written, `Err(Error::OutOfBounds)` if
    /// `index` is out of bounds, or `Err(Error::BufferTooSmall)` if the path
    /// doesn't fit the host's buffer.
    fn file_path(plugin: &P, index: u32, path: &mut String) -> Result<(), Error>;
}

pub(crate) use ffi::PluginResourceDirectory;

#[derive(Debug)]
pub enum Error {
    /// The requested index is out of bounds.
    OutOfBounds,
    /// The provided buffer is too small to hold the path.
    BufferTooSmall,
    /// A string could not be converted to/from the host representation.
    InvalidString,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use Error::*;
        match self {
            OutOfBounds => write!(f, "resource directory index out of bounds"),
            BufferTooSmall => write!(f, "resource directory path buffer too small"),
            InvalidString => write!(f, "invalid resource directory string"),
        }
    }
}

impl std::error::Error for Error {}

impl From<Error> for crate::Error {
    fn from(value: Error) -> Self {
        crate::ext::Error::ResourceDirectory(value).into()
    }
}

mod ffi {
    use std::{
        ffi::{CStr, CString},
        marker::PhantomData,
        os::raw::c_char,
    };

    use crate::{
        ext::resource_directory::ResourceDirectory,
        ffi::{clap_plugin, clap_plugin_resource_directory},
        plugin::{ClapPlugin, Plugin},
    };

    extern "C-unwind" fn set_directory<E, P>(
        plugin: *const clap_plugin,
        path: *const c_char,
        is_shared: bool,
    ) where
        E: ResourceDirectory<P>,
        P: Plugin,
    {
        if plugin.is_null() {
            return;
        }
        let path = if path.is_null() {
            None
        } else {
            // SAFETY: The pointer is a valid C string obtained from the host,
            // living for the duration of this call.
            let path = unsafe { CStr::from_ptr(path) };
            let Ok(path) = path.to_str() else {
                return;
            };
            Some(path)
        };
        // SAFETY: We just checked that the pointer is non-null and the plugin
        // has been obtained from host and is tied to type P.
        let mut clap_plugin = unsafe { ClapPlugin::<P>::new_unchecked(plugin) };
        // SAFETY: This function is called on the main thread.
        // It is guaranteed that we are the only function accessing the plugin now.
        // So the mutable reference to plugin for the duration of this call is
        // safe.
        let plugin = unsafe { clap_plugin.plugin() };
        E::set_directory(plugin, path, is_shared)
    }

    extern "C-unwind" fn collect<E, P>(plugin: *const clap_plugin, all: bool)
    where
        E: ResourceDirectory<P>,
        P: Plugin,
    {
        if plugin.is_null() {
            return;
        }
        // SAFETY: We just checked that the pointer is non-null and the plugin
        // has been obtained from host and is tied to type P.
        let mut clap_plugin = unsafe { ClapPlugin::<P>::new_unchecked(plugin) };
        // SAFETY: This function is called on the main thread.
        // It is guaranteed that we are the only function accessing the plugin now.
        // So the mutable reference to plugin for the duration of this call is
        // safe.
        let plugin = unsafe { clap_plugin.plugin() };
        E::collect(plugin, all)
    }

    extern "C-unwind" fn get_files_count<E, P>(plugin: *const clap_plugin) -> u32
    where
        E: ResourceDirectory<P>,
        P: Plugin,
    {
        if plugin.is_null() {
            return 0;
        }
        // SAFETY: We just checked that the pointer is non-null and the plugin
        // has been obtained from host and is tied to type P.
        let mut clap_plugin = unsafe { ClapPlugin::<P>::new_unchecked(plugin) };
        // SAFETY: This function is called on the main thread.
        // It is guaranteed that we are the only function accessing the plugin now.
        let plugin = unsafe { clap_plugin.plugin() };
        E::files_count(plugin)
    }

    extern "C-unwind" fn get_file_path<E, P>(
        plugin: *const clap_plugin,
        index: u32,
        path: *mut c_char,
        path_size: u32,
    ) -> i32
    where
        E: ResourceDirectory<P>,
        P: Plugin,
    {
        if plugin.is_null() || path.is_null() || path_size == 0 {
            return -1;
        }
        // SAFETY: We just checked that the pointer is non-null and the plugin
        // has been obtained from host and is tied to type P.
        let mut clap_plugin = unsafe { ClapPlugin::<P>::new_unchecked(plugin) };
        // SAFETY: This function is called on the main thread.
        // It is guaranteed that we are the only function accessing the plugin now.
        let plugin = unsafe { clap_plugin.plugin() };

        let mut buffer = String::new();
        if E::file_path(plugin, index, &mut buffer).is_err() {
            return -1;
        }
        let cstring = match CString::new(buffer) {
            Ok(s) => s,
            Err(_) => return -1,
        };
        let bytes = cstring.as_bytes_with_nul();
        if bytes.len() > path_size as usize {
            return -1;
        }
        // SAFETY: path is non-null and we just checked that it points to at
        // least bytes.len() writable bytes.
        unsafe {
            std::ptr::copy_nonoverlapping(bytes.as_ptr() as *const c_char, path, bytes.len());
        }
        bytes.len() as i32
    }

    pub(crate) struct PluginResourceDirectory<P> {
        #[allow(unused)]
        clap_plugin_resource_directory: clap_plugin_resource_directory,
        _marker: PhantomData<P>,
    }

    impl<P: Plugin> PluginResourceDirectory<P> {
        pub(crate) fn new<E: ResourceDirectory<P>>(_: E) -> Self {
            Self {
                clap_plugin_resource_directory: clap_plugin_resource_directory {
                    set_directory: Some(set_directory::<E, P>),
                    collect: Some(collect::<E, P>),
                    get_files_count: Some(get_files_count::<E, P>),
                    get_file_path: Some(get_file_path::<E, P>),
                },
                _marker: PhantomData,
            }
        }
    }
}

impl<P: Plugin> ResourceDirectory<P> for () {
    fn set_directory(_plugin: &mut P, _path: Option<&str>, _is_shared: bool) {}

    fn collect(_plugin: &mut P, _all: bool) {}

    fn files_count(_plugin: &P) -> u32 {
        0
    }

    fn file_path(_plugin: &P, _index: u32, _path: &mut String) -> Result<(), Error> {
        Err(Error::OutOfBounds)
    }
}

/// Host-side resource-directory extension.
#[derive(Debug)]
pub struct HostResourceDirectory<'a> {
    host: &'a Host,
    clap_host_resource_directory: &'a clap_host_resource_directory,
}

impl<'a> HostResourceDirectory<'a> {
    /// # Safety
    ///
    /// All extension interface function pointers must be non-null (Some), and
    /// the functions must be thread-safe.
    pub(crate) const unsafe fn new_unchecked(
        host: &'a Host,
        clap_host_resource_directory: &'a clap_host_resource_directory,
    ) -> Self {
        Self {
            host,
            clap_host_resource_directory,
        }
    }

    /// Request the host to set up a resource directory with the specified
    /// sharing.
    ///
    /// Returns `true` if the host will perform the request. If the plugin is
    /// done with the directory, it releases it with
    /// [`release_directory()`](Self::release_directory). If `is_shared` is
    /// `false`, the host may delete the directory content upon release.
    pub fn request_directory(&self, is_shared: bool) -> bool {
        if let Some(callback) = self.clap_host_resource_directory.request_directory {
            // SAFETY: By construction, the callback is a valid function pointer
            // obtained from the host, and the call is thread-safe.
            unsafe { callback(self.host.clap_host(), is_shared) }
        } else {
            false
        }
    }

    /// Tell the host that the resource directory of the specified sharing is
    /// no longer required.
    ///
    /// If `is_shared` is `false`, the host may delete the directory content.
    pub fn release_directory(&self, is_shared: bool) {
        if let Some(callback) = self.clap_host_resource_directory.release_directory {
            // SAFETY: By construction, the callback is a valid function pointer
            // obtained from the host, and the call is thread-safe.
            unsafe { callback(self.host.clap_host(), is_shared) }
        }
    }
}
