macro_rules! array_size_tests {
    ($testname:ident, $($len:expr => $ty:ident),+) => {
        #[test]
        fn $testname() {
            use typenum::Unsigned;
            $(
                assert_eq!($len, $ty::USIZE);
            )+
        }
    };
}
