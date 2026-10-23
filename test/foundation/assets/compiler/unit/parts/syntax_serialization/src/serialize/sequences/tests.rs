
    use super::super::{CompactSerializer, PrettySerializer, StructSerializer};
    use super::*;

    #[test]
    fn serialize_sequence() {
        struct Foo<'a> {
            none: &'a [&'a str],
            one: &'a [&'a str],
            two: [&'a str; 2],
        }

        impl ESTree for Foo<'_> {
            fn serialize<S: Serializer>(&self, serializer: S) {
                let mut state = serializer.serialize_struct();
                state.serialize_field("none", &self.none);
                state.serialize_field("one", &self.one);
                state.serialize_field("two", &self.two);
                state.end();
            }
        }

        let foo = Foo { none: &[], one: &["one"], two: ["two one", "two two"] };

        let mut serializer = CompactSerializer::default();
        foo.serialize(&mut serializer);
        let s = serializer.into_string();
        assert_eq!(&s, r#"{"none":[],"one":["one"],"two":["two one","two two"]}"#);

        let mut serializer = PrettySerializer::default();
        foo.serialize(&mut serializer);
        let s = serializer.into_string();
        assert_eq!(
            &s,
            r#"{
  "none": [],
  "one": [
    "one"
  ],
  "two": [
    "two one",
    "two two"
  ]
}"#
        );
    }
