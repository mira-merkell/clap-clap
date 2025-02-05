use clap_clap::id::ClapId;

#[test]
fn clap_sys_invalid_id_more_than_u16max() {
    assert!(clap_sys::CLAP_INVALID_ID > u16::MAX as clap_sys::clap_id);
}

#[test]
fn valid() {
    let _ = ClapId::try_from(0).unwrap();
    let _ = ClapId::try_from(10).unwrap();
    let _ = ClapId::try_from(1000).unwrap();
    let _ = ClapId::try_from(10000000).unwrap();
    let _ = ClapId::try_from(u32::MAX - 1).unwrap();
}

#[test]
fn invalid() {
    let _ = ClapId::try_from(u32::MAX).unwrap_err();
    let _ = ClapId::try_from(usize::try_from(1u64 << 33).unwrap()).unwrap_err();
    let _ = ClapId::try_from(-1).unwrap_err();
    let _ = ClapId::try_from(-10).unwrap_err();
}

#[test]
fn is_valid() {
    assert!(ClapId::from(0).is_valid());
    assert!(ClapId::from(10).is_valid());
    assert!(ClapId::from(100).is_valid());
    assert!(ClapId::from(1000).is_valid());
    assert!(ClapId::from(10000).is_valid());

    assert!(!ClapId::invalid_id().is_valid());
}

#[test]
fn invalid_is_max() {
    assert_eq!(clap_sys::CLAP_INVALID_ID, ClapId::invalid_id().into());
    assert_eq!(u32::MAX, ClapId::invalid_id().into());
}
