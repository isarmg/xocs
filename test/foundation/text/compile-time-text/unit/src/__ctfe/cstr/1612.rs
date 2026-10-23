#[test]
fn test_raw_cstr() {
    const FMT: &str = "%d\n";
    let fmt = raw_cstr!(FMT);
    let len = FMT.len() + 1;
    let bytes: &[u8] = unsafe { core::slice::from_raw_parts(fmt.cast(), len) };
    assert_eq!(bytes, b"%d\n\0");
}
