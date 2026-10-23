
    use super::*;

    #[test]
    fn non_static_type_comparisons() {
        assert!(type_eq_non_static::<u8, u8>());
        assert!(type_eq_non_static::<&'static u8, &'static u8>());
        assert!(type_eq_non_static::<&u8, &'static u8>());

        assert!(!type_eq_non_static::<u8, i8>());
        assert!(!type_eq_non_static::<u8, &'static u8>());
    }
