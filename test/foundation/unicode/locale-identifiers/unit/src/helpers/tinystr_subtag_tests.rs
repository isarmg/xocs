macro_rules! tinystr_subtag_tests {
    ($name:ident, [$good_example:literal $(,$more_good_examples:literal)*], [$bad_example:literal $(, $more_bad_examples:literal)*]) => {
        #[test]
        fn test_construction() {
            let maybe = $name::try_from_utf8($good_example.as_bytes());
            assert!(maybe.is_ok());
            assert_eq!(maybe, $name::try_from_raw(maybe.unwrap().into_raw()));
            assert_eq!(maybe.unwrap().as_str(), $good_example);
            $(
                let maybe = $name::try_from_utf8($more_good_examples.as_bytes());
                assert!(maybe.is_ok());
                assert_eq!(maybe, $name::try_from_raw(maybe.unwrap().into_raw()));
                assert_eq!(maybe.unwrap().as_str(), $more_good_examples);
            )*
            assert!($name::try_from_utf8($bad_example.as_bytes()).is_err());
            $(
                assert!($name::try_from_utf8($more_bad_examples.as_bytes()).is_err());
            )*
        }

        #[test]
        fn test_writeable() {
            writeable::assert_writeable_eq!(&$good_example.parse::<$name>().unwrap(), $good_example);
            $(
                writeable::assert_writeable_eq!($more_good_examples.parse::<$name>().unwrap(), $more_good_examples);
            )*
        }

    };
}
