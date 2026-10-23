
    use alloc::string::String;

    use proptest::prelude::*;

    use crate::CompactString;

    const MAX_SIZE: usize = core::mem::size_of::<String>();

    proptest! {
        #[test]
        #[cfg_attr(miri, ignore)]
        fn proptest_sanity(compact: CompactString) {
            let control: String = compact.clone().into();
            assert_eq!(control, compact);
        }

        /// We rely on [`proptest`]'s `String` strategy for generating a `CompactString`. When
        /// converting from a `String` into a `CompactString`, if it's short enough we should
        /// eagerly inline strings
        #[test]
        #[cfg_attr(miri, ignore)]
        fn proptest_does_not_inline_strings(compact: CompactString) {
            if compact.len() <= MAX_SIZE {
                assert!(!compact.is_heap_allocated());
            } else {
                assert!(compact.is_heap_allocated());
            }
        }
    }
