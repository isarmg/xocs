
    use super::*;

    #[test]
    #[expect(clippy::cast_precision_loss)]
    fn f64_to_int32_conversion() {
        assert_eq!(0.0_f64.to_int_32(), 0);
        assert_eq!((-0.0_f64).to_int_32(), 0);
        assert_eq!(f64::NAN.to_int_32(), 0);
        assert_eq!(f64::INFINITY.to_int_32(), 0);
        assert_eq!(f64::NEG_INFINITY.to_int_32(), 0);
        assert_eq!(((i64::from(i32::MAX) + 1) as f64).to_int_32(), i32::MIN);
        assert_eq!(((i64::from(i32::MIN) - 1) as f64).to_int_32(), i32::MAX);

        // Test edge cases with maximum safe integers
        assert_eq!((9_007_199_254_740_992.0_f64).to_int_32(), 0); // 2^53
        assert_eq!((-9_007_199_254_740_992.0_f64).to_int_32(), 0); // -2^53
    }

    #[test]
    fn test_generic_and_arm64_consistency() {
        // Test that generic implementation gives same results as ARM64 implementation
        // when both are available (ARM64 with JSCVT support)
        let test_values = [
            0.0,
            -0.0,
            1.0,
            -1.0,
            42.7,
            -42.7,
            f64::from(i32::MAX),
            f64::from(i32::MIN),
            f64::from(i32::MAX) + 1.0,
            f64::from(i32::MIN) - 1.0,
            9_007_199_254_740_992.0,  // 2^53
            -9_007_199_254_740_992.0, // -2^53
        ];

        for &value in &test_values {
            let generic_result = f64_to_int32_generic(value);
            let trait_result = value.to_int_32();
            assert_eq!(
                generic_result, trait_result,
                "Mismatch for value {value}: generic={generic_result}, trait={trait_result}"
            );
        }
    }

    #[test]
    fn test_nan_handling() {
        // Both implementations should handle NaN the same way
        assert_eq!(f64::NAN.to_int_32(), 0);
        assert_eq!(f64_to_int32_generic(f64::NAN), 0);

        #[cfg(target_arch = "aarch64")]
        {
            // SAFETY: This is a test and we're only testing NaN handling
            unsafe {
                assert_eq!(f64_to_int32_arm64(f64::NAN), 0);
            }
        }
    }

    #[test]
    #[expect(clippy::cast_precision_loss)]
    fn f64_to_uint32_conversion() {
        assert_eq!(0.0_f64.to_uint_32(), 0);
        assert_eq!((-0.0_f64).to_uint_32(), 0);
        assert_eq!(f64::NAN.to_uint_32(), 0);
        assert_eq!(f64::INFINITY.to_uint_32(), 0);
        assert_eq!(f64::NEG_INFINITY.to_uint_32(), 0);
        assert_eq!(((i64::from(u32::MAX) + 1) as f64).to_uint_32(), u32::MIN);
        assert_eq!(((i64::from(u32::MIN) - 1) as f64).to_uint_32(), u32::MAX);

        // Test edge cases with maximum safe integers
        assert_eq!((9_007_199_254_740_992.0_f64).to_uint_32(), 0); // 2^53
        assert_eq!((-9_007_199_254_740_992.0_f64).to_uint_32(), 0); // -2^53
    }
