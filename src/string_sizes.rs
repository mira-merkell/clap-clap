pub const CLAP_NAME_SIZE: usize = clap_sys::CLAP_NAME_SIZE;
pub const CLAP_PATH_SIZE: usize = clap_sys::CLAP_PATH_SIZE;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_size() {
        // This value is stable CLAP API, and should never change.
        assert_eq!(CLAP_NAME_SIZE, 256);
    }

    #[test]
    fn path_size() {
        // This value is stable CLAP API, and should never change.
        assert_eq!(CLAP_PATH_SIZE, 1024);
    }
}
