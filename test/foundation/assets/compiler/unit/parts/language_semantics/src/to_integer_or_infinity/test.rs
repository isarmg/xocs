
    use super::*;

    #[test]
    fn test_to_integer_or_infinity() {
        assert_eq!(f64::NAN.to_integer_or_infinity(), 0.0);
        assert_eq!(0.0.to_integer_or_infinity(), 0.0);
        assert_eq!(f64::INFINITY.to_integer_or_infinity(), f64::INFINITY);
        assert_eq!(f64::NEG_INFINITY.to_integer_or_infinity(), f64::NEG_INFINITY);
        assert_eq!(1.0.to_integer_or_infinity(), 1.0);
        assert_eq!(-1.0.to_integer_or_infinity(), -1.0);
        assert_eq!(1.1.to_integer_or_infinity(), 1.0);
        assert_eq!(-1.1.to_integer_or_infinity(), -1.0);
    }
