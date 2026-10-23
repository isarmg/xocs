
    use alloc::string::ToString;

    use super::IntoRepr;

    #[test]
    fn test_from_u8_sanity() {
        let vals = [u8::MIN, 1, 0, 42, u8::MAX - 1, u8::MAX];

        for x in &vals {
            let repr = u8::into_repr(*x).unwrap();
            assert_eq!(repr.as_str(), x.to_string());
        }
    }

    #[test]
    fn test_from_i8_sanity() {
        let vals = [i8::MIN, i8::MIN + 1, 0, 42, i8::MAX - 1, i8::MAX];

        for x in &vals {
            let repr = i8::into_repr(*x).unwrap();
            assert_eq!(repr.as_str(), x.to_string());
        }
    }

    #[test]
    fn test_from_u16_sanity() {
        let vals = [u16::MIN, 1, 0, 42, u16::MAX - 1, u16::MAX];

        for x in &vals {
            let repr = u16::into_repr(*x).unwrap();
            assert_eq!(repr.as_str(), x.to_string());
        }
    }

    #[test]
    fn test_from_i16_sanity() {
        let vals = [i16::MIN, i16::MIN + 1, 0, 42, i16::MAX - 1, i16::MAX];

        for x in &vals {
            let repr = i16::into_repr(*x).unwrap();
            assert_eq!(repr.as_str(), x.to_string());
        }
    }

    #[test]
    fn test_from_u32_sanity() {
        let vals = [u32::MIN, 1, 0, 42, u32::MAX - 1, u32::MAX];

        for x in &vals {
            let repr = u32::into_repr(*x).unwrap();
            assert_eq!(repr.as_str(), x.to_string());
        }
    }

    #[test]
    fn test_from_i32_sanity() {
        let vals = [i32::MIN, i32::MIN + 1, 0, 42, i32::MAX - 1, i32::MAX];

        for x in &vals {
            let repr = i32::into_repr(*x).unwrap();
            assert_eq!(repr.as_str(), x.to_string());
        }
    }

    #[test]
    fn test_from_u64_sanity() {
        let vals = [u64::MIN, 1, 0, 42, u64::MAX - 1, u64::MAX];

        for x in &vals {
            let repr = u64::into_repr(*x).unwrap();
            assert_eq!(repr.as_str(), x.to_string());
        }
    }

    #[test]
    fn test_from_i64_sanity() {
        let vals = [i64::MIN, i64::MIN + 1, 0, 42, i64::MAX - 1, i64::MAX];

        for x in &vals {
            let repr = i64::into_repr(*x).unwrap();
            assert_eq!(repr.as_str(), x.to_string());
        }
    }

    #[test]
    fn test_from_usize_sanity() {
        let vals = [usize::MIN, 1, 0, 42, usize::MAX - 1, usize::MAX];

        for x in &vals {
            let repr = usize::into_repr(*x).unwrap();
            assert_eq!(repr.as_str(), x.to_string());
        }
    }

    #[test]
    fn test_from_isize_sanity() {
        let vals = [
            isize::MIN,
            isize::MIN + 1,
            0,
            42,
            isize::MAX - 1,
            isize::MAX,
        ];

        for x in &vals {
            let repr = isize::into_repr(*x).unwrap();
            assert_eq!(repr.as_str(), x.to_string());
        }
    }

    #[test]
    fn test_from_u128_sanity() {
        let vals = [u128::MIN, 1, 0, 42, u128::MAX - 1, u128::MAX];

        for x in &vals {
            let repr = u128::into_repr(*x).unwrap();
            assert_eq!(repr.as_str(), x.to_string());
        }
    }

    #[test]
    fn test_from_i128_sanity() {
        let vals = [i128::MIN, i128::MIN + 1, 0, 42, i128::MAX - 1, i128::MAX];

        for x in &vals {
            let repr = i128::into_repr(*x).unwrap();
            assert_eq!(repr.as_str(), x.to_string());
        }
    }
