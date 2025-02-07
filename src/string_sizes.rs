pub const CLAP_NAME_SIZE: usize = crate::ffi::CLAP_NAME_SIZE as usize;
pub const CLAP_PATH_SIZE: usize = crate::ffi::CLAP_PATH_SIZE as usize;

#[cfg(test)]
mod tests {
    #[test]
    fn cast_name_size_as_usize() {
        usize::try_from(crate::ffi::CLAP_NAME_SIZE).unwrap();
    }

    #[test]
    fn cast_path_size_as_usize() {
        usize::try_from(crate::ffi::CLAP_PATH_SIZE).unwrap();
    }
}
