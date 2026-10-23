#[cfg(target_pointer_width = "64")]
#[test]
fn size_asserts() {
    assert_eq!(size_of::<Term>(), 16);
    assert_eq!(size_of::<CharacterClassContents>(), 16);
}
