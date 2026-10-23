
    use super::compare_bigint_and_f64;
    use num_bigint::BigInt;
    use std::cmp::Ordering;

    #[test]
    fn test_compare_bigint_and_f64() {
        assert_eq!(compare_bigint_and_f64(&BigInt::from(1), f64::NAN), None);
        assert_eq!(compare_bigint_and_f64(&BigInt::from(1), f64::INFINITY), None);
        assert_eq!(compare_bigint_and_f64(&BigInt::from(1), f64::NEG_INFINITY), None);

        assert_eq!(compare_bigint_and_f64(&BigInt::from(1), 0.0), Some(Ordering::Greater));
        assert_eq!(compare_bigint_and_f64(&BigInt::from(0), 0.0), Some(Ordering::Equal));
        assert_eq!(compare_bigint_and_f64(&BigInt::from(-1), 0.0), Some(Ordering::Less));

        assert_eq!(compare_bigint_and_f64(&BigInt::from(1), 0.9), Some(Ordering::Greater));
        assert_eq!(compare_bigint_and_f64(&BigInt::from(1), 1.0), Some(Ordering::Equal));
        assert_eq!(compare_bigint_and_f64(&BigInt::from(1), 1.1), Some(Ordering::Less));

        assert_eq!(compare_bigint_and_f64(&BigInt::from(-1), -1.1), Some(Ordering::Greater));
        assert_eq!(compare_bigint_and_f64(&BigInt::from(-1), -1.0), Some(Ordering::Equal));
        assert_eq!(compare_bigint_and_f64(&BigInt::from(-1), -0.9), Some(Ordering::Less));
    }
