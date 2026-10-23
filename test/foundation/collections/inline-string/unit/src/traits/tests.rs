
    use alloc::string::{String, ToString};
    use alloc::vec::Vec;
    use core::num;

    use proptest::prelude::*;
    use test_strategy::proptest;

    use super::{CompactStringExt, ToCompactString};
    use crate::CompactString;

    #[test]
    fn str_implements_to_compact_string() {
        fn assert_impl<T: ToCompactString + ?Sized>() {}
        assert_impl::<str>();
    }

    #[test]
    fn test_join() {
        let slice = ["hello", "world"];
        let c1 = (&slice).join_compact(" ");
        let c2 = slice.join_compact(" ");
        assert_eq!(c1, "hello world");
        assert_eq!(c2, "hello world");

        let vector = vec!["🍎", "🍊", "🍌"];
        let c = vector.join_compact(",");
        assert_eq!(c, "🍎,🍊,🍌");

        let owned_strings = ["foo".to_string(), "bar".to_string()];
        let c = owned_strings.join_compact("/");
        assert_eq!(c, "foo/bar");
    }

    #[test]
    fn test_join_iter() {
        let values = ["foo", "bar", "baz"];
        let compact = values.iter().map(|x| x.to_string()).join_compact("/");
        assert_eq!(compact, "foo/bar/baz");

        let compact = values.into_iter().map(|x| x.trim()).join_compact("-");
        assert_eq!(compact, "foo-bar-baz");

        let compact = values
            .into_iter()
            .map(CompactString::from)
            .join_compact("...");
        assert_eq!(compact, "foo...bar...baz");
    }

    #[proptest]
    #[cfg_attr(miri, ignore)]
    fn proptest_join(items: Vec<String>, separator: String) {
        let c: CompactString = items.iter().join_compact(&separator);
        let s: String = items.join(&separator);
        assert_eq!(c, s);
    }

    #[test]
    fn test_concat() {
        let items = vec!["hello", "world"];
        let c = items.join_compact(" ");
        assert_eq!(c, "hello world");

        let vector = vec!["🍎", "🍊", "🍌"];
        let c = vector.concat_compact();
        assert_eq!(c, "🍎🍊🍌");
    }

    #[proptest]
    #[cfg_attr(miri, ignore)]
    fn proptest_concat(items: Vec<String>) {
        let c: CompactString = items.iter().concat_compact();
        let s: String = items.concat();
        assert_eq!(c, s);
    }

    #[proptest]
    #[cfg_attr(miri, ignore)]
    fn proptest_to_compact_string_u8(val: u8) {
        let compact = val.to_compact_string();
        prop_assert_eq!(compact.as_str(), val.to_string());
    }

    #[proptest]
    #[cfg_attr(miri, ignore)]
    fn proptest_to_compact_string_i8(val: i8) {
        let compact = val.to_compact_string();
        prop_assert_eq!(compact.as_str(), val.to_string());
    }

    #[proptest]
    #[cfg_attr(miri, ignore)]
    fn proptest_to_compact_string_u16(val: u16) {
        let compact = val.to_compact_string();
        prop_assert_eq!(compact.as_str(), val.to_string());
    }

    #[proptest]
    #[cfg_attr(miri, ignore)]
    fn proptest_to_compact_string_i16(val: i16) {
        let compact = val.to_compact_string();
        prop_assert_eq!(compact.as_str(), val.to_string());
    }
    #[proptest]
    #[cfg_attr(miri, ignore)]
    fn proptest_to_compact_string_u32(val: u32) {
        let compact = val.to_compact_string();
        prop_assert_eq!(compact.as_str(), val.to_string());
    }
    #[proptest]
    #[cfg_attr(miri, ignore)]
    fn proptest_to_compact_string_i32(val: i32) {
        let compact = val.to_compact_string();
        prop_assert_eq!(compact.as_str(), val.to_string());
    }
    #[proptest]
    #[cfg_attr(miri, ignore)]
    fn proptest_to_compact_string_u64(val: u64) {
        let compact = val.to_compact_string();
        prop_assert_eq!(compact.as_str(), val.to_string());
    }
    #[proptest]
    #[cfg_attr(miri, ignore)]
    fn proptest_to_compact_string_i64(val: i64) {
        let compact = val.to_compact_string();
        prop_assert_eq!(compact.as_str(), val.to_string());
    }
    #[proptest]
    #[cfg_attr(miri, ignore)]
    fn proptest_to_compact_string_usize(val: usize) {
        let compact = val.to_compact_string();
        prop_assert_eq!(compact.as_str(), val.to_string());
    }
    #[proptest]
    #[cfg_attr(miri, ignore)]
    fn proptest_to_compact_string_isize(val: isize) {
        let compact = val.to_compact_string();
        prop_assert_eq!(compact.as_str(), val.to_string());
    }
    #[proptest]
    #[cfg_attr(miri, ignore)]
    fn proptest_to_compact_string_u128(val: u128) {
        let compact = val.to_compact_string();
        prop_assert_eq!(compact.as_str(), val.to_string());
    }
    #[proptest]
    #[cfg_attr(miri, ignore)]
    fn proptest_to_compact_string_i128(val: i128) {
        let compact = val.to_compact_string();
        prop_assert_eq!(compact.as_str(), val.to_string());
    }

    #[proptest]
    #[cfg_attr(miri, ignore)]
    fn proptest_to_compact_string_non_zero_u8(
        #[strategy((1..=u8::MAX).prop_map(|x| unsafe { num::NonZeroU8::new_unchecked(x)} ))]
        val: num::NonZeroU8,
    ) {
        let compact = val.to_compact_string();
        prop_assert_eq!(compact.as_str(), val.to_string());
    }

    #[proptest]
    #[cfg_attr(miri, ignore)]
    fn proptest_to_compact_string_non_zero_u16(
        #[strategy((1..=u16::MAX).prop_map(|x| unsafe { num::NonZeroU16::new_unchecked(x)} ))]
        val: num::NonZeroU16,
    ) {
        let compact = val.to_compact_string();
        prop_assert_eq!(compact.as_str(), val.to_string());
    }

    #[proptest]
    #[cfg_attr(miri, ignore)]
    fn proptest_to_compact_string_non_zero_u32(
        #[strategy((1..=u32::MAX).prop_map(|x| unsafe { num::NonZeroU32::new_unchecked(x)} ))]
        val: num::NonZeroU32,
    ) {
        let compact = val.to_compact_string();
        prop_assert_eq!(compact.as_str(), val.to_string());
    }

    #[proptest]
    #[cfg_attr(miri, ignore)]
    fn proptest_to_compact_string_non_zero_u64(
        #[strategy((1..=u64::MAX).prop_map(|x| unsafe { num::NonZeroU64::new_unchecked(x)} ))]
        val: num::NonZeroU64,
    ) {
        let compact = val.to_compact_string();
        prop_assert_eq!(compact.as_str(), val.to_string());
    }

    #[proptest]
    #[cfg_attr(miri, ignore)]
    fn proptest_to_compact_string_non_zero_u128(
        #[strategy((1..=u128::MAX).prop_map(|x| unsafe { num::NonZeroU128::new_unchecked(x)} ))]
        val: num::NonZeroU128,
    ) {
        let compact = val.to_compact_string();
        prop_assert_eq!(compact.as_str(), val.to_string());
    }

    #[proptest]
    #[cfg_attr(miri, ignore)]
    fn proptest_to_compact_string_non_zero_usize(
        #[strategy((1..=usize::MAX).prop_map(|x| unsafe { num::NonZeroUsize::new_unchecked(x)} ))]
        val: num::NonZeroUsize,
    ) {
        let compact = val.to_compact_string();
        prop_assert_eq!(compact.as_str(), val.to_string());
    }

    #[proptest]
    #[cfg_attr(miri, ignore)]
    fn proptest_to_compact_string_non_zero_i8(
        #[strategy((1..=u8::MAX).prop_map(|x| unsafe { num::NonZeroI8::new_unchecked(x as i8)} ))]
        val: num::NonZeroI8,
    ) {
        let compact = val.to_compact_string();
        prop_assert_eq!(compact.as_str(), val.to_string());
    }

    #[proptest]
    #[cfg_attr(miri, ignore)]
    fn proptest_to_compact_string_non_zero_i16(
        #[strategy((1..=u16::MAX).prop_map(|x| unsafe { num::NonZeroI16::new_unchecked(x as i16)} ))]
        val: num::NonZeroI16,
    ) {
        let compact = val.to_compact_string();
        prop_assert_eq!(compact.as_str(), val.to_string());
    }

    #[proptest]
    #[cfg_attr(miri, ignore)]
    fn proptest_to_compact_string_non_zero_i32(
        #[strategy((1..=u32::MAX).prop_map(|x| unsafe { num::NonZeroI32::new_unchecked(x as i32)} ))]
        val: num::NonZeroI32,
    ) {
        let compact = val.to_compact_string();
        prop_assert_eq!(compact.as_str(), val.to_string());
    }

    #[proptest]
    #[cfg_attr(miri, ignore)]
    fn proptest_to_compact_string_non_zero_i64(
        #[strategy((1..=u64::MAX).prop_map(|x| unsafe { num::NonZeroI64::new_unchecked(x as i64)} ))]
        val: num::NonZeroI64,
    ) {
        let compact = val.to_compact_string();
        prop_assert_eq!(compact.as_str(), val.to_string());
    }

    #[proptest]
    #[cfg_attr(miri, ignore)]
    fn proptest_to_compact_string_non_zero_i128(
        #[strategy((1..=u128::MAX).prop_map(|x| unsafe { num::NonZeroI128::new_unchecked(x as i128)} ))]
        val: num::NonZeroI128,
    ) {
        let compact = val.to_compact_string();
        prop_assert_eq!(compact.as_str(), val.to_string());
    }

    #[proptest]
    #[cfg_attr(miri, ignore)]
    fn proptest_to_compact_string_non_zero_isize(
        #[strategy((1..=usize::MAX).prop_map(|x| unsafe { num::NonZeroIsize::new_unchecked(x as isize)} ))]
        val: num::NonZeroIsize,
    ) {
        let compact = val.to_compact_string();
        prop_assert_eq!(compact.as_str(), val.to_string());
    }
