
    use arbitrary::{Arbitrary, Unstructured};

    use crate::CompactString;

    #[test]
    fn arbitrary_sanity() {
        let mut data = Unstructured::new(&[42; 50]);
        let compact = CompactString::arbitrary(&mut data).expect("generate a CompactString");

        // we don't really care what the content of the CompactString is, just that one's generated
        assert!(!compact.is_empty());
    }

    #[test]
    fn arbitrary_inlines_strings() {
        let mut data = Unstructured::new(&[42; 20]);
        let compact = CompactString::arbitrary(&mut data).expect("generate a CompactString");

        // running this manually, we generate the string "**"
        assert!(!compact.is_heap_allocated());
    }
