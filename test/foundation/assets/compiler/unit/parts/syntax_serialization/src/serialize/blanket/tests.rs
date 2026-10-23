
    use super::super::CompactSerializer;
    use super::*;

    #[expect(clippy::needless_borrow)]
    #[test]
    fn serialize_ref() {
        let cases = [(&"foo", r#""foo""#), (&&"bar", r#""bar""#)];

        for (input, output) in cases {
            let mut serializer = CompactSerializer::default();
            input.serialize(&mut serializer);
            let s = serializer.into_string();
            assert_eq!(&s, output);
        }
    }

    #[expect(clippy::mut_mut)]
    #[test]
    fn serialize_mut_ref() {
        let cases = [(&mut "foo", r#""foo""#), (&mut &mut "bar", r#""bar""#)];

        for (input, output) in cases {
            let mut serializer = CompactSerializer::default();
            input.serialize(&mut serializer);
            let s = serializer.into_string();
            assert_eq!(&s, output);
        }
    }

    #[test]
    fn serialize_option() {
        let cases = [(None, "null"), (Some(123.0f64), "123")];

        for (input, output) in cases {
            let mut serializer = CompactSerializer::default();
            input.serialize(&mut serializer);
            let s = serializer.into_string();
            assert_eq!(&s, output);
        }
    }

    #[test]
    fn serialize_cow() {
        let cases =
            [(Cow::Borrowed("foo"), r#""foo""#), (Cow::Owned("bar".to_string()), r#""bar""#)];

        for (input, output) in cases {
            let mut serializer = CompactSerializer::default();
            input.serialize(&mut serializer);
            let s = serializer.into_string();
            assert_eq!(&s, output);
        }
    }
