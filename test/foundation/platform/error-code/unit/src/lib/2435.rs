#[cfg(feature = "std")]
#[test]
fn check_description() {
    let expect = "Operation not permitted";
    let errno_code = libc::EPERM;
    set_errno(Errno(errno_code));

    assert_eq!(errno().to_string(), expect);
    assert_eq!(
        format!("{:?}", errno()),
        format!(
            "Errno {{ code: {}, description: Some({:?}) }}",
            errno_code, expect
        )
    );
}
