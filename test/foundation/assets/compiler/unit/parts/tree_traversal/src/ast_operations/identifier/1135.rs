#[test]
fn test() {
    let cases = &[
        ("foo", "foo"),
        ("fooBar", "fooBar"),
        ("fooBar1", "fooBar1"),
        ("foo-bar", "fooBar"),
        ("foo bar", "fooBar"),
        ("foo-bar-1", "fooBar1"),
        ("1foo-bar", "FooBar"),
        ("1-foo-bar", "FooBar"),
        ("-- --", "_"),
        ("_output$headers$x-amzn-requestid", "_output$headers$xAmznRequestid"),
    ];

    for &(input, expected) in cases {
        assert_eq!(to_identifier(input.to_string()), expected);
    }
}
