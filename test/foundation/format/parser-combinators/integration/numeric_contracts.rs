use blog_format_parser_combinators::{Parser, ascii::hex_uint, error::InputError};

#[test]
fn hexadecimal_digits_round_trip_for_text_and_bytes() {
    for value in u8::MIN..=u8::MAX {
        for input in [format!("{value:02x};"), format!("{value:02X};")] {
            let parsed = hex_uint::<_, u8, InputError<_>>
                .parse_peek(input.as_str())
                .unwrap();
            assert_eq!(parsed, (";", value));
            let parsed = hex_uint::<_, u8, InputError<_>>
                .parse_peek(input.as_bytes())
                .unwrap();
            assert_eq!(parsed, (b";".as_slice(), value));
        }
    }
}

#[test]
fn hexadecimal_input_rejects_invalid_digits_and_overflow() {
    for input in ["", "g", "-1", "é", "1g", "100", "000"] {
        assert!(hex_uint::<_, u8, InputError<_>>.parse(input).is_err());
        assert!(
            hex_uint::<_, u8, InputError<_>>
                .parse(input.as_bytes())
                .is_err()
        );
    }
    assert_eq!(
        hex_uint::<_, u32, InputError<_>>.parse("FFFFFFFF").unwrap(),
        u32::MAX
    );
    assert!(
        hex_uint::<_, u32, InputError<_>>
            .parse("100000000")
            .is_err()
    );
}
