
    use super::super::CompactSerializer;
    use super::*;

    #[test]
    fn serialize_string_slice() {
        let cases = [
            ("", r#""""#),
            ("foobar", r#""foobar""#),
            ("\n", r#""\n""#),
            ("\nfoobar", r#""\nfoobar""#),
            ("foo\nbar", r#""foo\nbar""#),
            ("foobar\n", r#""foobar\n""#),
            (
                "\x00\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0A\x0B\x0C\x0D\x0E\x0F",
                r#""\u0000\u0001\u0002\u0003\u0004\u0005\u0006\u0007\b\t\n\u000b\f\r\u000e\u000f""#,
            ),
            (
                "\x10\x11\x12\x13\x14\x15\x16\x17\x18\x19\x1A\x1B\x1C\x1D\x1E\x1F",
                r#""\u0010\u0011\u0012\u0013\u0014\u0015\u0016\u0017\u0018\u0019\u001a\u001b\u001c\u001d\u001e\u001f""#,
            ),
            (r"\r\n\q", r#""\\r\\n\\q""#),
            (
                r#"They call me "Bob" but I prefer "Dennis", innit?"#,
                r#""They call me \"Bob\" but I prefer \"Dennis\", innit?""#,
            ),
        ];

        for (input, output) in cases {
            let mut serializer = CompactSerializer::default();
            input.serialize(&mut serializer);
            let s = serializer.into_string();
            assert_eq!(&s, output);
        }
    }

    #[test]
    fn serialize_string() {
        let cases = [(String::new(), r#""""#), ("foobar".to_string(), r#""foobar""#)];

        for (input, output) in cases {
            let mut serializer = CompactSerializer::default();
            input.clone().serialize(&mut serializer);
            let s = serializer.into_string();
            assert_eq!(&s, output);
        }
    }

    #[test]
    fn serialize_json_safe_string() {
        let cases = [("", r#""""#), ("a", r#""a""#), ("abc", r#""abc""#)];

        for (input, output) in cases {
            let mut serializer = CompactSerializer::default();
            JsonSafeString(input).serialize(&mut serializer);
            let s = serializer.into_string();
            assert_eq!(&s, output);
        }
    }

    #[test]
    fn serialize_lone_surrogates_string() {
        let cases = [
            ("\u{FFFD}fffd", "\"\u{FFFD}\""),
            ("_x_\u{FFFD}fffd_y_\u{FFFD}fffd_z_", "\"_x_\u{FFFD}_y_\u{FFFD}_z_\""),
            ("\u{FFFD}d834\u{FFFD}d835", r#""\ud834\ud835""#),
            ("_x_\u{FFFD}d834\u{FFFD}d835", r#""_x_\ud834\ud835""#),
            ("\u{FFFD}d834\u{FFFD}d835_y_", r#""\ud834\ud835_y_""#),
            ("_x_\u{FFFD}d834_y_\u{FFFD}d835_z_", r#""_x_\ud834_y_\ud835_z_""#),
            (
                "They call me \"Bob\" but I prefer \"Den\u{FFFD}d834\\qnis\", innit?",
                r#""They call me \"Bob\" but I prefer \"Den\ud834\\qnis\", innit?""#,
            ),
        ];

        for (input, output) in cases {
            let mut serializer = CompactSerializer::default();
            LoneSurrogatesString(input).serialize(&mut serializer);
            let s = serializer.into_string();
            assert_eq!(&s, output);
        }
    }
