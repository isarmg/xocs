
    use alloc::string::String;

    use quickcheck_macros::quickcheck;

    use crate::CompactString;

    #[quickcheck]
    #[cfg_attr(miri, ignore)]
    fn quickcheck_sanity(compact: CompactString) {
        let control: String = compact.clone().into();
        assert_eq!(control, compact);
    }

    #[quickcheck]
    #[cfg_attr(miri, ignore)]
    fn quickcheck_inlines_strings(compact: CompactString) {
        if compact.len() <= core::mem::size_of::<String>() {
            assert!(!compact.is_heap_allocated())
        } else {
            assert!(compact.is_heap_allocated())
        }
    }
