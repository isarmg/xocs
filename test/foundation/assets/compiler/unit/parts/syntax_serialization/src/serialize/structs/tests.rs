
    use super::super::{CompactSerializer, FlatStructSerializer, PrettySerializer, Serializer};
    use super::*;

    #[test]
    fn serialize_struct() {
        struct Foo<'a> {
            n: f64,
            u: u32,
            bar: Bar<'a>,
            empty: Null,
            hello: Option<&'a str>,
            maybe_bar: Option<Bar<'a>>,
            maybe_not_bar: Option<Bar<'a>>,
        }

        struct Bar<'a> {
            yes: &'a str,
            no: &'a str,
        }

        struct Null;

        impl ESTree for Foo<'_> {
            fn serialize<S: Serializer>(&self, serializer: S) {
                let mut state = serializer.serialize_struct();
                state.serialize_field("n", &self.n);
                state.serialize_field("u", &self.u);
                state.serialize_field("bar", &self.bar);
                state.serialize_field("empty", &self.empty);
                state.serialize_field("hello", &self.hello);
                state.serialize_field("maybe_bar", &self.maybe_bar);
                state.serialize_field("maybe_not_bar", &self.maybe_not_bar);
                state.end();
            }
        }

        impl ESTree for Bar<'_> {
            fn serialize<S: Serializer>(&self, serializer: S) {
                let mut state = serializer.serialize_struct();
                state.serialize_field("yes", &self.yes);
                state.serialize_field("no", &self.no);
                state.end();
            }
        }

        impl ESTree for Null {
            fn serialize<S: Serializer>(&self, serializer: S) {
                let state = serializer.serialize_struct();
                state.end();
            }
        }

        let foo = Foo {
            n: 123.0,
            u: 12345,
            bar: Bar { yes: "yup", no: "nope" },
            empty: Null,
            hello: Some("hi!"),
            maybe_bar: Some(Bar { yes: "hell yeah!", no: "not a chance in a million, mate" }),
            maybe_not_bar: None,
        };

        let mut serializer = CompactSerializer::default();
        foo.serialize(&mut serializer);
        let s = serializer.into_string();
        assert_eq!(
            &s,
            r#"{"n":123,"u":12345,"bar":{"yes":"yup","no":"nope"},"empty":{},"hello":"hi!","maybe_bar":{"yes":"hell yeah!","no":"not a chance in a million, mate"},"maybe_not_bar":null}"#
        );

        let mut serializer = PrettySerializer::default();
        foo.serialize(&mut serializer);
        let s = serializer.into_string();
        assert_eq!(
            &s,
            r#"{
  "n": 123,
  "u": 12345,
  "bar": {
    "yes": "yup",
    "no": "nope"
  },
  "empty": {},
  "hello": "hi!",
  "maybe_bar": {
    "yes": "hell yeah!",
    "no": "not a chance in a million, mate"
  },
  "maybe_not_bar": null
}"#
        );
    }

    #[test]
    fn serialize_flattened_struct() {
        struct Outer {
            outer1: &'static str,
            inner: Inner,
            outer2: &'static str,
        }

        struct Inner {
            inner1: &'static str,
            innermost: Innermost,
            inner2: &'static str,
        }

        struct Innermost {
            innermost1: &'static str,
            innermost2: &'static str,
        }

        impl ESTree for Outer {
            fn serialize<S: Serializer>(&self, serializer: S) {
                let mut state = serializer.serialize_struct();
                state.serialize_field("outer1", &self.outer1);
                self.inner.serialize(FlatStructSerializer(&mut state));
                state.serialize_field("outer2", &self.outer2);
                state.end();
            }
        }

        impl ESTree for Inner {
            fn serialize<S: Serializer>(&self, serializer: S) {
                let mut state = serializer.serialize_struct();
                state.serialize_field("inner1", &self.inner1);
                self.innermost.serialize(FlatStructSerializer(&mut state));
                state.serialize_field("inner2", &self.inner2);
                state.end();
            }
        }

        impl ESTree for Innermost {
            fn serialize<S: Serializer>(&self, serializer: S) {
                let mut state = serializer.serialize_struct();
                state.serialize_field("innermost1", &self.innermost1);
                state.serialize_field("innermost2", &self.innermost2);
                state.end();
            }
        }

        let outer = Outer {
            outer1: "out1",
            inner: Inner {
                inner1: "in1",
                innermost: Innermost { innermost1: "inin1", innermost2: "inin2" },
                inner2: "in2",
            },
            outer2: "out2",
        };

        let mut serializer = CompactSerializer::default();
        outer.serialize(&mut serializer);
        let s = serializer.into_string();
        assert_eq!(
            &s,
            r#"{"outer1":"out1","inner1":"in1","innermost1":"inin1","innermost2":"inin2","inner2":"in2","outer2":"out2"}"#
        );

        let mut serializer = PrettySerializer::default();
        outer.serialize(&mut serializer);
        let s = serializer.into_string();
        assert_eq!(
            &s,
            r#"{
  "outer1": "out1",
  "inner1": "in1",
  "innermost1": "inin1",
  "innermost2": "inin2",
  "inner2": "in2",
  "outer2": "out2"
}"#
        );
    }

    #[test]
    fn serialize_struct_with_or_without_ts() {
        struct Foo {
            js: u32,
            ts: u32,
            js_only: u32,
            more_js: u32,
        }

        impl ESTree for Foo {
            fn serialize<S: Serializer>(&self, serializer: S) {
                let mut state = serializer.serialize_struct();
                state.serialize_field("js", &self.js);
                state.serialize_ts_field("ts", &self.ts);
                state.serialize_js_field("jsOnly", &self.js_only);
                state.serialize_field("moreJs", &self.more_js);
                state.end();
            }
        }

        let foo = Foo { js: 1, ts: 2, js_only: 3, more_js: 4 };

        let mut serializer = CompactSerializer::default();
        foo.serialize(&mut serializer);
        let s = serializer.into_string();
        assert_eq!(&s, r#"{"js":1,"ts":2,"moreJs":4}"#);

        let mut serializer = PrettySerializer::default();
        foo.serialize(&mut serializer);
        let s = serializer.into_string();
        assert_eq!(
            &s,
            r#"{
  "js": 1,
  "ts": 2,
  "moreJs": 4
}"#
        );

        let mut serializer = CompactSerializer::new(false, false);
        foo.serialize(&mut serializer);
        let s = serializer.into_string();
        assert_eq!(&s, r#"{"js":1,"jsOnly":3,"moreJs":4}"#);

        let mut serializer = PrettySerializer::new(false, false);
        foo.serialize(&mut serializer);
        let s = serializer.into_string();
        assert_eq!(
            &s,
            r#"{
  "js": 1,
  "jsOnly": 3,
  "moreJs": 4
}"#
        );
    }
