
    use alloc::string::String;

    use test_strategy::proptest;
    use valuable::{Valuable, Value, Visit};

    use super::*;
    use crate::tests::rand_unicode;

    /// A [`Visit`] that records the single scalar value passed to it.
    #[derive(Default)]
    struct CapturingVisit {
        captured: Option<String>,
    }

    impl Visit for CapturingVisit {
        fn visit_value(&mut self, value: Value<'_>) {
            if let Value::String(s) = value {
                self.captured = Some(String::from(s));
            }
        }
    }

    #[test]
    fn smoketest_as_value() {
        // Inline string.
        let short = CompactString::from("hello");
        assert!(matches!(short.as_value(), Value::String("hello")));

        // Heap allocated string.
        let long = CompactString::from("I am a long string that will be on the heap");
        assert!(long.is_heap_allocated());
        assert!(matches!(
            long.as_value(),
            Value::String("I am a long string that will be on the heap")
        ));
    }

    #[test]
    fn smoketest_visit() {
        let compact = CompactString::from("hello");
        let mut visitor = CapturingVisit::default();
        compact.visit(&mut visitor);
        assert_eq!(visitor.captured.as_deref(), Some("hello"));
    }

    #[proptest]
    #[cfg_attr(miri, ignore)]
    fn proptest_as_value_matches_str(#[strategy(rand_unicode())] s: String) {
        let compact = CompactString::new(&s);

        // The value should always present as a `Value::String` equal to the source `&str`.
        match compact.as_value() {
            Value::String(v) => assert_eq!(v, s.as_str()),
            other => panic!("expected Value::String, got {other:?}"),
        }

        // And visiting should surface the same string.
        let mut visitor = CapturingVisit::default();
        compact.visit(&mut visitor);
        assert_eq!(visitor.captured.as_deref(), Some(s.as_str()));
    }
