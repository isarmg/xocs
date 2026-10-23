
    use super::Capacity;

    #[test]
    fn test_zero_roundtrips() {
        let og = 0;
        let cap = Capacity::new(og);
        let after = unsafe { cap.as_usize() };

        assert_eq!(og, after);
    }

    #[test]
    fn test_max_value() {
        let available_bytes = (core::mem::size_of::<usize>() - 1) as u32;
        let max_value = 2usize.pow(available_bytes * 8) - 2;

        #[cfg(target_pointer_width = "64")]
        assert_eq!(max_value, 72057594037927934);
        #[cfg(target_pointer_width = "32")]
        assert_eq!(max_value, 16777214);

        let cap = Capacity::new(max_value);
        let after = unsafe { cap.as_usize() };

        assert_eq!(max_value, after);
    }

    #[cfg(target_pointer_width = "32")]
    #[test]
    fn test_invalid_value() {
        let invalid_val = usize::MAX;
        let cap = Capacity::new(invalid_val);
        let after = unsafe { cap.as_usize() };

        // anything greater than or equal to 16777215, should "resolve" to 16777215
        assert_eq!(16777215, after);
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_all_valid_32bit_values() {
        #[cfg(target_pointer_width = "32")]
        assert_eq!(16_777_214, super::MAX_VALUE);

        for i in 0..=16_777_214 {
            let cap = Capacity::new(i);
            let val = unsafe { cap.as_usize() };

            assert_eq!(val, i, "value roundtriped to wrong value?");
        }
    }
