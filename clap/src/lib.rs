pub use clap_sys::CLAP_VERSION;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn clap_version() {
        assert_eq!(CLAP_VERSION.major, 1);
        assert_eq!(CLAP_VERSION.minor, 2);
        assert_eq!(CLAP_VERSION.revision, 3);
    }
}